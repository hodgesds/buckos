//! Systemd unit file loader.
//!
//! This module provides support for loading systemd .service unit files
//! and converting them to sideros ServiceDefinition format.
//!
//! # Supported Directives
//!
//! ## [Unit] Section
//! - Description
//! - Requires, Wants, Before, After
//!
//! ## [Service] Section
//! - Type (simple, forking, oneshot, notify, idle)
//! - ExecStart, ExecStop, ExecReload
//! - WorkingDirectory
//! - User, Group
//! - Environment, EnvironmentFile
//! - Restart, RestartSec
//! - TimeoutStartSec, TimeoutStopSec
//! - StandardOutput, StandardError
//! - WatchdogSec
//! - MemoryLimit, CPUQuota, LimitNOFILE, LimitNPROC
//!
//! ## [Install] Section
//! - WantedBy, RequiredBy (used to determine if enabled)

use crate::error::{Error, Result};
use crate::service::{
    HealthCheck, ResourceLimits, RestartPolicy, ServiceDefinition, ServiceType, SocketConfig,
    TimerConfig, WatchdogConfig,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Loader for systemd unit files.
pub struct SystemdLoader;

impl super::ServiceLoader for SystemdLoader {
    fn load(&self, path: &Path) -> Result<ServiceDefinition> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            Error::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;

        parse_unit_file(&content, path)
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "service"
    }

    fn name(&self) -> &'static str {
        "systemd"
    }
}

impl SystemdLoader {
    /// Create a new systemd loader.
    pub fn new() -> Self {
        Self
    }

    /// Convert a systemd unit file to TOML format.
    ///
    /// This is useful for migrating systemd services to sideros.
    pub fn convert_to_toml(path: &Path) -> Result<String> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            Error::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let def = parse_unit_file(&content, path)?;

        toml::to_string_pretty(&def).map_err(|e| {
            Error::ConfigError(format!("Failed to serialize to TOML: {}", e))
        })
    }

    /// Migrate a systemd unit file to sideros TOML format.
    ///
    /// Reads the .service file and writes a .toml file.
    pub fn migrate(source: &Path, dest: &Path) -> Result<()> {
        let toml_content = Self::convert_to_toml(source)?;
        std::fs::write(dest, toml_content).map_err(|e| {
            Error::ConfigError(format!("Failed to write {}: {}", dest.display(), e))
        })?;
        Ok(())
    }

    /// Migrate multiple systemd unit files from a directory.
    ///
    /// Converts all .service files in source_dir to .toml files in dest_dir.
    pub fn migrate_directory(source_dir: &Path, dest_dir: &Path) -> Result<Vec<PathBuf>> {
        if !source_dir.exists() {
            return Err(Error::ConfigError(format!(
                "Source directory does not exist: {}",
                source_dir.display()
            )));
        }

        std::fs::create_dir_all(dest_dir)?;

        let mut migrated = Vec::new();
        let entries = std::fs::read_dir(source_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("service") {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let dest_path = dest_dir.join(format!("{}.toml", stem));

                match Self::migrate(&path, &dest_path) {
                    Ok(()) => {
                        migrated.push(dest_path);
                    }
                    Err(e) => {
                        tracing::error!(path = ?path, error = %e, "Failed to migrate unit file");
                    }
                }
            }
        }

        Ok(migrated)
    }
}

impl Default for SystemdLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed sections from a systemd unit file.
#[derive(Debug, Default)]
struct UnitSections {
    unit: HashMap<String, String>,
    service: HashMap<String, String>,
    install: HashMap<String, String>,
    timer: HashMap<String, String>,
    socket: HashMap<String, String>,
}

/// Parse a systemd unit file content into sections.
fn parse_sections(content: &str) -> UnitSections {
    let mut sections = UnitSections::default();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Check for section header
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            continue;
        }

        // Parse key=value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();

            match current_section.as_str() {
                "Unit" => {
                    // Handle list values (append with space separator)
                    if let Some(existing) = sections.unit.get_mut(&key) {
                        existing.push(' ');
                        existing.push_str(&value);
                    } else {
                        sections.unit.insert(key, value);
                    }
                }
                "Service" => {
                    // Handle list values
                    if let Some(existing) = sections.service.get_mut(&key) {
                        existing.push(' ');
                        existing.push_str(&value);
                    } else {
                        sections.service.insert(key, value);
                    }
                }
                "Install" => {
                    if let Some(existing) = sections.install.get_mut(&key) {
                        existing.push(' ');
                        existing.push_str(&value);
                    } else {
                        sections.install.insert(key, value);
                    }
                }
                "Timer" => {
                    sections.timer.insert(key, value);
                }
                "Socket" => {
                    sections.socket.insert(key, value);
                }
                _ => {}
            }
        }
    }

    sections
}

