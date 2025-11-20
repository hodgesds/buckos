//! Service definition types and states for the init system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Health check configuration for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Command to run for health check
    pub exec: String,
    /// Interval between health checks
    #[serde(default = "default_health_interval")]
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    /// Timeout for health check command
    #[serde(default = "default_health_timeout")]
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    #[serde(default = "default_health_retries")]
    pub retries: u32,
    /// Initial delay before starting health checks
    #[serde(default = "default_health_start_period")]
    #[serde(with = "humantime_serde")]
    pub start_period: Duration,
}

fn default_health_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_health_timeout() -> Duration {
    Duration::from_secs(10)
}

fn default_health_retries() -> u32 {
    3
}

fn default_health_start_period() -> Duration {
    Duration::from_secs(0)
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            exec: String::new(),
            interval: default_health_interval(),
            timeout: default_health_timeout(),
            retries: default_health_retries(),
            start_period: default_health_start_period(),
        }
    }
}

/// Resource limits for a service.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceLimits {
    /// Memory limit in bytes (soft limit)
    pub memory_soft: Option<u64>,
    /// Memory limit in bytes (hard limit)
    pub memory_hard: Option<u64>,
    /// CPU quota (percentage, e.g., 50 for 50%)
    pub cpu_percent: Option<u32>,
    /// Maximum number of open file descriptors
    pub nofile: Option<u64>,
    /// Maximum number of processes
    pub nproc: Option<u64>,
    /// Maximum file size in bytes
    pub fsize: Option<u64>,
    /// Maximum core file size in bytes
    pub core: Option<u64>,
    /// Maximum stack size in bytes
    pub stack: Option<u64>,
    /// Maximum data segment size in bytes
    pub data: Option<u64>,
    /// Maximum locked memory in bytes
    pub memlock: Option<u64>,
    /// Maximum CPU time in seconds
    pub cpu_time: Option<u64>,
}

/// Socket configuration for socket activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketConfig {
    /// Socket type: stream, dgram, seqpacket
    #[serde(default = "default_socket_type")]
    pub socket_type: String,
    /// Listen address (e.g., "127.0.0.1:8080" or "/run/myservice.sock")
    pub listen: String,
    /// Accept connections (pass socket to service)
    #[serde(default)]
    pub accept: bool,
    /// Maximum connections in backlog
    #[serde(default = "default_socket_backlog")]
    pub backlog: u32,
    /// File permissions for Unix sockets
    pub socket_mode: Option<u32>,
    /// User for Unix sockets
    pub socket_user: Option<String>,
    /// Group for Unix sockets
    pub socket_group: Option<String>,
}

fn default_socket_type() -> String {
    "stream".to_string()
}

fn default_socket_backlog() -> u32 {
    128
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            socket_type: default_socket_type(),
            listen: String::new(),
            accept: false,
            backlog: default_socket_backlog(),
            socket_mode: None,
            socket_user: None,
            socket_group: None,
        }
    }
}

/// Timer configuration for scheduled service execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    /// Calendar expression (cron-like): "daily", "weekly", "Mon *-*-* 00:00:00"
    pub on_calendar: Option<String>,
    /// Time after boot to trigger
    #[serde(default)]
    #[serde(with = "option_humantime_serde")]
    pub on_boot: Option<Duration>,
    /// Time after last activation to trigger
    #[serde(default)]
    #[serde(with = "option_humantime_serde")]
    pub on_unit_active: Option<Duration>,
    /// Time after unit became inactive to trigger
    #[serde(default)]
    #[serde(with = "option_humantime_serde")]
    pub on_unit_inactive: Option<Duration>,
    /// Whether timer is persistent (triggers missed runs on startup)
    #[serde(default)]
    pub persistent: bool,
    /// Accuracy/randomization window
    #[serde(default = "default_timer_accuracy")]
    #[serde(with = "humantime_serde")]
    pub accuracy: Duration,
}

fn default_timer_accuracy() -> Duration {
    Duration::from_secs(60)
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            on_calendar: None,
            on_boot: None,
            on_unit_active: None,
            on_unit_inactive: None,
            persistent: false,
            accuracy: default_timer_accuracy(),
        }
    }
}

