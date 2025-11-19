//! Process management for the init system.
//!
//! This module handles spawning, supervising, and reaping processes.

use crate::error::{Error, Result};
use crate::service::ServiceDefinition;
use nix::sys::signal::{self, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Information about a spawned process.
#[derive(Debug)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Child process handle
    pub child: Child,
    /// Service name this process belongs to
    pub service_name: String,
    /// Whether this is the main process
    pub is_main: bool,
}

/// Exit status of a process.
#[derive(Debug, Clone)]
pub struct ExitStatus {
    /// Process ID
    pub pid: u32,
    /// Exit code (if exited normally)
    pub code: Option<i32>,
    /// Signal (if killed by signal)
    pub signal: Option<i32>,
}

impl ExitStatus {
    /// Check if the process exited successfully.
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

/// Process supervisor that manages process lifecycle.
pub struct ProcessSupervisor {
    /// Map of PID to process info
    processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
}

impl ProcessSupervisor {
    /// Create a new process supervisor.
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Spawn a process for a service.
    pub async fn spawn(&self, service: &ServiceDefinition) -> Result<u32> {
        let parts: Vec<&str> = service.exec_start.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Error::ProcessSpawnFailed(
                "Empty exec_start command".to_string(),
            ));
        }

        let program = parts[0];
        let args = &parts[1..];

        let mut cmd = Command::new(program);
        cmd.args(args);

        // Set working directory if specified
        if let Some(ref dir) = service.working_directory {
            cmd.current_dir(dir);
        }

        // Set environment variables
        cmd.envs(&service.environment);

        // Clear environment and set basic vars
        cmd.env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");

        // Set user/group if specified
        if let Some(ref user) = service.user {
            if let Ok(uid) = user.parse::<u32>() {
                unsafe {
                    cmd.pre_exec(move || {
                        nix::unistd::setuid(nix::unistd::Uid::from_raw(uid))
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                        Ok(())
                    });
                }
            }
        }

        // Create new session for the process
        unsafe {
            cmd.pre_exec(|| {
                nix::unistd::setsid()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                Ok(())
            });
        }

        // Redirect stdio
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::inherit()); // TODO: Log to journal
        cmd.stderr(Stdio::inherit());

        // Spawn the process
        let child = cmd.spawn().map_err(|e| {
            Error::ProcessSpawnFailed(format!("{}: {}", service.exec_start, e))
        })?;

        let pid = child.id();
        info!(service = %service.name, pid = pid, "Spawned process");

        // Track the process
        let process_info = ProcessInfo {
            pid,
            child,
            service_name: service.name.clone(),
            is_main: true,
        };

        self.processes.write().await.insert(pid, process_info);

        Ok(pid)
    }

    /// Send a signal to a process.
    pub async fn signal(&self, pid: u32, sig: Signal) -> Result<()> {
        let processes = self.processes.read().await;
        if !processes.contains_key(&pid) {
            return Err(Error::ProcessNotFound(pid));
        }

        signal::kill(Pid::from_raw(pid as i32), sig)?;
        debug!(pid = pid, signal = ?sig, "Sent signal to process");
        Ok(())
    }

    /// Stop a process gracefully.
    pub async fn stop(&self, pid: u32, timeout: std::time::Duration) -> Result<ExitStatus> {
        // Send SIGTERM first
        self.signal(pid, Signal::SIGTERM).await?;

        // Wait for process to exit
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = self.try_wait(pid).await? {
                return Ok(status);
            }

            if start.elapsed() > timeout {
                // Process didn't exit in time, send SIGKILL
                warn!(pid = pid, "Process didn't exit in time, sending SIGKILL");
                self.signal(pid, Signal::SIGKILL).await?;

                // Wait a bit more for SIGKILL
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                if let Some(status) = self.try_wait(pid).await? {
                    return Ok(status);
                }

                return Err(Error::ServiceStopFailed {
                    name: pid.to_string(),
                    reason: "Process didn't respond to SIGKILL".to_string(),
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    /// Try to reap a specific process without blocking.
    pub async fn try_wait(&self, pid: u32) -> Result<Option<ExitStatus>> {
        match waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, code)) => {
                self.processes.write().await.remove(&pid);
                Ok(Some(ExitStatus {
                    pid,
                    code: Some(code),
                    signal: None,
                }))
            }
            Ok(WaitStatus::Signaled(_, sig, _)) => {
                self.processes.write().await.remove(&pid);
                Ok(Some(ExitStatus {
                    pid,
                    code: None,
                    signal: Some(sig as i32),
                }))
            }
            Ok(WaitStatus::StillAlive) => Ok(None),
            Ok(_) => Ok(None),
            Err(nix::Error::ECHILD) => {
                // Process doesn't exist
                self.processes.write().await.remove(&pid);
                Ok(Some(ExitStatus {
                    pid,
                    code: None,
                    signal: None,
                }))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Reap any zombie processes (for PID 1 duty).
    pub async fn reap_zombies(&self) -> Vec<ExitStatus> {
        let mut statuses = Vec::new();

        loop {
            match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(pid, code)) => {
                    let pid = pid.as_raw() as u32;
                    self.processes.write().await.remove(&pid);
                    debug!(pid = pid, code = code, "Reaped zombie process");
                    statuses.push(ExitStatus {
                        pid,
                        code: Some(code),
                        signal: None,
                    });
                }
                Ok(WaitStatus::Signaled(pid, sig, _)) => {
                    let pid = pid.as_raw() as u32;
                    self.processes.write().await.remove(&pid);
                    debug!(pid = pid, signal = ?sig, "Reaped signaled process");
                    statuses.push(ExitStatus {
                        pid,
                        code: None,
                        signal: Some(sig as i32),
                    });
                }
                Ok(WaitStatus::StillAlive) | Err(nix::Error::ECHILD) => {
                    // No more zombies to reap
                    break;
                }
                Ok(_) => continue,
                Err(e) => {
                    error!(error = %e, "Error reaping zombies");
                    break;
                }
            }
        }

        statuses
    }

    /// Get the service name for a PID.
    pub async fn get_service_name(&self, pid: u32) -> Option<String> {
        self.processes.read().await.get(&pid).map(|p| p.service_name.clone())
    }

    /// Check if a process is running.
    pub async fn is_running(&self, pid: u32) -> bool {
        // Check if we're tracking it
        if !self.processes.read().await.contains_key(&pid) {
            return false;
        }

        // Check if the process actually exists
        match signal::kill(Pid::from_raw(pid as i32), None) {
            Ok(_) => true,
            Err(_) => {
                // Process doesn't exist, remove it from tracking
                self.processes.write().await.remove(&pid);
                false
            }
        }
    }

    /// Get all tracked PIDs.
    pub async fn get_pids(&self) -> Vec<u32> {
        self.processes.read().await.keys().copied().collect()
    }
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}
