//! Sandbox support for isolated builds
//!
//! Provides filesystem and network isolation for package builds,
//! similar to Portage's FEATURES="sandbox".

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable filesystem sandboxing
    pub filesystem: bool,
    /// Enable network sandboxing
    pub network: bool,
    /// Enable user namespace sandboxing
    pub userns: bool,
    /// Paths that are allowed to be read
    pub read_paths: Vec<PathBuf>,
    /// Paths that are allowed to be written
    pub write_paths: Vec<PathBuf>,
    /// Paths that are denied access
    pub deny_paths: Vec<PathBuf>,
    /// Network access rules
    pub network_rules: NetworkRules,
    /// Environment variables to preserve
    pub preserve_env: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            filesystem: true,
            network: false,
            userns: true,
            read_paths: vec![PathBuf::from("/")],
            write_paths: vec![
                PathBuf::from("/var/tmp/portage"),
                PathBuf::from("/var/cache/distfiles"),
                PathBuf::from("/var/db/repos"),
            ],
            deny_paths: vec![
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/etc/shadow"),
                PathBuf::from("/etc/sudoers"),
                PathBuf::from("/root"),
            ],
            network_rules: NetworkRules::default(),
            preserve_env: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "SHELL".to_string(),
                "TERM".to_string(),
                "LANG".to_string(),
                "LC_ALL".to_string(),
            ],
        }
    }
}

/// Network access rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRules {
    /// Allow all network access
    pub allow_all: bool,
    /// Allow local network only
    pub allow_local: bool,
    /// Specific hosts that are allowed
    pub allowed_hosts: HashSet<String>,
    /// Specific ports that are allowed
    pub allowed_ports: HashSet<u16>,
}

impl Default for NetworkRules {
    fn default() -> Self {
        Self {
            allow_all: false,
            allow_local: true,
            allowed_hosts: HashSet::new(),
            allowed_ports: HashSet::new(),
        }
    }
}

/// A sandbox violation
#[derive(Debug, Clone)]
pub struct SandboxViolation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// Path or resource involved
    pub path: String,
    /// Operation attempted
    pub operation: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of sandbox violation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    /// Attempted to read a denied path
    ReadDenied,
    /// Attempted to write outside allowed paths
    WriteDenied,
    /// Attempted network access when denied
    NetworkDenied,
    /// Attempted to execute a denied binary
    ExecDenied,
}

/// Sandbox execution context
pub struct Sandbox {
    /// Configuration
    config: SandboxConfig,
    /// Working directory
    workdir: PathBuf,
    /// Recorded violations
    violations: Vec<SandboxViolation>,
}

impl Sandbox {
    /// Create a new sandbox with default configuration
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    /// Create a sandbox with custom configuration
    pub fn with_config(config: SandboxConfig) -> Self {
        Self {
            config,
            workdir: std::env::temp_dir(),
            violations: Vec::new(),
        }
    }

    /// Set the working directory
    pub fn set_workdir(&mut self, path: PathBuf) {
        self.workdir = path;
    }

    /// Add a path to the write allowlist
    pub fn allow_write(&mut self, path: PathBuf) {
        self.config.write_paths.push(path);
    }

    /// Add a path to the deny list
    pub fn deny_path(&mut self, path: PathBuf) {
        self.config.deny_paths.push(path);
    }

