//! Service manager for tracking and managing services.

use crate::error::{Error, Result};
use crate::journal::Journal;
use crate::process::{ExitStatus, ProcessSupervisor};
use crate::service::{
    HealthStatus, RestartPolicy, ServiceDefinition, ServiceInstance, ServiceState, ServiceStatus,
};
use chrono::Utc;
use nix::sys::signal::Signal;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Boot timing information for a service.
#[derive(Debug, Clone)]
pub struct BootTiming {
    /// Service name
    pub name: String,
    /// Time when service started booting
    pub start_time: Instant,
    /// Time when service finished booting
    pub end_time: Option<Instant>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Dependency graph node.
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Service name
    pub name: String,
    /// Services this depends on
    pub requires: Vec<String>,
    /// Soft dependencies
    pub wants: Vec<String>,
    /// Services that must start after
    pub before: Vec<String>,
    /// Services that must start before
    pub after: Vec<String>,
}

/// Service manager that orchestrates services.
pub struct ServiceManager {
    /// Service definitions
    definitions: Arc<RwLock<HashMap<String, ServiceDefinition>>>,
    /// Service instances
    instances: Arc<RwLock<HashMap<String, ServiceInstance>>>,
    /// Process supervisor
    supervisor: Arc<ProcessSupervisor>,
    /// Services directory
    services_dir: PathBuf,
    /// Journal for logging
    journal: Arc<Journal>,
    /// Boot timings for analysis
    boot_timings: Arc<RwLock<Vec<BootTiming>>>,
    /// System boot start time
    boot_start: Instant,
}