/// Parse a systemd unit file into a ServiceDefinition.
fn parse_unit_file(content: &str, path: &Path) -> Result<ServiceDefinition> {
    let sections = parse_sections(content);

    // Extract service name from filename
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Get description from [Unit] section
    let description = sections
        .unit
        .get("Description")
        .cloned()
        .unwrap_or_default();

    // Parse service type
    let service_type = sections
        .service
        .get("Type")
        .map(|t| parse_service_type(t))
        .unwrap_or(ServiceType::Simple);

    // Get exec commands
    let exec_start = sections
        .service
        .get("ExecStart")
        .cloned()
        .unwrap_or_default();

    if exec_start.is_empty() {
        return Err(Error::ConfigError(format!(
            "Missing ExecStart in {}",
            path.display()
        )));
    }

    let exec_stop = sections.service.get("ExecStop").cloned();
    let exec_reload = sections.service.get("ExecReload").cloned();

    // Working directory
    let working_directory = sections
        .service
        .get("WorkingDirectory")
        .map(|s| PathBuf::from(s));

    // User and group
    let user = sections.service.get("User").cloned();
    let group = sections.service.get("Group").cloned();

    // Parse environment variables
    let mut environment = HashMap::new();
    if let Some(env_str) = sections.service.get("Environment") {
        for env in env_str.split_whitespace() {
            // Remove quotes if present
            let env = env.trim_matches('"').trim_matches('\'');
            if let Some((key, value)) = env.split_once('=') {
                environment.insert(key.to_string(), value.to_string());
            }
        }
    }

    // Parse dependencies
    let requires = parse_list(sections.unit.get("Requires"));
    let wants = parse_list(sections.unit.get("Wants"));
    let before = parse_list(sections.unit.get("Before"));
    let after = parse_list(sections.unit.get("After"));

    // Parse restart policy
    let restart = sections
        .service
        .get("Restart")
        .map(|r| parse_restart_policy(r))
        .unwrap_or(RestartPolicy::No);

    // Parse time values
    let restart_sec = sections
        .service
        .get("RestartSec")
        .and_then(|s| parse_duration(s))
        .unwrap_or(Duration::from_secs(1));

    let timeout_start_sec = sections
        .service
        .get("TimeoutStartSec")
        .and_then(|s| parse_duration(s))
        .unwrap_or(Duration::from_secs(30));

    let timeout_stop_sec = sections
        .service
        .get("TimeoutStopSec")
        .and_then(|s| parse_duration(s))
        .unwrap_or(Duration::from_secs(30));

    // Check if enabled (based on WantedBy/RequiredBy)
    let enabled = sections.install.get("WantedBy").is_some()
        || sections.install.get("RequiredBy").is_some();

    // Standard output/error
    let standard_output = sections
        .service
        .get("StandardOutput")
        .map(|s| normalize_stdio(s))
        .unwrap_or_else(|| "journal".to_string());

    let standard_error = sections
        .service
        .get("StandardError")
        .map(|s| normalize_stdio(s))
        .unwrap_or_else(|| "journal".to_string());

    // Parse resource limits
    let resource_limits = parse_resource_limits(&sections.service);

    // Parse watchdog config
    let watchdog = sections
        .service
        .get("WatchdogSec")
        .and_then(|s| parse_duration(s))
        .map(|timeout| WatchdogConfig {
            timeout,
            action: "restart".to_string(),
        });

    // Parse health check (from systemd notify or custom)
    let health_check = parse_health_check(&sections.service);

    // Parse timer configuration
    let timer = if !sections.timer.is_empty() {
        Some(parse_timer_config(&sections.timer))
    } else {
        None
    };

    // Parse socket configuration
    let sockets = if !sections.socket.is_empty() {
        parse_socket_config(&sections.socket)
    } else {
        Vec::new()
    };

    // Check if template
    let template = name.contains('@');

    Ok(ServiceDefinition {
        name,
        description,
        service_type,
        exec_start,
        exec_stop,
        exec_reload,
        working_directory,
        environment,
        user,
        group,
        requires,
        wants,
        before,
        after,
        restart,
        restart_sec,
        timeout_start_sec,
        timeout_stop_sec,
        enabled,
        health_check,
        resource_limits,
        sockets,
        timer,
        watchdog,
        template,
        standard_output,
        standard_error,
    })
}

