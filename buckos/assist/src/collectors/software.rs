//! Software information collectors.

use serde::{Deserialize, Serialize};
use sysinfo::{System, Users};

use crate::error::Result;
use crate::privacy::Redactor;

/// Collected software information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInfo {
    /// Operating system information.
    pub os: OsInfo,
    /// Process information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processes: Option<ProcessSummary>,
    /// User information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<UserInfo>>,
    /// Environment variables (filtered for safety).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<EnvVar>>,
}

/// Operating system information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    /// OS name.
    pub name: String,
    /// OS version.
    pub version: String,
    /// Kernel version.
    pub kernel_version: String,
    /// OS architecture.
    pub arch: String,
    /// Hostname (may be redacted).
    pub hostname: String,
    /// System uptime in seconds.
    pub uptime: u64,
    /// Boot time as Unix timestamp.
    pub boot_time: u64,
}

/// Summary of running processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSummary {
    /// Total number of processes.
    pub total_count: usize,
    /// Number of running processes.
    pub running_count: usize,
    /// Number of sleeping processes.
    pub sleeping_count: usize,
    /// Top processes by CPU usage.
    pub top_by_cpu: Vec<ProcessInfo>,
    /// Top processes by memory usage.
    pub top_by_memory: Vec<ProcessInfo>,
}

/// Information about a single process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// Process name.
    pub name: String,
    /// CPU usage percentage.
    pub cpu_usage: f32,
    /// Memory usage in bytes.
    pub memory: u64,
    /// Process status.
    pub status: String,
}

/// User information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// Username (may be redacted).
    pub name: String,
    /// User's groups.
    pub groups: Vec<String>,
}

/// Environment variable (filtered).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Variable name.
    pub name: String,
    /// Variable value (may be redacted).
    pub value: String,
}

impl SoftwareInfo {
    /// Collect software information.
    pub fn collect(redactor: &Redactor) -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Collect OS info
        let os = Self::collect_os(&sys, redactor);

        // Collect process info if allowed
        let processes = if redactor.should_collect("processes") {
            Some(Self::collect_processes(&sys, redactor))
        } else {
            None
        };

        // Collect user info
        let users = Self::collect_users(redactor);

        // Collect safe environment variables
        let environment = Self::collect_environment(redactor);

        Ok(Self {
            os,
            processes,
            users,
            environment,
        })
    }

    fn collect_os(_sys: &System, redactor: &Redactor) -> OsInfo {
        OsInfo {
            name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            arch: std::env::consts::ARCH.to_string(),
            hostname: redactor
                .redact(&System::host_name().unwrap_or_else(|| "Unknown".to_string())),
            uptime: System::uptime(),
            boot_time: System::boot_time(),
        }
    }

    fn collect_processes(sys: &System, redactor: &Redactor) -> ProcessSummary {
        let processes = sys.processes();

        let mut running_count = 0;
        let mut sleeping_count = 0;
        let mut process_list: Vec<ProcessInfo> = Vec::new();

        for (pid, process) in processes {
            let status = format!("{:?}", process.status());
            if status.contains("Run") {
                running_count += 1;
            } else if status.contains("Sleep") {
                sleeping_count += 1;
            }

            process_list.push(ProcessInfo {
                pid: pid.as_u32(),
                name: redactor.redact(&process.name().to_string()),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
                status,
            });
        }

        // Sort by CPU usage and get top 10
        let mut top_by_cpu = process_list.clone();
        top_by_cpu.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        top_by_cpu.truncate(10);

        // Sort by memory and get top 10
        let mut top_by_memory = process_list;
        top_by_memory.sort_by(|a, b| b.memory.cmp(&a.memory));
        top_by_memory.truncate(10);

        ProcessSummary {
            total_count: processes.len(),
            running_count,
            sleeping_count,
            top_by_cpu,
            top_by_memory,
        }
    }

    fn collect_users(redactor: &Redactor) -> Option<Vec<UserInfo>> {
        let users = Users::new_with_refreshed_list();
        let user_info: Vec<UserInfo> = users
            .iter()
            .map(|user| UserInfo {
                name: redactor.redact(user.name()),
                groups: user.groups().iter().map(|g| g.name().to_string()).collect(),
            })
            .collect();

        if user_info.is_empty() {
            None
        } else {
            Some(user_info)
        }
    }

    fn collect_environment(redactor: &Redactor) -> Option<Vec<EnvVar>> {
        // Only collect safe, non-sensitive environment variables
        let safe_vars = [
            "SHELL",
            "TERM",
            "LANG",
            "LC_ALL",
            "EDITOR",
            "VISUAL",
            "PAGER",
            "XDG_SESSION_TYPE",
            "XDG_CURRENT_DESKTOP",
            "DESKTOP_SESSION",
            "PATH",
            "RUST_VERSION",
            "CARGO_HOME",
        ];

        let env_vars: Vec<EnvVar> = std::env::vars()
            .filter(|(name, _)| {
                safe_vars.contains(&name.as_str())
                    || name.starts_with("XDG_")
                    || name.starts_with("LC_")
            })
            .filter(|(name, _)| {
                // Exclude potentially sensitive variables
                !name.contains("KEY")
                    && !name.contains("SECRET")
                    && !name.contains("TOKEN")
                    && !name.contains("PASSWORD")
                    && !name.contains("CREDENTIAL")
            })
            .map(|(name, value)| EnvVar {
                name,
                value: redactor.redact(&value),
            })
            .collect();

        if env_vars.is_empty() {
            None
        } else {
            Some(env_vars)
        }
    }
}

/// Format duration from seconds into a human-readable string.
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
