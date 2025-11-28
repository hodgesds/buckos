//! Control socket for communicating with running init process.
//!
//! This module provides IPC communication between the boss CLI tool
//! and the running init process via a Unix domain socket.

use crate::error::{Error, Result};
use crate::ShutdownType;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, info, warn};

/// Default path for the control socket
pub const DEFAULT_CONTROL_SOCKET: &str = "/run/boss/control.sock";

/// Commands that can be sent to the init process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlCommand {
    /// Start a service
    StartService { name: String },
    /// Stop a service
    StopService { name: String },
    /// Restart a service
    RestartService { name: String },
    /// Reload a service configuration
    ReloadService { name: String },
    /// Enable a service for auto-start
    EnableService { name: String },
    /// Disable a service from auto-start
    DisableService { name: String },
    /// Get status of a specific service
    GetServiceStatus { name: String },
    /// Get status of all services
    GetAllStatus,
    /// List all services
    ListServices,
    /// Initiate system shutdown
    Shutdown { shutdown_type: ShutdownType },
    /// Reload service definitions
    ReloadDaemon,
    /// Ping to check if init is responding
    Ping,
}

/// Response from the init process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlResponse {
    /// Command succeeded
    Success { message: String },
    /// Command failed
    Error { message: String },
    /// Service status response
    ServiceStatus {
        name: String,
        state: String,
        pid: Option<u32>,
        uptime_secs: Option<u64>,
    },
    /// List of services
    ServiceList { services: Vec<ServiceInfo> },
    /// Pong response
    Pong,
}

/// Basic service information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub state: String,
    pub enabled: bool,
    pub description: Option<String>,
}

/// Control socket server (runs in init process)
pub struct ControlServer {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
}

impl ControlServer {
    /// Create a new control server
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            listener: None,
        }
    }

    /// Create with default socket path
    pub fn with_default_path() -> Self {
        Self::new(DEFAULT_CONTROL_SOCKET)
    }

    /// Start listening for connections
    pub async fn start(&mut self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Remove existing socket file
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        info!(path = %self.socket_path.display(), "Control socket listening");

        // Set socket permissions (allow user and group)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o660);
            std::fs::set_permissions(&self.socket_path, permissions)?;
        }

        self.listener = Some(listener);
        Ok(())
    }

    /// Accept and handle a single connection
    pub async fn accept(&self) -> Result<UnixStream> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| Error::Other("Control server not started".to_string()))?;

        let (stream, _addr) = listener.accept().await?;
        debug!("Accepted control connection");
        Ok(stream)
    }

    /// Read a command from a stream
    pub async fn read_command(stream: &mut UnixStream) -> Result<ControlCommand> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let command: ControlCommand = serde_json::from_str(line.trim())
            .map_err(|e| Error::Other(format!("Failed to parse command: {}", e)))?;

        debug!(command = ?command, "Received control command");
        Ok(command)
    }

    /// Write a response to a stream
    pub async fn write_response(stream: &mut UnixStream, response: &ControlResponse) -> Result<()> {
        let json = serde_json::to_string(response)
            .map_err(|e| Error::Other(format!("Failed to serialize response: {}", e)))?;

        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        debug!(response = ?response, "Sent control response");
        Ok(())
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for ControlServer {
    fn drop(&mut self) {
        // Clean up socket file
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                warn!(error = %e, "Failed to remove control socket");
            }
        }
    }
}

/// Control socket client (used by boss CLI)
pub struct ControlClient {
    socket_path: PathBuf,
}

impl ControlClient {
    /// Create a new control client
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Create with default socket path
    pub fn with_default_path() -> Self {
        Self::new(DEFAULT_CONTROL_SOCKET)
    }

    /// Check if the control socket exists
    pub fn is_available(&self) -> bool {
        self.socket_path.exists()
    }

    /// Connect to the init process
    pub async fn connect(&self) -> Result<UnixStream> {
        if !self.socket_path.exists() {
            return Err(Error::Other(format!(
                "Control socket not found at {}. Is the init process running?",
                self.socket_path.display()
            )));
        }

        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            Error::Other(format!(
                "Failed to connect to control socket: {}. Is the init process running?",
                e
            ))
        })?;

        debug!("Connected to control socket");
        Ok(stream)
    }

    /// Send a command and receive a response
    pub async fn send_command(&self, command: ControlCommand) -> Result<ControlResponse> {
        let mut stream = self.connect().await?;

        // Send command
        let json = serde_json::to_string(&command)
            .map_err(|e| Error::Other(format!("Failed to serialize command: {}", e)))?;

        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Read response
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: ControlResponse = serde_json::from_str(line.trim())
            .map_err(|e| Error::Other(format!("Failed to parse response: {}", e)))?;

        Ok(response)
    }

    /// Convenience methods for common commands
    pub async fn start_service(&self, name: &str) -> Result<ControlResponse> {
        self.send_command(ControlCommand::StartService {
            name: name.to_string(),
        })
        .await
    }

    pub async fn stop_service(&self, name: &str) -> Result<ControlResponse> {
        self.send_command(ControlCommand::StopService {
            name: name.to_string(),
        })
        .await
    }

    pub async fn restart_service(&self, name: &str) -> Result<ControlResponse> {
        self.send_command(ControlCommand::RestartService {
            name: name.to_string(),
        })
        .await
    }

    pub async fn get_service_status(&self, name: &str) -> Result<ControlResponse> {
        self.send_command(ControlCommand::GetServiceStatus {
            name: name.to_string(),
        })
        .await
    }

    pub async fn list_services(&self) -> Result<ControlResponse> {
        self.send_command(ControlCommand::ListServices).await
    }

    pub async fn shutdown(&self, shutdown_type: ShutdownType) -> Result<ControlResponse> {
        self.send_command(ControlCommand::Shutdown { shutdown_type })
            .await
    }

    pub async fn ping(&self) -> Result<bool> {
        match self.send_command(ControlCommand::Ping).await {
            Ok(ControlResponse::Pong) => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Ok(false),
        }
    }
}

/// Helper to determine if we should use local init or connect to running init
pub fn should_use_control_socket() -> bool {
    // Use control socket if:
    // 1. We're not PID 1
    // 2. Control socket exists
    std::process::id() != 1 && Path::new(DEFAULT_CONTROL_SOCKET).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let cmd = ControlCommand::StartService {
            name: "nginx".to_string(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: ControlCommand = serde_json::from_str(&json).unwrap();

        match parsed {
            ControlCommand::StartService { name } => assert_eq!(name, "nginx"),
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let resp = ControlResponse::Success {
            message: "Service started".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: ControlResponse = serde_json::from_str(&json).unwrap();

        match parsed {
            ControlResponse::Success { message } => assert_eq!(message, "Service started"),
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_service_info_serialization() {
        let info = ServiceInfo {
            name: "nginx".to_string(),
            state: "running".to_string(),
            enabled: true,
            description: Some("Web server".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ServiceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "nginx");
        assert_eq!(parsed.state, "running");
        assert!(parsed.enabled);
    }
}