/// Watchdog configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    /// Watchdog timeout - service must ping within this interval
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Action on timeout: restart, kill, none
    #[serde(default = "default_watchdog_action")]
    pub action: String,
}

fn default_watchdog_action() -> String {
    "restart".to_string()
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            action: default_watchdog_action(),
        }
    }
}

/// Module for optional duration humantime serialization.
mod option_humantime_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => serializer.serialize_some(&d.as_secs()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<u64> = Option::deserialize(deserializer)?;
        Ok(opt.map(Duration::from_secs))
    }
}

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
    /// Health check configuration
    #[serde(default)]
    pub health_check: Option<HealthCheck>,
    /// Resource limits
    #[serde(default)]
    pub resource_limits: Option<ResourceLimits>,
    /// Socket activation configuration
    #[serde(default)]
    pub sockets: Vec<SocketConfig>,
    /// Timer configuration for scheduled execution
    #[serde(default)]
    pub timer: Option<TimerConfig>,
    /// Watchdog configuration
    #[serde(default)]
    pub watchdog: Option<WatchdogConfig>,
    /// Whether this service is a template (name contains @)
    #[serde(default)]
    pub template: bool,
    /// Standard output handling: inherit, null, journal, file:/path
    #[serde(default = "default_stdout")]
    pub standard_output: String,
    /// Standard error handling: inherit, null, journal, file:/path
    #[serde(default = "default_stderr")]
    pub standard_error: String,
}

fn default_stdout() -> String {
    "journal".to_string()
}

fn default_stderr() -> String {
    "journal".to_string()
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
            health_check: None,
            resource_limits: None,
            sockets: Vec::new(),
            timer: None,
            watchdog: None,
            template: false,
            standard_output: default_stdout(),
            standard_error: default_stderr(),
        }
    }

    /// Check if this is a template service.
    pub fn is_template(&self) -> bool {
        self.template || self.name.contains('@')
    }

    /// Create an instance from a template with the given instance name.
    pub fn instantiate(&self, instance: &str) -> Self {
        let mut def = self.clone();
        def.name = self.name.replace('@', &format!("@{}", instance));
        def.template = false;
        // Replace %i in commands with instance name
        def.exec_start = def.exec_start.replace("%i", instance);
        if let Some(ref mut cmd) = def.exec_stop {
            *cmd = cmd.replace("%i", instance);
        }
        if let Some(ref mut cmd) = def.exec_reload {
            *cmd = cmd.replace("%i", instance);
        }
        def
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

/// Health status for a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// No health check configured
    None,
    /// Health check starting (in start_period)
    Starting,
    /// Service is healthy
    Healthy,
    /// Service is unhealthy
    Unhealthy,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::None
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::None => write!(f, "none"),
            HealthStatus::Starting => write!(f, "starting"),
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
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
    /// Timestamps of recent restarts (for rate limiting)
    pub restart_timestamps: Vec<DateTime<Utc>>,
    /// Last failure reason
    pub failure_reason: Option<String>,
    /// Health status
    pub health_status: HealthStatus,
    /// Number of consecutive health check failures
    pub health_failures: u32,
    /// Last health check time
    pub last_health_check: Option<DateTime<Utc>>,
    /// Last watchdog ping time
    pub last_watchdog_ping: Option<DateTime<Utc>>,
    /// Whether the service is masked
    pub masked: bool,
    /// Boot time for this service (for analyze)
    pub boot_duration_ms: Option<u64>,
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
            restart_timestamps: Vec::new(),
            failure_reason: None,
            health_status: HealthStatus::None,
            health_failures: 0,
            last_health_check: None,
            last_watchdog_ping: None,
            masked: false,
            boot_duration_ms: None,
        }
    }

    /// Check if the service can restart based on rate limiting.
    ///
    /// Returns true if restart is allowed, false if rate limited.
    /// Uses a sliding window of 5 restarts in 10 seconds.
    pub fn can_restart(&mut self) -> bool {
        let now = Utc::now();
        let window = chrono::Duration::seconds(10);
        let max_restarts = 5;

        // Clean up old timestamps outside the window
        self.restart_timestamps
            .retain(|ts| now.signed_duration_since(*ts) < window);

        // Check if we're within the limit
        if self.restart_timestamps.len() >= max_restarts {
            false
        } else {
            // Record this restart attempt
            self.restart_timestamps.push(now);
            true
        }
    }

    /// Reset restart rate limiting (e.g., after successful long run).
    pub fn reset_restart_rate(&mut self) {
        self.restart_timestamps.clear();
        self.restart_count = 0;
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
    /// Health status
    pub health_status: HealthStatus,
    /// Whether the service is masked
    pub masked: bool,
    /// Boot duration in milliseconds
    pub boot_duration_ms: Option<u64>,
    /// Whether the service is enabled
    pub enabled: bool,
    /// Dependencies (requires)
    pub requires: Vec<String>,
    /// Soft dependencies (wants)
    pub wants: Vec<String>,
}

