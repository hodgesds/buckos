//! Init system core - PID 1 duties and signal handling.

use crate::error::{Error, Result};
use crate::manager::ServiceManager;
use nix::mount::{mount, MsFlags};
use nix::sys::reboot::{reboot, RebootMode};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Init system configuration.
#[derive(Debug, Clone)]
pub struct InitConfig {
    /// Directory containing service definitions
    pub services_dir: PathBuf,
    /// Whether to mount virtual filesystems
    pub mount_filesystems: bool,
    /// Whether to enforce PID 1 requirement
    pub require_pid1: bool,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            services_dir: PathBuf::from("/etc/buckos/services"),
            mount_filesystems: true,
            require_pid1: true,
        }
    }
}

/// The main init system.
pub struct Init {
    /// Configuration
    config: InitConfig,
    /// Service manager
    manager: Arc<ServiceManager>,
    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<ShutdownType>,
}

/// Type of shutdown to perform.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ShutdownType {
    /// Power off the system
    PowerOff,
    /// Reboot the system
    Reboot,
    /// Halt the system
    Halt,
}

impl Init {
    /// Create a new init system.
    pub fn new(config: InitConfig) -> Result<Self> {
        // Check if we're PID 1
        let pid = std::process::id();
        if config.require_pid1 && pid != 1 {
            return Err(Error::NotPid1(pid));
        }

        let manager = Arc::new(ServiceManager::new(config.services_dir.clone()));
        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(Self {
            config,
            manager,
            shutdown_tx,
        })
    }

    /// Run the init system.
    pub async fn run(&self) -> Result<()> {
        info!("Buckos init system starting");

        // Mount virtual filesystems if configured
        if self.config.mount_filesystems {
            self.mount_filesystems()?;
        }

        // Load service definitions
        self.manager.load_services().await?;

        // Start enabled services in parallel for faster boot
        self.manager.start_enabled_services_parallel().await?;

        // Run the main event loop
        self.event_loop().await?;

        Ok(())
    }

    /// Mount virtual filesystems (/proc, /sys, /dev, etc.)
    fn mount_filesystems(&self) -> Result<()> {
        info!("Mounting virtual filesystems");

        // Mount /proc
        if let Err(e) = self.mount_fs("proc", "/proc", "proc", MsFlags::empty()) {
            warn!(error = %e, "Failed to mount /proc");
        }

        // Mount /sys
        if let Err(e) = self.mount_fs("sysfs", "/sys", "sysfs", MsFlags::empty()) {
            warn!(error = %e, "Failed to mount /sys");
        }

        // Mount /dev (devtmpfs)
        if let Err(e) = self.mount_fs("devtmpfs", "/dev", "devtmpfs", MsFlags::empty()) {
            warn!(error = %e, "Failed to mount /dev");
        }

        // Mount /dev/pts
        if let Err(e) = self.mount_fs("devpts", "/dev/pts", "devpts", MsFlags::empty()) {
            warn!(error = %e, "Failed to mount /dev/pts");
        }

        // Mount /run (tmpfs)
        if let Err(e) = self.mount_fs("tmpfs", "/run", "tmpfs", MsFlags::empty()) {
            warn!(error = %e, "Failed to mount /run");
        }

        Ok(())
    }

    /// Mount a filesystem.
    fn mount_fs(&self, source: &str, target: &str, fstype: &str, flags: MsFlags) -> Result<()> {
        let target_path = std::path::Path::new(target);
        if !target_path.exists() {
            std::fs::create_dir_all(target_path)?;
        }

        mount(Some(source), target, Some(fstype), flags, None::<&str>)?;

        info!(
            source = source,
            target = target,
            fstype = fstype,
            "Mounted filesystem"
        );
        Ok(())
    }

    /// Main event loop for the init system.
    async fn event_loop(&self) -> Result<()> {
        // Set up signal handlers
        let mut sigchld = signal(SignalKind::child())?;
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigint = signal(SignalKind::interrupt())?;

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        info!("Init system ready, entering event loop");

        loop {
            tokio::select! {
                // Handle SIGCHLD - reap zombie processes
                _ = sigchld.recv() => {
                    self.handle_sigchld().await;
                }

                // Handle SIGTERM - initiate shutdown
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, initiating shutdown");
                    self.shutdown(ShutdownType::PowerOff).await?;
                    break;
                }

                // Handle SIGINT - initiate shutdown
                _ = sigint.recv() => {
                    info!("Received SIGINT, initiating shutdown");
                    self.shutdown(ShutdownType::PowerOff).await?;
                    break;
                }

                // Handle shutdown request
                shutdown_type = shutdown_rx.recv() => {
                    if let Ok(shutdown_type) = shutdown_type {
                        self.shutdown(shutdown_type).await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle SIGCHLD signal - reap zombies and notify service manager.
    async fn handle_sigchld(&self) {
        let supervisor = self.manager.supervisor();
        let statuses = supervisor.reap_zombies().await;

        for status in statuses {
            self.manager.handle_process_exit(status).await;
        }
    }

    /// Initiate system shutdown.
    async fn shutdown(&self, shutdown_type: ShutdownType) -> Result<()> {
        info!(shutdown_type = ?shutdown_type, "Initiating system shutdown");

        // Stop all services
        self.manager.stop_all_services().await?;

        // Sync filesystems
        unsafe {
            libc::sync();
        }

        // Perform the actual shutdown
        if self.config.require_pid1 {
            let mode = match shutdown_type {
                ShutdownType::PowerOff => RebootMode::RB_POWER_OFF,
                ShutdownType::Reboot => RebootMode::RB_AUTOBOOT,
                ShutdownType::Halt => RebootMode::RB_HALT_SYSTEM,
            };

            reboot(mode)?;
        }

        Ok(())
    }

    /// Request a shutdown.
    pub fn request_shutdown(&self, shutdown_type: ShutdownType) -> Result<()> {
        self.shutdown_tx
            .send(shutdown_type)
            .map_err(|_| Error::SignalError("Failed to send shutdown signal".to_string()))?;
        Ok(())
    }

    /// Get a reference to the service manager.
    pub fn manager(&self) -> Arc<ServiceManager> {
        Arc::clone(&self.manager)
    }
}

/// Create a minimal init system for testing or non-PID1 operation.
pub fn create_test_init(services_dir: PathBuf) -> Result<Init> {
    let config = InitConfig {
        services_dir,
        mount_filesystems: false,
        require_pid1: false,
    };
    Init::new(config)
}