impl ServiceManager {
    /// Create a new service manager.
    pub fn new(services_dir: PathBuf) -> Self {
        let log_dir = services_dir.parent()
            .unwrap_or(&services_dir)
            .join("logs");

        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            supervisor: Arc::new(ProcessSupervisor::new()),
            services_dir,
            journal: Arc::new(Journal::new(log_dir)),
            boot_timings: Arc::new(RwLock::new(Vec::new())),
            boot_start: Instant::now(),
        }
    }

    /// Get a reference to the journal.
    pub fn journal(&self) -> Arc<Journal> {
        Arc::clone(&self.journal)
    }

    /// Load all service definitions from the services directory.
    pub async fn load_services(&self) -> Result<()> {
        if !self.services_dir.exists() {
            info!(dir = ?self.services_dir, "Services directory doesn't exist, creating");
            std::fs::create_dir_all(&self.services_dir)?;
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.services_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match ServiceDefinition::from_file(&path) {
                    Ok(def) => {
                        info!(service = %def.name, "Loaded service definition");
                        self.register_service(def).await?;
                    }
                    Err(e) => {
                        error!(path = ?path, error = %e, "Failed to load service definition");
                    }
                }
            }
        }

        Ok(())
    }

    /// Register a service definition.
    pub async fn register_service(&self, def: ServiceDefinition) -> Result<()> {
        let name = def.name.clone();

        // Check for duplicate
        if self.definitions.read().await.contains_key(&name) {
            return Err(Error::ServiceAlreadyExists(name));
        }

        // Create instance
        let instance = ServiceInstance::new(&name);

        self.definitions.write().await.insert(name.clone(), def);
        self.instances.write().await.insert(name, instance);

        Ok(())
    }

    /// Start a service by name.
    pub async fn start_service(&self, name: &str) -> Result<()> {
        let start_time = Instant::now();

        // Get the service definition
        let def = self
            .definitions
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        // Check if masked
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(name) {
                if instance.masked {
                    return Err(Error::ServiceMasked(name.to_string()));
                }
            }
        }

        // Check if already running
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(name) {
                if instance.is_active() {
                    info!(service = %name, "Service is already running");
                    return Ok(());
                }
            }
        }

        // Start dependencies first
        for dep in &def.requires {
            Box::pin(self.start_service(dep)).await.map_err(|e| Error::DependencyError {
                service: name.to_string(),
                dependency: dep.clone(),
                reason: e.to_string(),
            })?;
        }

        // Start wanted services (ignore failures)
        for dep in &def.wants {
            if let Err(e) = Box::pin(self.start_service(dep)).await {
                warn!(service = %name, dependency = %dep, error = %e, "Failed to start wanted service");
            }
        }

        // Update state to starting
        self.set_state(name, ServiceState::Starting).await?;

        info!(service = %name, "Starting service");

        // Spawn the process
        match self.supervisor.spawn(&def, Arc::clone(&self.journal)).await {
            Ok(pid) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;

                // Update instance with PID and running state
                let mut instances = self.instances.write().await;
                if let Some(instance) = instances.get_mut(name) {
                    instance.main_pid = Some(pid);
                    instance.started_at = Some(Utc::now());
                    instance.state = ServiceState::Running;
                    instance.exit_code = None;
                    instance.exit_signal = None;
                    instance.failure_reason = None;
                    instance.boot_duration_ms = Some(duration_ms);

                    // Set initial health status if health check is configured
                    if def.health_check.is_some() {
                        instance.health_status = HealthStatus::Starting;
                    }
                }

                // Record boot timing
                self.boot_timings.write().await.push(BootTiming {
                    name: name.to_string(),
                    start_time,
                    end_time: Some(Instant::now()),
                    duration_ms,
                });

                info!(service = %name, pid = pid, duration_ms = duration_ms, "Service started");
                Ok(())
            }
            Err(e) => {
                // Update state to failed
                let mut instances = self.instances.write().await;
                if let Some(instance) = instances.get_mut(name) {
                    instance.state = ServiceState::Failed;
                    instance.failure_reason = Some(e.to_string());
                }

                error!(service = %name, error = %e, "Failed to start service");
                Err(Error::ServiceStartFailed {
                    name: name.to_string(),
                    reason: e.to_string(),
                })
            }
        }
    }

    /// Stop a service by name.
    pub async fn stop_service(&self, name: &str) -> Result<()> {
        // Get the service definition
        let def = self
            .definitions
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        // Get current instance
        let pid = {
            let instances = self.instances.read().await;
            let instance = instances
                .get(name)
                .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

            if !instance.is_active() {
                info!(service = %name, "Service is not running");
                return Ok(());
            }

            instance.main_pid
        };

        // Update state to stopping
        self.set_state(name, ServiceState::Stopping).await?;

        info!(service = %name, "Stopping service");

        // Stop the process
        if let Some(pid) = pid {
            match self.supervisor.stop(pid, def.timeout_stop_sec).await {
                Ok(status) => {
                    // Update instance
                    let mut instances = self.instances.write().await;
                    if let Some(instance) = instances.get_mut(name) {
                        instance.main_pid = None;
                        instance.stopped_at = Some(Utc::now());
                        instance.state = ServiceState::Stopped;
                        instance.exit_code = status.code;
                        instance.exit_signal = status.signal;
                    }

                    info!(service = %name, "Service stopped");
                    Ok(())
                }
                Err(e) => {
                    error!(service = %name, error = %e, "Failed to stop service");
                    Err(Error::ServiceStopFailed {
                        name: name.to_string(),
                        reason: e.to_string(),
                    })
                }
            }
        } else {
            // No PID, just mark as stopped
            self.set_state(name, ServiceState::Stopped).await?;
            Ok(())
        }
    }

    /// Restart a service by name.
    pub async fn restart_service(&self, name: &str) -> Result<()> {
        self.stop_service(name).await?;
        self.start_service(name).await
    }

    /// Reload a service by name.
    pub async fn reload_service(&self, name: &str) -> Result<()> {
        // Get the service definition
        let def = self
            .definitions
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        // Get current instance
        let pid = {
            let instances = self.instances.read().await;
            let instance = instances
                .get(name)
                .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

            if !instance.is_active() {
                return Err(Error::ServiceReloadFailed {
                    name: name.to_string(),
                    reason: "Service is not running".to_string(),
                });
            }

            instance.main_pid
        };

        // Update state to reloading
        self.set_state(name, ServiceState::Reloading).await?;

        info!(service = %name, "Reloading service");

        // If there's a custom reload command, run it
        if let Some(ref reload_cmd) = def.exec_reload {
            // Execute reload command
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(reload_cmd)
                .output()
                .await
                .map_err(|e| Error::ServiceReloadFailed {
                    name: name.to_string(),
                    reason: e.to_string(),
                })?;

            if !output.status.success() {
                self.set_state(name, ServiceState::Running).await?;
                return Err(Error::ServiceReloadFailed {
                    name: name.to_string(),
                    reason: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            }
        } else if let Some(pid) = pid {
            // Send SIGHUP to the process
            self.supervisor.signal(pid, Signal::SIGHUP).await?;
        }

        // Set back to running
        self.set_state(name, ServiceState::Running).await?;

        info!(service = %name, "Service reloaded");
        Ok(())
    }

    /// Enable a service for auto-start.
    pub async fn enable_service(&self, name: &str) -> Result<()> {
        let mut definitions = self.definitions.write().await;
        let def = definitions
            .get_mut(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        def.enabled = true;

        // Save to file
        let path = self.services_dir.join(format!("{}.toml", name));
        def.to_file(&path)?;

        info!(service = %name, "Service enabled");
        Ok(())
    }

    /// Disable a service from auto-start.
    pub async fn disable_service(&self, name: &str) -> Result<()> {
        let mut definitions = self.definitions.write().await;
        let def = definitions
            .get_mut(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        def.enabled = false;

        // Save to file
        let path = self.services_dir.join(format!("{}.toml", name));
        def.to_file(&path)?;

        info!(service = %name, "Service disabled");
        Ok(())
    }

    /// Mask a service to prevent it from starting.
    pub async fn mask_service(&self, name: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        instance.masked = true;

        info!(service = %name, "Service masked");
        Ok(())
    }

    /// Unmask a service to allow it to start.
    pub async fn unmask_service(&self, name: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        instance.masked = false;

        info!(service = %name, "Service unmasked");
        Ok(())
    }

    /// Start all enabled services in parallel.
    pub async fn start_enabled_services_parallel(&self) -> Result<()> {
        // Build dependency graph and find services that can start in parallel
        let enabled: Vec<String> = self
            .definitions
            .read()
            .await
            .iter()
            .filter(|(_, def)| def.enabled)
            .map(|(name, _)| name.clone())
            .collect();

        if enabled.is_empty() {
            return Ok(());
        }

        // Topologically sort services based on dependencies
        let sorted = self.topological_sort(&enabled).await?;

        // Group services by dependency level for parallel execution
        let levels = self.group_by_dependency_level(&sorted).await;

        for level in levels {
            // Start all services in this level in parallel
            let handles: Vec<_> = level
                .iter()
                .map(|name| {
                    let manager = self.clone_for_restart();
                    let name = name.clone();
                    tokio::spawn(async move {
                        if let Err(e) = manager.start_service(&name).await {
                            error!(service = %name, error = %e, "Failed to start service in parallel");
                        }
                    })
                })
                .collect();

            // Wait for all services in this level to start
            for handle in handles {
                let _ = handle.await;
            }
        }

        Ok(())
    }

    /// Topologically sort services based on dependencies.
    async fn topological_sort(&self, services: &[String]) -> Result<Vec<String>> {
        let definitions = self.definitions.read().await;
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for name in services {
            in_degree.insert(name.clone(), 0);
            graph.insert(name.clone(), Vec::new());
        }

        // Build graph
        for name in services {
            if let Some(def) = definitions.get(name) {
                for dep in &def.requires {
                    if services.contains(dep) {
                        graph.get_mut(dep).unwrap().push(name.clone());
                        *in_degree.get_mut(name).unwrap() += 1;
                    }
                }
                for dep in &def.after {
                    if services.contains(dep) {
                        graph.get_mut(dep).unwrap().push(name.clone());
                        *in_degree.get_mut(name).unwrap() += 1;
                    }
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(name) = queue.pop_front() {
            result.push(name.clone());
            if let Some(dependents) = graph.get(&name) {
                for dep in dependents {
                    let degree = in_degree.get_mut(dep).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        if result.len() != services.len() {
            return Err(Error::CircularDependency(services.to_vec()));
        }

        Ok(result)
    }

    /// Group services by dependency level for parallel execution.
    async fn group_by_dependency_level(&self, sorted: &[String]) -> Vec<Vec<String>> {
        let definitions = self.definitions.read().await;
        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut assigned: HashSet<String> = HashSet::new();

        for name in sorted {
            let def = match definitions.get(name) {
                Some(d) => d,
                None => continue,
            };

            // Find the level this service should be in
            let mut level = 0;
            for dep in def.requires.iter().chain(def.after.iter()) {
                for (i, lvl) in levels.iter().enumerate() {
                    if lvl.contains(dep) {
                        level = level.max(i + 1);
                    }
                }
            }

            // Ensure we have enough levels
            while levels.len() <= level {
                levels.push(Vec::new());
            }

            levels[level].push(name.clone());
            assigned.insert(name.clone());
        }

        levels
    }

    /// Get the dependency graph for all services.
    pub async fn get_dependency_graph(&self) -> Vec<DependencyNode> {
        let definitions = self.definitions.read().await;
        definitions
            .iter()
            .map(|(name, def)| DependencyNode {
                name: name.clone(),
                requires: def.requires.clone(),
                wants: def.wants.clone(),
                before: def.before.clone(),
                after: def.after.clone(),
            })
            .collect()
    }

    /// Get boot analysis (blame) - services sorted by boot time.
    pub async fn get_boot_blame(&self) -> Vec<BootTiming> {
        let mut timings = self.boot_timings.read().await.clone();
        timings.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
        timings
    }

    /// Get critical chain - the chain of services that took longest to start.
    pub async fn get_critical_chain(&self) -> Vec<String> {
        let timings = self.boot_timings.read().await;
        let definitions = self.definitions.read().await;

        // Find the service that took longest
        let slowest = timings.iter().max_by_key(|t| t.duration_ms);

        if let Some(slowest) = slowest {
            let mut chain = vec![slowest.name.clone()];
            let mut current = slowest.name.clone();

            // Trace back through dependencies
            while let Some(def) = definitions.get(&current) {
                if let Some(dep) = def.requires.first().or(def.after.first()) {
                    if !chain.contains(dep) {
                        chain.push(dep.clone());
                        current = dep.clone();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            chain.reverse();
            chain
        } else {
            Vec::new()
        }
    }

    /// Get total boot time in milliseconds.
    pub fn get_total_boot_time(&self) -> u64 {
        self.boot_start.elapsed().as_millis() as u64
    }

    /// Run a health check for a service.
    pub async fn run_health_check(&self, name: &str) -> Result<bool> {
        let def = self
            .definitions
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        let health_check = match def.health_check {
            Some(hc) => hc,
            None => return Ok(true), // No health check = always healthy
        };

        // Run the health check command
        let output = tokio::time::timeout(
            health_check.timeout,
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&health_check.exec)
                .output(),
        )
        .await
        .map_err(|_| Error::HealthCheckFailed {
            name: name.to_string(),
            reason: "Health check timed out".to_string(),
        })?
        .map_err(|e| Error::HealthCheckFailed {
            name: name.to_string(),
            reason: e.to_string(),
        })?;

        let healthy = output.status.success();

        // Update health status
        {
            let mut instances = self.instances.write().await;
            if let Some(instance) = instances.get_mut(name) {
                instance.last_health_check = Some(Utc::now());

                if healthy {
                    instance.health_failures = 0;
                    instance.health_status = HealthStatus::Healthy;
                } else {
                    instance.health_failures += 1;
                    if instance.health_failures >= health_check.retries {
                        instance.health_status = HealthStatus::Unhealthy;
                    }
                }
            }
        }

        Ok(healthy)
    }

    /// Handle a watchdog ping from a service.
    pub async fn watchdog_ping(&self, name: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        instance.last_watchdog_ping = Some(Utc::now());
        Ok(())
    }

    /// Instantiate a template service.
    pub async fn instantiate_template(&self, template_name: &str, instance_name: &str) -> Result<()> {
        let def = self
            .definitions
            .read()
            .await
            .get(template_name)
            .cloned()
            .ok_or_else(|| Error::ServiceNotFound(template_name.to_string()))?;

        if !def.is_template() {
            return Err(Error::TemplateError {
                template: template_name.to_string(),
                instance: instance_name.to_string(),
                reason: "Service is not a template".to_string(),
            });
        }

        let instantiated = def.instantiate(instance_name);
        self.register_service(instantiated).await?;

        info!(template = %template_name, instance = %instance_name, "Instantiated template service");
        Ok(())
    }

    /// Get logs for a service.
    pub async fn get_logs(&self, name: &str, limit: Option<usize>) -> Vec<crate::journal::JournalEntry> {
        self.journal.get_logs(name, limit, false).await
    }

    /// Get the status of a service.
    pub async fn get_status(&self, name: &str) -> Result<ServiceStatus> {
        let definitions = self.definitions.read().await;
        let instances = self.instances.read().await;

        let def = definitions
            .get(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        let instance = instances
            .get(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        Ok(ServiceStatus::from_service(def, instance))
    }

    /// Get status of all services.
    pub async fn get_all_status(&self) -> Vec<ServiceStatus> {
        let definitions = self.definitions.read().await;
        let instances = self.instances.read().await;

        definitions
            .iter()
            .filter_map(|(name, def)| {
                instances
                    .get(name)
                    .map(|instance| ServiceStatus::from_service(def, instance))
            })
            .collect()
    }

    /// List all service names.
    pub async fn list_services(&self) -> Vec<String> {
        self.definitions.read().await.keys().cloned().collect()
    }

    /// Handle a process exit.
    pub async fn handle_process_exit(&self, status: ExitStatus) {
        // Find which service this process belongs to
        let service_name = match self.supervisor.get_service_name(status.pid).await {
            Some(name) => name,
            None => {
                debug!(pid = status.pid, "Unknown process exited");
                return;
            }
        };

        info!(
            service = %service_name,
            pid = status.pid,
            code = ?status.code,
            signal = ?status.signal,
            "Service process exited"
        );

        // Get service definition and instance
        let def = match self.definitions.read().await.get(&service_name).cloned() {
            Some(def) => def,
            None => return,
        };

        // Update instance state
        {
            let mut instances = self.instances.write().await;
            if let Some(instance) = instances.get_mut(&service_name) {
                instance.main_pid = None;
                instance.stopped_at = Some(Utc::now());
                instance.exit_code = status.code;
                instance.exit_signal = status.signal;

                if status.success() {
                    instance.state = ServiceState::Stopped;
                } else {
                    instance.state = ServiceState::Failed;
                    instance.failure_reason = Some(format!(
                        "Process exited with code {:?}, signal {:?}",
                        status.code, status.signal
                    ));
                }
            }
        }

        // Check if we should restart
        let should_restart = match def.restart {
            RestartPolicy::No => false,
            RestartPolicy::Always => true,
            RestartPolicy::OnSuccess => status.success(),
            RestartPolicy::OnFailure => !status.success(),
            RestartPolicy::OnAbnormal => status.signal.is_some(),
        };

        if should_restart {
            // Check restart count
            let restart_count = {
                let instances = self.instances.read().await;
                instances
                    .get(&service_name)
                    .map(|i| i.restart_count)
                    .unwrap_or(0)
            };

            // TODO: Add max restart count and rate limiting
            if restart_count < 5 {
                info!(
                    service = %service_name,
                    restart_count = restart_count,
                    "Scheduling service restart"
                );

                // Increment restart count
                {
                    let mut instances = self.instances.write().await;
                    if let Some(instance) = instances.get_mut(&service_name) {
                        instance.restart_count += 1;
                    }
                }

                // Wait before restarting
                let delay = def.restart_sec;
                let name = service_name.clone();
                let manager = self.clone_for_restart();

                tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    if let Err(e) = manager.start_service(&name).await {
                        error!(service = %name, error = %e, "Failed to restart service");
                    }
                });
            } else {
                warn!(
                    service = %service_name,
                    restart_count = restart_count,
                    "Service exceeded max restart count"
                );
            }
        }
    }

    /// Get the process supervisor.
    pub fn supervisor(&self) -> Arc<ProcessSupervisor> {
        Arc::clone(&self.supervisor)
    }

    /// Set service state.
    async fn set_state(&self, name: &str, state: ServiceState) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;
        instance.state = state;
        Ok(())
    }

    /// Clone for use in restart tasks.
    fn clone_for_restart(&self) -> Self {
        Self {
            definitions: Arc::clone(&self.definitions),
            instances: Arc::clone(&self.instances),
            supervisor: Arc::clone(&self.supervisor),
            services_dir: self.services_dir.clone(),
            journal: Arc::clone(&self.journal),
            boot_timings: Arc::clone(&self.boot_timings),
            boot_start: self.boot_start,
        }
    }

    /// Start all enabled services.
    pub async fn start_enabled_services(&self) -> Result<()> {
        let enabled: Vec<String> = self
            .definitions
            .read()
            .await
            .iter()
            .filter(|(_, def)| def.enabled)
            .map(|(name, _)| name.clone())
            .collect();

        for name in enabled {
            if let Err(e) = self.start_service(&name).await {
                error!(service = %name, error = %e, "Failed to start enabled service");
            }
        }

        Ok(())
    }

    /// Stop all running services.
    pub async fn stop_all_services(&self) -> Result<()> {
        let running: Vec<String> = {
            let instances = self.instances.read().await;
            instances
                .iter()
                .filter(|(_, instance)| instance.is_active())
                .map(|(name, _)| name.clone())
                .collect()
        };

        for name in running {
            if let Err(e) = self.stop_service(&name).await {
                error!(service = %name, error = %e, "Failed to stop service");
            }
        }

        Ok(())
    }
}
