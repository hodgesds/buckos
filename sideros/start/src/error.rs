//! Error types for the sideros init system.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for init system operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the init system.
#[derive(Error, Debug)]
pub enum Error {
    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Service already exists
    #[error("Service already exists: {0}")]
    ServiceAlreadyExists(String),

    /// Service failed to start
    #[error("Service failed to start: {name}: {reason}")]
    ServiceStartFailed { name: String, reason: String },

    /// Service failed to stop
    #[error("Service failed to stop: {name}: {reason}")]
    ServiceStopFailed { name: String, reason: String },

    /// Service failed to reload
    #[error("Service failed to reload: {name}: {reason}")]
    ServiceReloadFailed { name: String, reason: String },

    /// Service is masked
    #[error("Service is masked: {0}")]
    ServiceMasked(String),

    /// Service dependency error
    #[error("Service dependency error: {service} depends on {dependency}: {reason}")]
    DependencyError {
        service: String,
        dependency: String,
        reason: String,
    },

    /// Circular dependency detected
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<String>),

    /// Process spawn error
    #[error("Failed to spawn process: {0}")]
    ProcessSpawnFailed(String),

    /// Process not found
    #[error("Process not found: PID {0}")]
    ProcessNotFound(u32),

    /// Signal error
    #[error("Signal error: {0}")]
    SignalError(String),

    /// Mount error
    #[error("Mount error: {source_path} -> {target}: {reason}")]
    MountError {
        source_path: String,
        target: String,
        reason: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid service unit
    #[error("Invalid service unit: {path}: {reason}")]
    InvalidServiceUnit { path: PathBuf, reason: String },

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Not running as PID 1
    #[error("Not running as PID 1 (current PID: {0})")]
    NotPid1(u32),

    /// Health check failed
    #[error("Health check failed for {name}: {reason}")]
    HealthCheckFailed { name: String, reason: String },

    /// Watchdog timeout
    #[error("Watchdog timeout for service: {0}")]
    WatchdogTimeout(String),

    /// Socket activation error
    #[error("Socket activation error for {name}: {reason}")]
    SocketActivationError { name: String, reason: String },

    /// Timer error
    #[error("Timer error for {name}: {reason}")]
    TimerError { name: String, reason: String },

    /// Template instantiation error
    #[error("Failed to instantiate template {template} with instance {instance}: {reason}")]
    TemplateError {
        template: String,
        instance: String,
        reason: String,
    },

    /// Resource limit error
    #[error("Failed to set resource limits: {0}")]
    ResourceLimitError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Nix error
    #[error("System error: {0}")]
    Nix(#[from] nix::Error),
}
