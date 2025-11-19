//! Sideros init system - PID 1 service manager.
//!
//! This crate provides a systemd-like init system for managing services
//! on a Linux system. It is designed to run as PID 1 and handles:
//!
//! - Service lifecycle management (start, stop, restart, reload)
//! - Process supervision and automatic restart
//! - Service dependencies with parallel startup
//! - Signal handling (SIGCHLD, SIGTERM, SIGINT)
//! - Zombie process reaping
//! - Virtual filesystem mounting
//! - Health checks and watchdog support
//! - Socket activation
//! - Timer services
//! - Resource limits
//! - Service templates
//! - Structured logging (journal)
//! - Boot time analysis
//!
//! # Architecture
//!
//! The init system is composed of several components:
//!
//! - **Init**: The main init system that coordinates everything
//! - **ServiceManager**: Manages service definitions and instances
//! - **ProcessSupervisor**: Handles process spawning and supervision
//! - **Journal**: Structured logging for service output
//! - **Service types**: Define service configurations and states
//!
//! # Example
//!
//! ```no_run
//! use sideros_start::{Init, InitConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = InitConfig::default();
//!     let init = Init::new(config)?;
//!     init.run().await?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod init;
pub mod journal;
pub mod loaders;
pub mod manager;
pub mod process;
pub mod service;

// Re-export main types
pub use error::{Error, Result};
pub use init::{create_test_init, Init, InitConfig, ShutdownType};
pub use journal::{Journal, JournalEntry, Priority};
pub use loaders::{LoaderRegistry, ServiceLoader, SystemdLoader, TomlLoader};
pub use manager::{BootTiming, DependencyNode, ServiceManager};
pub use process::{ExitStatus, ProcessSupervisor};
pub use service::{
    HealthCheck, HealthStatus, ResourceLimits, RestartPolicy, ServiceDefinition, ServiceInstance,
    ServiceState, ServiceStatus, ServiceType, SocketConfig, TimerConfig, WatchdogConfig,
};
