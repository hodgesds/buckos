//! System checks and utilities for the installer

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use sysinfo::{Disks, System};

use crate::types::{
    AudioDeviceInfo, DiskInfo, GpuInfo, GpuVendor, HardwareInfo, HardwarePackageSuggestion,
    NetworkInterfaceInfo, NetworkInterfaceType, PartitionInfo, PowerProfile, StorageControllerType,
};

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
    let mut disk_map: std::collections::HashMap<String, DiskInfo> =
        std::collections::HashMap::new();

    // Use lsblk for more detailed disk information
    if let Ok(output) = Command::new("lsblk")
        .args([
            "-b",
            "-o",
            "NAME,SIZE,TYPE,MODEL,RM,MOUNTPOINT,FSTYPE",
            "-J",
        ])
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
                                if let Some(children) =
                                    device.get("children").and_then(|v| v.as_array())
                                {
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

                                disk_map.insert(
                                    device_path.clone(),
                                    DiskInfo {
                                        device: device_path,
                                        model,
                                        size,
                                        removable,
                                        partitions,
                                    },
                                );
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
            if device.starts_with("/dev/sd")
                || device.starts_with("/dev/nvme")
                || device.starts_with("/dev/vd")
            {
                if !disk_map.contains_key(&device) {
                    disk_map.insert(
                        device.clone(),
                        DiskInfo {
                            device: device.clone(),
                            model: "Unknown".to_string(),
                            size: disk.total_space(),
                            removable: disk.is_removable(),
                            partitions: Vec::new(),
                        },
                    );
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
pub fn mount_filesystem(
    device: &str,
    target: &str,
    fstype: Option<&str>,
    options: Option<&str>,
) -> Result<()> {
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
    let regions = [
        "Africa",
        "America",
        "Asia",
        "Atlantic",
        "Australia",
        "Europe",
        "Pacific",
    ];

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

/// Detect GPUs in the system using lspci
pub fn detect_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    // Try lspci first
    if let Ok(output) = Command::new("lspci").args(["-nn", "-d", "::0300"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(gpu) = parse_gpu_line(line) {
                    gpus.push(gpu);
                }
            }
        }
    }

    // Also check for 3D controllers (some GPUs)
    if let Ok(output) = Command::new("lspci").args(["-nn", "-d", "::0302"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(gpu) = parse_gpu_line(line) {
                    gpus.push(gpu);
                }
            }
        }
    }

    // Fallback: check /sys/class/drm
    if gpus.is_empty() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("card") && !name.contains('-') {
                    let vendor_path = entry.path().join("device/vendor");
                    if let Ok(vendor_id) = std::fs::read_to_string(&vendor_path) {
                        let vendor = match vendor_id.trim() {
                            "0x10de" => GpuVendor::Nvidia,
                            "0x1002" => GpuVendor::Amd,
                            "0x8086" => GpuVendor::Intel,
                            "0x80ee" => GpuVendor::VirtualBox,
                            "0x15ad" => GpuVendor::VMware,
                            _ => GpuVendor::Unknown,
                        };
                        gpus.push(GpuInfo {
                            vendor,
                            name: format!("GPU {}", name),
                            pci_id: vendor_id.trim().to_string(),
                        });
                    }
                }
            }
        }
    }

    gpus
}

