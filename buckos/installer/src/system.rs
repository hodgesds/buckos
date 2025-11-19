//! System checks and utilities for the installer

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use sysinfo::{Disks, System};

use crate::types::{DiskInfo, PartitionInfo};

/// Required tools for installation
const REQUIRED_TOOLS: &[(&str, &str)] = &[
    ("fdisk", "util-linux"),
    ("mkfs.ext4", "e2fsprogs"),
    ("mount", "util-linux"),
    ("umount", "util-linux"),
    ("chroot", "coreutils"),
];

/// Optional but recommended tools
const RECOMMENDED_TOOLS: &[(&str, &str)] = &[
    ("parted", "parted"),
    ("mkfs.btrfs", "btrfs-progs"),
    ("mkfs.xfs", "xfsprogs"),
    ("grub-install", "grub"),
    ("blkid", "util-linux"),
    ("lsblk", "util-linux"),
];

/// Check if all required system tools are available
pub fn check_requirements() -> Result<()> {
    let mut missing = Vec::new();

    for (tool, package) in REQUIRED_TOOLS {
        if !tool_exists(tool) {
            missing.push((*tool, *package));
        }
    }

    if !missing.is_empty() {
        let msg = missing
            .iter()
            .map(|(tool, pkg)| format!("  - {} (from {})", tool, pkg))
            .collect::<Vec<_>>()
            .join("\n");
        bail!("Missing required tools:\n{}", msg);
    }

    // Check for root privileges
    if !is_root() {
        bail!("Root privileges required. Please run with sudo or as root.");
    }

    Ok(())
}

/// Check which recommended tools are available
pub fn check_recommended_tools() -> Vec<(&'static str, &'static str, bool)> {
    RECOMMENDED_TOOLS
        .iter()
        .map(|(tool, pkg)| (*tool, *pkg, tool_exists(tool)))
        .collect()
}

/// Check if a tool exists in PATH
fn tool_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if running as root
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Get system information
pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    SystemInfo {
        total_memory: sys.total_memory(),
        available_memory: sys.available_memory(),
        cpu_count: sys.cpus().len(),
        cpu_brand: sys.cpus().first().map(|c| c.brand().to_string()),
        kernel_version: System::kernel_version(),
        os_version: System::os_version(),
        hostname: System::host_name(),
    }
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: usize,
    pub cpu_brand: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub hostname: Option<String>,
}

/// Get list of available disks
pub fn get_available_disks() -> Result<Vec<DiskInfo>> {
    let disks = Disks::new_with_refreshed_list();
    let mut disk_map: std::collections::HashMap<String, DiskInfo> = std::collections::HashMap::new();

    // Use lsblk for more detailed disk information
    if let Ok(output) = Command::new("lsblk")
        .args(["-b", "-o", "NAME,SIZE,TYPE,MODEL,RM,MOUNTPOINT,FSTYPE", "-J"])
        .output()
    {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(devices) = json.get("blockdevices").and_then(|v| v.as_array()) {
                    for device in devices {
                        if let Some(dtype) = device.get("type").and_then(|v| v.as_str()) {
                            if dtype == "disk" {
                                let name = device
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let device_path = format!("/dev/{}", name);

                                let size = device
                                    .get("size")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse().ok())
                                    .or_else(|| device.get("size").and_then(|v| v.as_u64()))
                                    .unwrap_or(0);

                                let model = device
                                    .get("model")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .trim()
                                    .to_string();

                                let removable = device
                                    .get("rm")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s == "1")
                                    .or_else(|| device.get("rm").and_then(|v| v.as_bool()))
                                    .unwrap_or(false);

                                let mut partitions = Vec::new();

                                // Get child partitions
                                if let Some(children) = device.get("children").and_then(|v| v.as_array()) {
                                    for child in children {
                                        let part_name = child
                                            .get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        let part_size = child
                                            .get("size")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| s.parse().ok())
                                            .or_else(|| child.get("size").and_then(|v| v.as_u64()))
                                            .unwrap_or(0);

                                        let fstype = child
                                            .get("fstype")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());

                                        let mountpoint = child
                                            .get("mountpoint")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());

                                        partitions.push(PartitionInfo {
                                            device: format!("/dev/{}", part_name),
                                            size: part_size,
                                            filesystem: fstype,
                                            mount_point: mountpoint,
                                            label: None,
                                        });
                                    }
                                }

                                disk_map.insert(device_path.clone(), DiskInfo {
                                    device: device_path,
                                    model,
                                    size,
                                    removable,
                                    partitions,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to sysinfo if lsblk failed
    if disk_map.is_empty() {
        for disk in disks.list() {
            let device = disk.name().to_string_lossy().to_string();
            // Only include actual disk devices, not partitions
            if device.starts_with("/dev/sd") || device.starts_with("/dev/nvme") || device.starts_with("/dev/vd") {
                if !disk_map.contains_key(&device) {
                    disk_map.insert(device.clone(), DiskInfo {
                        device: device.clone(),
                        model: "Unknown".to_string(),
                        size: disk.total_space(),
                        removable: disk.is_removable(),
                        partitions: Vec::new(),
                    });
                }
            }
        }
    }

    Ok(disk_map.into_values().collect())
}

/// Format a size in bytes to human readable string
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Execute a command and return its output
pub fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute: {} {:?}", cmd, args))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Command failed: {} {:?}\n{}", cmd, args, stderr)
    }
}

