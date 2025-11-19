//! Sideros init system - PID 1 service manager.
//!
//! This crate provides a systemd-like init system for managing services
//! on a Linux system. It is designed to run as PID 1 and handles:
//!
//! - Service lifecycle management (start, stop, restart)
//! - Process supervision and automatic restart
//! - Service dependencies
//! - Signal handling (SIGCHLD, SIGTERM, SIGINT)
//! - Zombie process reaping
//! - Virtual filesystem mounting
//!
//! # Architecture
//!
//! The init system is composed of several components:
//!
//! - **Init**: The main init system that coordinates everything
//! - **ServiceManager**: Manages service definitions and instances
//! - **ProcessSupervisor**: Handles process spawning and supervision
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
pub mod manager;
pub mod process;
pub mod service;

// Re-export main types
pub use error::{Error, Result};
pub use init::{create_test_init, Init, InitConfig, ShutdownType};
pub use manager::ServiceManager;
pub use process::{ExitStatus, ProcessSupervisor};
pub use service::{
    RestartPolicy, ServiceDefinition, ServiceInstance, ServiceState, ServiceStatus, ServiceType,
};