fn parse_gpu_line(line: &str) -> Option<GpuInfo> {
    // Example: "00:02.0 VGA compatible controller [0300]: Intel Corporation ... [8086:3e92]"
    let vendor = if line.contains("NVIDIA") || line.contains("[10de:") {
        GpuVendor::Nvidia
    } else if line.contains("AMD") || line.contains("ATI") || line.contains("[1002:") {
        GpuVendor::Amd
    } else if line.contains("Intel") || line.contains("[8086:") {
        GpuVendor::Intel
    } else if line.contains("VirtualBox") || line.contains("[80ee:") {
        GpuVendor::VirtualBox
    } else if line.contains("VMware") || line.contains("[15ad:") {
        GpuVendor::VMware
    } else {
        GpuVendor::Unknown
    };

    // Extract PCI ID
    let pci_id = if let Some(start) = line.rfind('[') {
        if let Some(end) = line.rfind(']') {
            line[start + 1..end].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Extract name (between controller type and PCI ID)
    let name = if let Some(colon_pos) = line.find("]: ") {
        let after_type = &line[colon_pos + 3..];
        if let Some(bracket_pos) = after_type.rfind(" [") {
            after_type[..bracket_pos].trim().to_string()
        } else {
            after_type.trim().to_string()
        }
    } else {
        line.to_string()
    };

    Some(GpuInfo {
        vendor,
        name,
        pci_id,
    })
}

/// Detect network interfaces
pub fn detect_network_interfaces() -> Vec<NetworkInterfaceInfo> {
    let mut interfaces = Vec::new();
    let net_path = Path::new("/sys/class/net");

    if let Ok(entries) = std::fs::read_dir(net_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip loopback
            if name == "lo" {
                continue;
            }

            let path = entry.path();

            // Determine interface type
            let interface_type = if path.join("wireless").exists() {
                NetworkInterfaceType::Wifi
            } else if name.starts_with("eth") || name.starts_with("en") {
                NetworkInterfaceType::Ethernet
            } else if name.starts_with("br") || name.starts_with("docker") {
                NetworkInterfaceType::Bridge
            } else if name.starts_with("veth")
                || name.starts_with("virbr")
                || name.starts_with("vnet")
            {
                NetworkInterfaceType::Virtual
            } else {
                NetworkInterfaceType::Unknown
            };

            // Get MAC address
            let mac_address = std::fs::read_to_string(path.join("address"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| s != "00:00:00:00:00:00");

            // Get driver
            let driver = std::fs::read_link(path.join("device/driver"))
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()));

            interfaces.push(NetworkInterfaceInfo {
                name,
                interface_type,
                mac_address,
                driver,
            });
        }
    }

    interfaces
}

/// Detect audio devices
pub fn detect_audio_devices() -> Vec<AudioDeviceInfo> {
    let mut devices = Vec::new();

    // Check /proc/asound/cards
    if let Ok(content) = std::fs::read_to_string("/proc/asound/cards") {
        for line in content.lines() {
            // Lines look like: " 0 [PCH            ]: HDA-Intel - HDA Intel PCH"
            if let Some(bracket_start) = line.find('[') {
                if let Some(bracket_end) = line.find(']') {
                    let card_id = line[bracket_start + 1..bracket_end].trim().to_string();

                    // Get the name part after the colon
                    let name = if let Some(colon_pos) = line.find(':') {
                        line[colon_pos + 1..].trim().to_string()
                    } else {
                        card_id.clone()
                    };

                    let is_hdmi = name.to_lowercase().contains("hdmi")
                        || name.to_lowercase().contains("displayport")
                        || card_id.to_lowercase().contains("hdmi");

                    devices.push(AudioDeviceInfo {
                        name,
                        card_id,
                        is_hdmi,
                    });
                }
            }
        }
    }

    devices
}

/// Detect storage controller type
pub fn detect_storage_controller() -> StorageControllerType {
    // Check for NVMe
    if Path::new("/sys/class/nvme").exists() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/nvme") {
            if entries.count() > 0 {
                return StorageControllerType::Nvme;
            }
        }
    }

    // Check for virtio
    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Virtio") && stdout.contains("block") {
            return StorageControllerType::Virtio;
        }
        if stdout.contains("RAID") {
            return StorageControllerType::Raid;
        }
    }

    // Default to AHCI (SATA)
    StorageControllerType::Ahci
}

/// Detect if system is a laptop
pub fn detect_is_laptop() -> bool {
    // Check for battery
    let battery_path = Path::new("/sys/class/power_supply");
    if let Ok(entries) = std::fs::read_dir(battery_path) {
        for entry in entries.flatten() {
            let type_path = entry.path().join("type");
            if let Ok(ptype) = std::fs::read_to_string(type_path) {
                if ptype.trim() == "Battery" {
                    return true;
                }
            }
        }
    }

    // Check DMI chassis type
    if let Ok(chassis) = std::fs::read_to_string("/sys/class/dmi/id/chassis_type") {
        let chassis_type: u32 = chassis.trim().parse().unwrap_or(0);
        // Laptop chassis types: 8=Portable, 9=Laptop, 10=Notebook, 14=Sub Notebook
        // 30=Tablet, 31=Convertible, 32=Detachable
        if matches!(chassis_type, 8 | 9 | 10 | 14 | 30 | 31 | 32) {
            return true;
        }
    }

    false
}