/// Mount a filesystem
pub fn mount_filesystem(device: &str, target: &str, fstype: Option<&str>, options: Option<&str>) -> Result<()> {
    let mut args = vec![device, target];

    if let Some(fs) = fstype {
        args.insert(0, "-t");
        args.insert(1, fs);
    }

    if let Some(opts) = options {
        args.push("-o");
        args.push(opts);
    }

    run_command("mount", &args)?;
    tracing::info!("Mounted {} to {}", device, target);
    Ok(())
}

/// Unmount a filesystem
pub fn unmount_filesystem(target: &str) -> Result<()> {
    run_command("umount", &[target])?;
    tracing::info!("Unmounted {}", target);
    Ok(())
}

/// Create a directory
pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Check if EFI boot is available
pub fn is_efi_system() -> bool {
    Path::new("/sys/firmware/efi").exists()
}

/// Get available timezones
pub fn get_timezones() -> Vec<String> {
    let tz_dir = Path::new("/usr/share/zoneinfo");
    if !tz_dir.exists() {
        return vec!["UTC".to_string()];
    }

    let mut timezones = Vec::new();

    // Common timezone regions
    let regions = ["Africa", "America", "Asia", "Atlantic", "Australia", "Europe", "Pacific"];

    for region in regions {
        let region_path = tz_dir.join(region);
        if region_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(region_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('.') {
                        timezones.push(format!("{}/{}", region, name));
                    }
                }
            }
        }
    }

    timezones.push("UTC".to_string());
    timezones.sort();
    timezones
}

/// Get available keyboard layouts
pub fn get_keyboard_layouts() -> Vec<String> {
    // Common keyboard layouts
    vec![
        "us".to_string(),
        "gb".to_string(),
        "de".to_string(),
        "fr".to_string(),
        "es".to_string(),
        "it".to_string(),
        "pt".to_string(),
        "ru".to_string(),
        "jp".to_string(),
        "kr".to_string(),
        "br".to_string(),
        "ca".to_string(),
        "ch".to_string(),
        "pl".to_string(),
        "se".to_string(),
        "no".to_string(),
        "dk".to_string(),
        "fi".to_string(),
    ]
}

/// Get available locales
pub fn get_locales() -> Vec<String> {
    // Common locales
    vec![
        "en_US.UTF-8".to_string(),
        "en_GB.UTF-8".to_string(),
        "de_DE.UTF-8".to_string(),
        "fr_FR.UTF-8".to_string(),
        "es_ES.UTF-8".to_string(),
        "it_IT.UTF-8".to_string(),
        "pt_BR.UTF-8".to_string(),
        "ru_RU.UTF-8".to_string(),
        "ja_JP.UTF-8".to_string(),
        "ko_KR.UTF-8".to_string(),
        "zh_CN.UTF-8".to_string(),
        "zh_TW.UTF-8".to_string(),
        "pl_PL.UTF-8".to_string(),
        "nl_NL.UTF-8".to_string(),
        "sv_SE.UTF-8".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_tool_exists() {
        // These should exist on most systems
        assert!(tool_exists("ls"));
        assert!(!tool_exists("nonexistent_tool_12345"));
    }
}