/// Parse a service type string to ServiceType enum.
fn parse_service_type(s: &str) -> ServiceType {
    match s.to_lowercase().as_str() {
        "simple" => ServiceType::Simple,
        "forking" => ServiceType::Forking,
        "oneshot" => ServiceType::Oneshot,
        "notify" => ServiceType::Notify,
        "idle" => ServiceType::Idle,
        _ => ServiceType::Simple,
    }
}

/// Parse a restart policy string to RestartPolicy enum.
fn parse_restart_policy(s: &str) -> RestartPolicy {
    match s.to_lowercase().as_str() {
        "no" => RestartPolicy::No,
        "on-success" => RestartPolicy::OnSuccess,
        "on-failure" => RestartPolicy::OnFailure,
        "on-abnormal" => RestartPolicy::OnAbnormal,
        "always" => RestartPolicy::Always,
        _ => RestartPolicy::No,
    }
}

/// Parse a duration string (supports "30s", "5min", "1h", etc.)
fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();

    // Try to parse as plain number (seconds)
    if let Ok(secs) = s.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // Try to parse with suffix
    if let Some(num_str) = s.strip_suffix("ms") {
        if let Ok(ms) = num_str.trim().parse::<u64>() {
            return Some(Duration::from_millis(ms));
        }
    }
    if let Some(num_str) = s.strip_suffix('s') {
        if let Ok(secs) = num_str.trim().parse::<u64>() {
            return Some(Duration::from_secs(secs));
        }
    }
    if let Some(num_str) = s
        .strip_suffix("min")
        .or_else(|| s.strip_suffix('m'))
    {
        if let Ok(mins) = num_str.trim().parse::<u64>() {
            return Some(Duration::from_secs(mins * 60));
        }
    }
    if let Some(num_str) = s.strip_suffix('h') {
        if let Ok(hours) = num_str.trim().parse::<u64>() {
            return Some(Duration::from_secs(hours * 3600));
        }
    }

    None
}

/// Parse a space/comma separated list.
fn parse_list(s: Option<&String>) -> Vec<String> {
    s.map(|s| {
        s.split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    })
    .unwrap_or_default()
}

/// Normalize standard I/O type to sideros format.
fn normalize_stdio(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "inherit" | "tty" => "inherit".to_string(),
        "null" | "none" => "null".to_string(),
        "journal" | "syslog" | "kmsg" | "journal+console" => "journal".to_string(),
        s if s.starts_with("file:") => s.to_string(),
        _ => "journal".to_string(),
    }
}

/// Parse resource limits from the service section.
fn parse_resource_limits(service: &HashMap<String, String>) -> Option<ResourceLimits> {
    let mut limits = ResourceLimits::default();
    let mut has_limits = false;

    // Memory limits
    if let Some(mem) = service.get("MemoryLimit").or(service.get("MemoryMax")) {
        if let Some(bytes) = parse_memory_size(mem) {
            limits.memory_hard = Some(bytes);
            has_limits = true;
        }
    }

    if let Some(mem) = service.get("MemoryHigh") {
        if let Some(bytes) = parse_memory_size(mem) {
            limits.memory_soft = Some(bytes);
            has_limits = true;
        }
    }

    // CPU quota
    if let Some(cpu) = service.get("CPUQuota") {
        if let Some(percent_str) = cpu.strip_suffix('%') {
            if let Ok(percent) = percent_str.trim().parse::<u32>() {
                limits.cpu_percent = Some(percent);
                has_limits = true;
            }
        }
    }

    // File descriptor limit
    if let Some(nofile) = service.get("LimitNOFILE") {
        if let Ok(n) = nofile.parse::<u64>() {
            limits.nofile = Some(n);
            has_limits = true;
        }
    }

    // Process limit
    if let Some(nproc) = service.get("LimitNPROC") {
        if let Ok(n) = nproc.parse::<u64>() {
            limits.nproc = Some(n);
            has_limits = true;
        }
    }

    // File size limit
    if let Some(fsize) = service.get("LimitFSIZE") {
        if let Some(bytes) = parse_memory_size(fsize) {
            limits.fsize = Some(bytes);
            has_limits = true;
        }
    }

    // Core dump limit
    if let Some(core) = service.get("LimitCORE") {
        if let Some(bytes) = parse_memory_size(core) {
            limits.core = Some(bytes);
            has_limits = true;
        }
    }

    // Stack size
    if let Some(stack) = service.get("LimitSTACK") {
        if let Some(bytes) = parse_memory_size(stack) {
            limits.stack = Some(bytes);
            has_limits = true;
        }
    }

    // CPU time limit
    if let Some(cpu) = service.get("LimitCPU") {
        if let Some(duration) = parse_duration(cpu) {
            limits.cpu_time = Some(duration.as_secs());
            has_limits = true;
        }
    }

    if has_limits {
        Some(limits)
    } else {
        None
    }
}