    /// Execute a command in the sandbox
    pub fn execute(&mut self, command: &str, args: &[&str]) -> Result<SandboxResult> {
        let start_time = std::time::Instant::now();

        // Build the sandboxed command
        let mut cmd = self.build_sandboxed_command(command, args)?;

        // Execute
        let output = cmd
            .output()
            .map_err(|e| Error::SandboxError(format!("Failed to execute command: {}", e)))?;

        let duration = start_time.elapsed();

        Ok(SandboxResult {
            success: output.status.success(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
            violations: self.violations.clone(),
        })
    }

    /// Build the sandboxed command using available isolation tools
    fn build_sandboxed_command(&self, command: &str, args: &[&str]) -> Result<Command> {
        // Try different sandboxing methods in order of preference
        if self.is_available("bwrap") {
            self.build_bwrap_command(command, args)
        } else if self.is_available("unshare") && self.config.userns {
            self.build_unshare_command(command, args)
        } else if self.is_available("sandbox") {
            self.build_portage_sandbox_command(command, args)
        } else {
            // Fall back to unsandboxed execution with a warning
            tracing::warn!("No sandbox tool available, running unsandboxed");
            let mut cmd = Command::new(command);
            cmd.args(args);
            cmd.current_dir(&self.workdir);
            Ok(cmd)
        }
    }

    /// Check if a command is available
    fn is_available(&self, cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Build command using bubblewrap (bwrap)
    fn build_bwrap_command(&self, command: &str, args: &[&str]) -> Result<Command> {
        let mut cmd = Command::new("bwrap");

        // Basic isolation
        cmd.arg("--unshare-all");
        cmd.arg("--die-with-parent");

        // Allow /proc and /dev
        cmd.args(["--proc", "/proc"]);
        cmd.args(["--dev", "/dev"]);

        // Read-only bindings for system paths
        for path in &self.config.read_paths {
            if path.exists() {
                cmd.args(["--ro-bind", path.to_str().unwrap(), path.to_str().unwrap()]);
            }
        }

        // Writable bindings
        for path in &self.config.write_paths {
            if path.exists() {
                cmd.args(["--bind", path.to_str().unwrap(), path.to_str().unwrap()]);
            }
        }

        // Working directory
        cmd.args(["--chdir", self.workdir.to_str().unwrap()]);

        // Network isolation
        if !self.config.network_rules.allow_all {
            cmd.arg("--unshare-net");
        }

        // Preserve environment
        for var in &self.config.preserve_env {
            if let Ok(val) = std::env::var(var) {
                cmd.args(["--setenv", var, &val]);
            }
        }

        // The actual command
        cmd.arg("--").arg(command).args(args);

        Ok(cmd)
    }

    /// Build command using unshare
    fn build_unshare_command(&self, command: &str, args: &[&str]) -> Result<Command> {
        let mut cmd = Command::new("unshare");

        // User namespace for rootless operation
        cmd.arg("--user");
        cmd.arg("--map-root-user");

        // Mount namespace
        cmd.arg("--mount");

        // Network namespace if needed
        if !self.config.network_rules.allow_all {
            cmd.arg("--net");
        }

        // PID namespace
        cmd.arg("--pid");
        cmd.arg("--fork");

        // The actual command
        cmd.arg("--").arg(command).args(args);

        cmd.current_dir(&self.workdir);

        Ok(cmd)
    }

    /// Build command using Portage's sandbox
    fn build_portage_sandbox_command(&self, command: &str, args: &[&str]) -> Result<Command> {
        let mut cmd = Command::new("sandbox");

        // Set sandbox variables
        let write_paths: Vec<_> = self
            .config
            .write_paths
            .iter()
            .filter_map(|p| p.to_str())
            .collect();
        let deny_paths: Vec<_> = self
            .config
            .deny_paths
            .iter()
            .filter_map(|p| p.to_str())
            .collect();

        cmd.env("SANDBOX_WRITE", write_paths.join(":"));
        cmd.env("SANDBOX_DENY", deny_paths.join(":"));

        if !self.config.network_rules.allow_all {
            cmd.env("SANDBOX_NET", "");
        }

        cmd.arg(command).args(args);
        cmd.current_dir(&self.workdir);

        Ok(cmd)
    }

    /// Check if a path access would be allowed
    pub fn check_access(&self, path: &Path, write: bool) -> bool {
        // Check deny list first
        for denied in &self.config.deny_paths {
            if path.starts_with(denied) {
                return false;
            }
        }

        if write {
            // Check write allowlist
            for allowed in &self.config.write_paths {
                if path.starts_with(allowed) {
                    return true;
                }
            }
            false
        } else {
            // Check read allowlist
            for allowed in &self.config.read_paths {
                if path.starts_with(allowed) {
                    return true;
                }
            }
            false
        }
    }

    /// Record a violation
    pub fn record_violation(&mut self, violation_type: ViolationType, path: &str, operation: &str) {
        self.violations.push(SandboxViolation {
            violation_type,
            path: path.to_string(),
            operation: operation.to_string(),
            timestamp: chrono::Utc::now(),
        });
    }

    /// Get recorded violations
    pub fn get_violations(&self) -> &[SandboxViolation] {
        &self.violations
    }

    /// Clear recorded violations
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone)]
pub struct SandboxResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    pub duration: std::time::Duration,
    /// Any sandbox violations
    pub violations: Vec<SandboxViolation>,
}

/// Builder for creating sandbox configurations
pub struct SandboxBuilder {
    config: SandboxConfig,
}

impl SandboxBuilder {
    /// Create a new sandbox builder
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }

    /// Enable or disable filesystem sandboxing
    pub fn filesystem(mut self, enabled: bool) -> Self {
        self.config.filesystem = enabled;
        self
    }

    /// Enable or disable network sandboxing
    pub fn network(mut self, enabled: bool) -> Self {
        self.config.network = enabled;
        self
    }

    /// Add a writable path
    pub fn allow_write(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.write_paths.push(path.into());
        self
    }

    /// Add a denied path
    pub fn deny(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.deny_paths.push(path.into());
        self
    }

    /// Allow network access to specific host
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.config.network_rules.allowed_hosts.insert(host.into());
        self
    }

    /// Preserve an environment variable
    pub fn preserve_env(mut self, var: impl Into<String>) -> Self {
        self.config.preserve_env.push(var.into());
        self
    }

    /// Build the sandbox
    pub fn build(self) -> Sandbox {
        Sandbox::with_config(self.config)
    }
}

impl Default for SandboxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(config.filesystem);
        assert!(!config.network);
    }

    #[test]
    fn test_check_access() {
        let sandbox = Sandbox::new();

        // Should allow reading from root
        assert!(sandbox.check_access(Path::new("/usr/bin/ls"), false));

        // Should deny writing to protected paths
        assert!(!sandbox.check_access(Path::new("/etc/passwd"), true));
    }

    #[test]
    fn test_sandbox_builder() {
        let sandbox = SandboxBuilder::new()
            .filesystem(true)
            .network(false)
            .allow_write("/tmp/build")
            .build();

        assert!(sandbox.config.filesystem);
        assert!(!sandbox.config.network);
    }
}
