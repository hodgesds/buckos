//! Service definition types and states for the init system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Type of service execution model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    /// Simple service - main process is the service
    Simple,
    /// Forking service - forks and parent exits
    Forking,
    /// Oneshot service - runs once and exits
    Oneshot,
    /// Notify service - sends notification when ready
    Notify,
    /// Idle service - runs when system is idle
    Idle,
}

impl Default for ServiceType {
    fn default() -> Self {
        ServiceType::Simple
    }
}

/// Service restart policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    /// Never restart
    No,
    /// Restart on success (exit 0)
    OnSuccess,
    /// Restart on failure (non-zero exit)
    OnFailure,
    /// Restart on abnormal exit (signal, timeout)
    OnAbnormal,
    /// Always restart
    Always,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        RestartPolicy::OnFailure
    }
}

/// Current state of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    /// Service is inactive and not running
    Inactive,
    /// Service is starting up
    Starting,
    /// Service is running
    Running,
    /// Service is stopping
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service has failed
    Failed,
    /// Service is reloading configuration
    Reloading,
}

impl Default for ServiceState {
    fn default() -> Self {
        ServiceState::Inactive
    }
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceState::Inactive => write!(f, "inactive"),
            ServiceState::Starting => write!(f, "starting"),
            ServiceState::Running => write!(f, "running"),
            ServiceState::Stopping => write!(f, "stopping"),
            ServiceState::Stopped => write!(f, "stopped"),
            ServiceState::Failed => write!(f, "failed"),
            ServiceState::Reloading => write!(f, "reloading"),
        }
    }
}

/// Service definition - describes how to run a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Unique name of the service
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Type of service
    #[serde(default)]
    pub service_type: ServiceType,
    /// Command to execute
    pub exec_start: String,
    /// Command to stop the service (optional, defaults to SIGTERM)
    pub exec_stop: Option<String>,
    /// Command to reload the service
    pub exec_reload: Option<String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// User to run as
    pub user: Option<String>,
    /// Group to run as
    pub group: Option<String>,
    /// Services this depends on (must start first)
    #[serde(default)]
    pub requires: Vec<String>,
    /// Services this wants (should start first, but not required)
    #[serde(default)]
    pub wants: Vec<String>,
    /// Services that must start after this one
    #[serde(default)]
    pub before: Vec<String>,
    /// Services that must start before this one
    #[serde(default)]
    pub after: Vec<String>,
    /// Restart policy
    #[serde(default)]
    pub restart: RestartPolicy,
    /// Delay before restarting
    #[serde(default = "default_restart_sec")]
    #[serde(with = "humantime_serde")]
    pub restart_sec: Duration,
    /// Maximum time to wait for service to start
    #[serde(default = "default_timeout_start")]
    #[serde(with = "humantime_serde")]
    pub timeout_start_sec: Duration,
    /// Maximum time to wait for service to stop
    #[serde(default = "default_timeout_stop")]
    #[serde(with = "humantime_serde")]
    pub timeout_stop_sec: Duration,
    /// Whether to enable this service by default
    #[serde(default)]
    pub enabled: bool,
}

fn default_restart_sec() -> Duration {
    Duration::from_secs(1)
}

fn default_timeout_start() -> Duration {
    Duration::from_secs(30)
}

fn default_timeout_stop() -> Duration {
    Duration::from_secs(30)
}

/// Module for humantime serialization.
mod humantime_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl ServiceDefinition {
    /// Create a new service definition with minimal configuration.
    pub fn new(name: impl Into<String>, exec_start: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            service_type: ServiceType::default(),
            exec_start: exec_start.into(),
            exec_stop: None,
            exec_reload: None,
            working_directory: None,
            environment: HashMap::new(),
            user: None,
            group: None,
            requires: Vec::new(),
            wants: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
            restart: RestartPolicy::default(),
            restart_sec: default_restart_sec(),
            timeout_start_sec: default_timeout_start(),
            timeout_stop_sec: default_timeout_stop(),
            enabled: false,
        }
    }

    /// Load a service definition from a TOML file.
    pub fn from_file(path: &std::path::Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let def: ServiceDefinition = toml::from_str(&content)?;
        Ok(def)
    }

    /// Save the service definition to a TOML file.
    pub fn to_file(&self, path: &std::path::Path) -> crate::error::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::Error::ConfigError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Runtime information about a running service instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// Unique instance ID
    pub id: Uuid,
    /// Service name
    pub name: String,
    /// Current state
    pub state: ServiceState,
    /// Main process ID (if running)
    pub main_pid: Option<u32>,
    /// Control process ID (if any)
    pub control_pid: Option<u32>,
    /// Time when the service was started
    pub started_at: Option<DateTime<Utc>>,
    /// Time when the service stopped
    pub stopped_at: Option<DateTime<Utc>>,
    /// Exit code (if stopped)
    pub exit_code: Option<i32>,
    /// Exit signal (if killed by signal)
    pub exit_signal: Option<i32>,
    /// Number of restarts
    pub restart_count: u32,
    /// Last failure reason
    pub failure_reason: Option<String>,
}

impl ServiceInstance {
    /// Create a new service instance.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            state: ServiceState::Inactive,
            main_pid: None,
            control_pid: None,
            started_at: None,
            stopped_at: None,
            exit_code: None,
            exit_signal: None,
            restart_count: 0,
            failure_reason: None,
        }
    }

    /// Check if the service is active (running or starting).
    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            ServiceState::Running | ServiceState::Starting | ServiceState::Reloading
        )
    }

    /// Check if the service has failed.
    pub fn is_failed(&self) -> bool {
        self.state == ServiceState::Failed
    }

    /// Get the uptime of the service.
    pub fn uptime(&self) -> Option<Duration> {
        self.started_at.map(|start| {
            let now = Utc::now();
            let duration = now.signed_duration_since(start);
            Duration::from_secs(duration.num_seconds().max(0) as u64)
        })
    }
}

/// Service status information for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Current state
    pub state: ServiceState,
    /// Description
    pub description: String,
    /// Main PID
    pub main_pid: Option<u32>,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// CPU usage percentage
    pub cpu_percent: Option<f64>,
    /// Uptime
    pub uptime_secs: Option<u64>,
    /// Number of restarts
    pub restart_count: u32,
}

impl ServiceStatus {
    /// Create status from definition and instance.
    pub fn from_service(def: &ServiceDefinition, instance: &ServiceInstance) -> Self {
        Self {
            name: def.name.clone(),
            state: instance.state,
            description: def.description.clone(),
            main_pid: instance.main_pid,
            memory_bytes: None, // TODO: Get from /proc
            cpu_percent: None,  // TODO: Get from /proc
            uptime_secs: instance.uptime().map(|d| d.as_secs()),
            restart_count: instance.restart_count,
        }
    }
}