/// Detect if running in a virtual machine
pub fn detect_is_virtual_machine() -> bool {
    // Check DMI product name
    if let Ok(product) = std::fs::read_to_string("/sys/class/dmi/id/product_name") {
        let product = product.to_lowercase();
        if product.contains("virtualbox")
            || product.contains("vmware")
            || product.contains("qemu")
            || product.contains("kvm")
            || product.contains("hyper-v")
            || product.contains("virtual machine")
        {
            return true;
        }
    }

    // Check systemd-detect-virt
    if let Ok(output) = Command::new("systemd-detect-virt").output() {
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if result != "none" {
                return true;
            }
        }
    }

    // Check /proc/cpuinfo for hypervisor flag
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        if cpuinfo.contains("hypervisor") {
            return true;
        }
    }

    false
}

/// Detect if Bluetooth is available
pub fn detect_bluetooth() -> bool {
    // Check /sys/class/bluetooth
    if let Ok(entries) = std::fs::read_dir("/sys/class/bluetooth") {
        if entries.count() > 0 {
            return true;
        }
    }

    // Check for Bluetooth in lsusb
    if let Ok(output) = Command::new("lsusb").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.to_lowercase().contains("bluetooth") {
                return true;
            }
        }
    }

    false
}

/// Detect if touchscreen is present
pub fn detect_touchscreen() -> bool {
    let input_path = Path::new("/sys/class/input");
    if let Ok(entries) = std::fs::read_dir(input_path) {
        for entry in entries.flatten() {
            let name_path = entry.path().join("device/name");
            if let Ok(name) = std::fs::read_to_string(name_path) {
                let name_lower = name.to_lowercase();
                if name_lower.contains("touch") || name_lower.contains("wacom") {
                    return true;
                }
            }
        }
    }
    false
}

/// Detect buckos-build repository path
///
/// Standard locations checked (in order):
/// 1. User-specified path (if provided)
/// 2. /var/db/repos/buckos-build (standard Gentoo-style location)
/// 3. /usr/share/buckos-build (system-wide, read-only - typical for live USB)
/// 4. /opt/buckos-build (alternative system location)
/// 5. ~/buckos-build (user directory)
/// 6. ./buckos-build (current directory - for development)
pub fn detect_buckos_build_path(custom_path: Option<&str>) -> Result<std::path::PathBuf> {
    // If user provided a path, validate it
    if let Some(path_str) = custom_path {
        let path = std::path::PathBuf::from(path_str);
        return validate_buckos_build_path(&path);
    }

    // Standard locations to check
    let candidate_paths = vec![
        std::path::PathBuf::from("/var/db/repos/buckos-build"),
        std::path::PathBuf::from("/usr/share/buckos-build"),
        std::path::PathBuf::from("/opt/buckos-build"),
    ];

    // Add user home directory path if available
    let mut all_paths = candidate_paths;
    if let Ok(home) = std::env::var("HOME") {
        all_paths.push(std::path::PathBuf::from(home).join("buckos-build"));
    }

    // Add current directory as last resort
    all_paths.push(std::path::PathBuf::from("./buckos-build"));

    // Try each path
    for path in &all_paths {
        if path.exists() {
            match validate_buckos_build_path(path) {
                Ok(p) => {
                    tracing::info!("Found buckos-build at: {}", p.display());
                    return Ok(p);
                }
                Err(e) => {
                    tracing::debug!("Path {} exists but validation failed: {}", path.display(), e);
                    continue;
                }
            }
        }
    }

    // No valid path found
    bail!(
        "Could not find buckos-build repository.\n\
        Searched locations:\n{}\n\n\
        Please specify the path with --buckos-build-path <PATH>\n\
        or ensure buckos-build is installed in one of the standard locations.",
        all_paths
            .iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Validate that a path contains a valid buckos-build repository
fn validate_buckos_build_path(path: &std::path::Path) -> Result<std::path::PathBuf> {
    let path = path.canonicalize().with_context(|| {
        format!("Failed to resolve path: {}", path.display())
    })?;

    if !path.exists() {
        bail!("Path does not exist: {}", path.display());
    }

    if !path.is_dir() {
        bail!("Path is not a directory: {}", path.display());
    }

    // Check for essential buckos-build components
    let required_dirs = vec!["defs", "packages"];
    let required_files = vec!["defs/package_defs.bzl", "defs/use_flags.bzl"];

    // Check directories
    for dir in &required_dirs {
        let dir_path = path.join(dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            bail!(
                "Invalid buckos-build repository: missing required directory '{}' in {}",
                dir,
                path.display()
            );
        }
    }

    // Check files
    for file in &required_files {
        let file_path = path.join(file);
        if !file_path.exists() || !file_path.is_file() {
            bail!(
                "Invalid buckos-build repository: missing required file '{}' in {}",
                file,
                path.display()
            );
        }
    }

    tracing::debug!("Validated buckos-build repository at: {}", path.display());
    Ok(path)
}

/// Get CPU vendor and flags
pub fn get_cpu_info() -> (String, Vec<String>) {
    let mut vendor = String::new();
    let mut flags = Vec::new();

    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("vendor_id") {
                if let Some(v) = line.split(':').nth(1) {
                    vendor = v.trim().to_string();
                }
            } else if line.starts_with("flags") {
                if let Some(f) = line.split(':').nth(1) {
                    flags = f.split_whitespace().map(|s| s.to_string()).collect();
                }
                break; // Only need first CPU's flags
            }
        }
    }

    (vendor, flags)
}

