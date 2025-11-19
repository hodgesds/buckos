//! Service manager for tracking and managing services.

use crate::error::{Error, Result};
use crate::process::{ExitStatus, ProcessSupervisor};
use crate::service::{
    RestartPolicy, ServiceDefinition, ServiceInstance, ServiceState, ServiceStatus,
};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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
}

impl ServiceManager {
    /// Create a new service manager.
    pub fn new(services_dir: PathBuf) -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            supervisor: Arc::new(ProcessSupervisor::new()),
            services_dir,
        }
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
        // Get the service definition
        let def = self
            .definitions
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

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
        match self.supervisor.spawn(&def).await {
            Ok(pid) => {
                // Update instance with PID and running state
                let mut instances = self.instances.write().await;
                if let Some(instance) = instances.get_mut(name) {
                    instance.main_pid = Some(pid);
                    instance.started_at = Some(Utc::now());
                    instance.state = ServiceState::Running;
                    instance.exit_code = None;
                    instance.exit_signal = None;
                    instance.failure_reason = None;
                }

                info!(service = %name, pid = pid, "Service started");
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