/// Parse memory size strings (e.g., "512M", "1G", "1024K").
fn parse_memory_size(s: &str) -> Option<u64> {
    let s = s.trim();

    // Try plain number (bytes)
    if let Ok(bytes) = s.parse::<u64>() {
        return Some(bytes);
    }

    // Parse with suffix
    if let Some(num_str) = s.strip_suffix('K') {
        if let Ok(n) = num_str.trim().parse::<u64>() {
            return Some(n * 1024);
        }
    }
    if let Some(num_str) = s.strip_suffix('M') {
        if let Ok(n) = num_str.trim().parse::<u64>() {
            return Some(n * 1024 * 1024);
        }
    }
    if let Some(num_str) = s.strip_suffix('G') {
        if let Ok(n) = num_str.trim().parse::<u64>() {
            return Some(n * 1024 * 1024 * 1024);
        }
    }
    if let Some(num_str) = s.strip_suffix('T') {
        if let Ok(n) = num_str.trim().parse::<u64>() {
            return Some(n * 1024 * 1024 * 1024 * 1024);
        }
    }

    None
}

/// Parse health check configuration.
fn parse_health_check(_service: &HashMap<String, String>) -> Option<HealthCheck> {
    // systemd doesn't have direct health check support
    // This could be extended to parse custom X-HealthCheck* directives
    None
}

/// Parse timer configuration from [Timer] section.
fn parse_timer_config(timer: &HashMap<String, String>) -> TimerConfig {
    let on_calendar = timer.get("OnCalendar").cloned();
    let on_boot = timer
        .get("OnBootSec")
        .and_then(|s| parse_duration(s));
    let on_unit_active = timer
        .get("OnUnitActiveSec")
        .and_then(|s| parse_duration(s));
    let on_unit_inactive = timer
        .get("OnUnitInactiveSec")
        .and_then(|s| parse_duration(s));
    let persistent = timer
        .get("Persistent")
        .map(|s| s.to_lowercase() == "true" || s == "yes")
        .unwrap_or(false);
    let accuracy = timer
        .get("AccuracySec")
        .and_then(|s| parse_duration(s))
        .unwrap_or(Duration::from_secs(60));

    TimerConfig {
        on_calendar,
        on_boot,
        on_unit_active,
        on_unit_inactive,
        persistent,
        accuracy,
    }
}