/// Perform complete hardware detection
pub fn detect_hardware() -> HardwareInfo {
    let gpus = detect_gpus();
    let network_interfaces = detect_network_interfaces();
    let audio_devices = detect_audio_devices();
    let storage_controller = detect_storage_controller();
    let is_laptop = detect_is_laptop();
    let is_virtual_machine = detect_is_virtual_machine();
    let has_bluetooth = detect_bluetooth();
    let has_touchscreen = detect_touchscreen();
    let (cpu_vendor, cpu_flags) = get_cpu_info();

    // Determine power profile
    let power_profile = if is_virtual_machine {
        PowerProfile::Desktop
    } else if is_laptop {
        // Check if it's a gaming laptop by looking for high-end GPU
        let has_dedicated_gpu = gpus
            .iter()
            .any(|g| matches!(g.vendor, GpuVendor::Nvidia | GpuVendor::Amd));
        if has_dedicated_gpu {
            PowerProfile::Gaming
        } else {
            PowerProfile::Laptop
        }
    } else {
        PowerProfile::Desktop
    };

    HardwareInfo {
        gpus,
        network_interfaces,
        audio_devices,
        storage_controller,
        power_profile,
        has_bluetooth,
        has_touchscreen,
        is_laptop,
        is_virtual_machine,
        cpu_vendor,
        cpu_flags,
    }
}