impl ServiceStatus {
    /// Create status from definition and instance.
    pub fn from_service(def: &ServiceDefinition, instance: &ServiceInstance) -> Self {
        let (memory_bytes, cpu_percent) = if let Some(pid) = instance.main_pid {
            (get_process_memory(pid), get_process_cpu(pid))
        } else {
            (None, None)
        };

        Self {
            name: def.name.clone(),
            state: instance.state,
            description: def.description.clone(),
            main_pid: instance.main_pid,
            memory_bytes,
            cpu_percent,
            uptime_secs: instance.uptime().map(|d| d.as_secs()),
            restart_count: instance.restart_count,
            health_status: instance.health_status,
            masked: instance.masked,
            boot_duration_ms: instance.boot_duration_ms,
            enabled: def.enabled,
            requires: def.requires.clone(),
            wants: def.wants.clone(),
        }
    }
}

/// Get memory usage for a process from /proc/{pid}/statm
fn get_process_memory(pid: u32) -> Option<u64> {
    let statm_path = format!("/proc/{}/statm", pid);
    let content = std::fs::read_to_string(&statm_path).ok()?;
    let parts: Vec<&str> = content.split_whitespace().collect();

    if parts.len() >= 2 {
        // Second field is resident set size in pages
        let rss_pages: u64 = parts[1].parse().ok()?;
        // Page size is typically 4096 bytes
        let page_size = 4096u64;
        Some(rss_pages * page_size)
    } else {
        None
    }
}

/// Get CPU usage for a process from /proc/{pid}/stat
fn get_process_cpu(pid: u32) -> Option<f64> {
    let stat_path = format!("/proc/{}/stat", pid);
    let content = std::fs::read_to_string(&stat_path).ok()?;

    // Parse the stat file - fields are space-separated but comm field can contain spaces
    // Format: pid (comm) state ppid pgrp session tty_nr tpgid flags minflt cminflt majflt cmajflt utime stime ...
    let comm_end = content.rfind(')')?;
    let after_comm = &content[comm_end + 2..];
    let parts: Vec<&str> = after_comm.split_whitespace().collect();

    if parts.len() >= 13 {
        // utime is at index 11, stime is at index 12 (0-indexed after comm)
        let utime: u64 = parts[11].parse().ok()?;
        let stime: u64 = parts[12].parse().ok()?;
        let total_time = utime + stime;

        // Get system uptime
        let uptime_content = std::fs::read_to_string("/proc/uptime").ok()?;
        let uptime: f64 = uptime_content.split_whitespace().next()?.parse().ok()?;

        // Get process start time (index 19 after comm)
        let starttime: u64 = parts[19].parse().ok()?;

        // Get clock ticks per second (typically 100)
        let hertz = 100.0f64;

        // Calculate process age in seconds
        let process_age = uptime - (starttime as f64 / hertz);

        if process_age > 0.0 {
            // CPU percentage = (total_time / hertz) / process_age * 100
            let cpu_percent = (total_time as f64 / hertz) / process_age * 100.0;
            Some(cpu_percent)
        } else {
            Some(0.0)
        }
    } else {
        None
    }
}