/// Parse socket configuration from [Socket] section.
fn parse_socket_config(socket: &HashMap<String, String>) -> Vec<SocketConfig> {
    let mut configs = Vec::new();

    // Parse listen addresses
    let listen_stream = socket.get("ListenStream");
    let listen_dgram = socket.get("ListenDatagram");
    let listen_seq = socket.get("ListenSequentialPacket");

    if let Some(addr) = listen_stream {
        for listen in addr.split_whitespace() {
            configs.push(SocketConfig {
                socket_type: "stream".to_string(),
                listen: listen.to_string(),
                accept: socket
                    .get("Accept")
                    .map(|s| s.to_lowercase() == "true" || s == "yes")
                    .unwrap_or(false),
                backlog: socket
                    .get("Backlog")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(128),
                socket_mode: socket
                    .get("SocketMode")
                    .and_then(|s| u32::from_str_radix(s, 8).ok()),
                socket_user: socket.get("SocketUser").cloned(),
                socket_group: socket.get("SocketGroup").cloned(),
            });
        }
    }

    if let Some(addr) = listen_dgram {
        for listen in addr.split_whitespace() {
            configs.push(SocketConfig {
                socket_type: "dgram".to_string(),
                listen: listen.to_string(),
                accept: false,
                backlog: 128,
                socket_mode: socket
                    .get("SocketMode")
                    .and_then(|s| u32::from_str_radix(s, 8).ok()),
                socket_user: socket.get("SocketUser").cloned(),
                socket_group: socket.get("SocketGroup").cloned(),
            });
        }
    }

    if let Some(addr) = listen_seq {
        for listen in addr.split_whitespace() {
            configs.push(SocketConfig {
                socket_type: "seqpacket".to_string(),
                listen: listen.to_string(),
                accept: socket
                    .get("Accept")
                    .map(|s| s.to_lowercase() == "true" || s == "yes")
                    .unwrap_or(false),
                backlog: socket
                    .get("Backlog")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(128),
                socket_mode: socket
                    .get("SocketMode")
                    .and_then(|s| u32::from_str_radix(s, 8).ok()),
                socket_user: socket.get("SocketUser").cloned(),
                socket_group: socket.get("SocketGroup").cloned(),
            });
        }
    }

    configs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_unit() {
        let content = r#"
[Unit]
Description=My Test Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/myservice
Restart=on-failure

[Install]
WantedBy=multi-user.target
"#;

        let def = parse_unit_file(content, Path::new("myservice.service")).unwrap();

        assert_eq!(def.name, "myservice");
        assert_eq!(def.description, "My Test Service");
        assert_eq!(def.service_type, ServiceType::Simple);
        assert_eq!(def.exec_start, "/usr/bin/myservice");
        assert_eq!(def.restart, RestartPolicy::OnFailure);
        assert!(def.after.contains(&"network.target".to_string()));
        assert!(def.enabled);
    }

    #[test]
    fn test_parse_complex_unit() {
        let content = r#"
[Unit]
Description=Complex Service
Requires=database.service
After=database.service network.target

[Service]
Type=notify
ExecStart=/usr/bin/complex --config /etc/complex.conf
ExecStop=/usr/bin/complex --stop
ExecReload=/bin/kill -HUP $MAINPID
WorkingDirectory=/var/lib/complex
User=complex
Group=complex
Environment="KEY1=value1" "KEY2=value2"
Restart=always
RestartSec=5
TimeoutStartSec=60
TimeoutStopSec=30
MemoryLimit=512M
CPUQuota=50%
LimitNOFILE=4096
WatchdogSec=30

[Install]
WantedBy=multi-user.target
"#;

        let def = parse_unit_file(content, Path::new("complex.service")).unwrap();

        assert_eq!(def.name, "complex");
        assert_eq!(def.service_type, ServiceType::Notify);
        assert_eq!(def.working_directory.as_deref(), Some(Path::new("/var/lib/complex")));
        assert_eq!(def.user.as_deref(), Some("complex"));
        assert_eq!(def.restart, RestartPolicy::Always);
        assert_eq!(def.restart_sec, Duration::from_secs(5));
        assert_eq!(def.timeout_start_sec, Duration::from_secs(60));

        // Check environment
        assert_eq!(def.environment.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(def.environment.get("KEY2"), Some(&"value2".to_string()));

        // Check resource limits
        let limits = def.resource_limits.unwrap();
        assert_eq!(limits.memory_hard, Some(512 * 1024 * 1024));
        assert_eq!(limits.cpu_percent, Some(50));
        assert_eq!(limits.nofile, Some(4096));

        // Check watchdog
        let watchdog = def.watchdog.unwrap();
        assert_eq!(watchdog.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30"), Some(Duration::from_secs(30)));
        assert_eq!(parse_duration("30s"), Some(Duration::from_secs(30)));
        assert_eq!(parse_duration("5min"), Some(Duration::from_secs(300)));
        assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_duration("100ms"), Some(Duration::from_millis(100)));
    }

    #[test]
    fn test_parse_memory_size() {
        assert_eq!(parse_memory_size("1024"), Some(1024));
        assert_eq!(parse_memory_size("1K"), Some(1024));
        assert_eq!(parse_memory_size("1M"), Some(1024 * 1024));
        assert_eq!(parse_memory_size("1G"), Some(1024 * 1024 * 1024));
    }

    #[test]
    fn test_parse_restart_policy() {
        assert_eq!(parse_restart_policy("no"), RestartPolicy::No);
        assert_eq!(parse_restart_policy("on-failure"), RestartPolicy::OnFailure);
        assert_eq!(parse_restart_policy("always"), RestartPolicy::Always);
    }
}