/// Generate package suggestions based on detected hardware
pub fn generate_hardware_suggestions(hardware: &HardwareInfo) -> Vec<HardwarePackageSuggestion> {
    let mut suggestions = Vec::new();

    // GPU drivers
    for gpu in &hardware.gpus {
        let packages: Vec<String> = gpu
            .vendor
            .driver_packages()
            .iter()
            .map(|s| s.to_string())
            .collect();
        if !packages.is_empty() {
            suggestions.push(HardwarePackageSuggestion {
                category: "Graphics".to_string(),
                reason: format!(
                    "Detected {} GPU: {}",
                    match gpu.vendor {
                        GpuVendor::Nvidia => "NVIDIA",
                        GpuVendor::Amd => "AMD",
                        GpuVendor::Intel => "Intel",
                        GpuVendor::VirtualBox => "VirtualBox",
                        GpuVendor::VMware => "VMware",
                        GpuVendor::Unknown => "Unknown",
                    },
                    gpu.name
                ),
                packages,
                selected: true,
            });
        }
    }

    // WiFi support
    let has_wifi = hardware
        .network_interfaces
        .iter()
        .any(|i| matches!(i.interface_type, NetworkInterfaceType::Wifi));
    if has_wifi {
        suggestions.push(HardwarePackageSuggestion {
            category: "Networking".to_string(),
            reason: "WiFi interface detected".to_string(),
            packages: vec![
                "wpa_supplicant".to_string(),
                "wireless-tools".to_string(),
                "iw".to_string(),
            ],
            selected: true,
        });
    }

    // Bluetooth
    if hardware.has_bluetooth {
        suggestions.push(HardwarePackageSuggestion {
            category: "Bluetooth".to_string(),
            reason: "Bluetooth adapter detected".to_string(),
            packages: vec!["bluez".to_string(), "bluez-utils".to_string()],
            selected: true,
        });
    }

    // Touchscreen
    if hardware.has_touchscreen {
        suggestions.push(HardwarePackageSuggestion {
            category: "Input".to_string(),
            reason: "Touchscreen detected".to_string(),
            packages: vec!["xf86-input-wacom".to_string(), "libinput".to_string()],
            selected: true,
        });
    }

    // Power management for laptops
    if hardware.is_laptop && !hardware.is_virtual_machine {
        let packages: Vec<String> = hardware
            .power_profile
            .packages()
            .iter()
            .map(|s| s.to_string())
            .collect();
        if !packages.is_empty() {
            suggestions.push(HardwarePackageSuggestion {
                category: "Power Management".to_string(),
                reason: "Laptop detected - power optimization tools".to_string(),
                packages,
                selected: true,
            });
        }
    }

    // Virtual machine tools
    if hardware.is_virtual_machine {
        let vm_packages = if hardware
            .gpus
            .iter()
            .any(|g| matches!(g.vendor, GpuVendor::VirtualBox))
        {
            vec!["virtualbox-guest-additions".to_string()]
        } else if hardware
            .gpus
            .iter()
            .any(|g| matches!(g.vendor, GpuVendor::VMware))
        {
            vec!["open-vm-tools".to_string()]
        } else {
            vec!["qemu-guest-agent".to_string()]
        };

        suggestions.push(HardwarePackageSuggestion {
            category: "Virtualization".to_string(),
            reason: "Running in virtual machine".to_string(),
            packages: vm_packages,
            selected: true,
        });
    }

    // NVMe tools
    if matches!(hardware.storage_controller, StorageControllerType::Nvme) {
        suggestions.push(HardwarePackageSuggestion {
            category: "Storage".to_string(),
            reason: "NVMe storage detected".to_string(),
            packages: vec!["nvme-cli".to_string()],
            selected: true,
        });
    }

    // CPU microcode
    if hardware.cpu_vendor.contains("Intel") {
        suggestions.push(HardwarePackageSuggestion {
            category: "Firmware".to_string(),
            reason: "Intel CPU detected".to_string(),
            packages: vec!["intel-microcode".to_string()],
            selected: true,
        });
    } else if hardware.cpu_vendor.contains("AMD") {
        suggestions.push(HardwarePackageSuggestion {
            category: "Firmware".to_string(),
            reason: "AMD CPU detected".to_string(),
            packages: vec!["amd-microcode".to_string()],
            selected: true,
        });
    }

    suggestions
}

/// Detect handheld gaming device type
pub fn detect_handheld_device() -> Option<crate::types::HandheldDevice> {
    // Check DMI product name
    if let Ok(product) = std::fs::read_to_string("/sys/class/dmi/id/product_name") {
        let product_lower = product.to_lowercase();

        if product_lower.contains("jupiter") || product_lower.contains("steam deck") {
            return Some(crate::types::HandheldDevice::SteamDeck);
        }
        if product_lower.contains("aya") || product_lower.contains("ayaneo") {
            return Some(crate::types::HandheldDevice::AyaNeo);
        }
        if product_lower.contains("gpd") {
            return Some(crate::types::HandheldDevice::GpdWin);
        }
        if product_lower.contains("legion go") {
            return Some(crate::types::HandheldDevice::LegionGo);
        }
        if product_lower.contains("rog ally") {
            return Some(crate::types::HandheldDevice::RogAlly);
        }
    }

    None
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
