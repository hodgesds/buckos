//! Installation logic and helpers

use crate::system;
use crate::types::{
    AudioSubsystem, DesktopEnvironment, FilesystemType, GpuVendor, InitSystem, InstallConfig,
    InstallProfile, InstallProgress, MountPoint, NetworkInterfaceType, PartitionConfig,
    SystemLimitsConfig,
};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// USE flag configuration generated from installer options
#[derive(Debug, Clone, Default)]
pub struct UseFlags {
    /// Global USE flags
    pub global: Vec<String>,
    /// VIDEO_CARDS USE_EXPAND
    pub video_cards: Vec<String>,
    /// INPUT_DEVICES USE_EXPAND
    pub input_devices: Vec<String>,
    /// Per-package USE flag overrides
    pub package_use: Vec<(String, Vec<String>)>,
}

impl UseFlags {
    /// Generate USE flags from InstallConfig
    pub fn from_config(config: &InstallConfig) -> Self {
        let mut flags = UseFlags::default();

        // Init system USE flag
        let init_flag = match config.init_system {
            InitSystem::Systemd => "systemd",
            InitSystem::OpenRC => "openrc",
            InitSystem::Runit => "runit",
            InitSystem::S6 => "s6",
            InitSystem::SysVinit => "sysvinit",
            InitSystem::Dinit => "dinit",
            InitSystem::BusyBoxInit => "busybox",
        };
        flags.global.push(init_flag.to_string());

        // Audio subsystem USE flags
        match config.audio_subsystem {
            AudioSubsystem::PipeWire => {
                flags.global.push("pipewire".to_string());
                flags.global.push("alsa".to_string());
            }
            AudioSubsystem::PulseAudio => {
                flags.global.push("pulseaudio".to_string());
                flags.global.push("alsa".to_string());
            }
            AudioSubsystem::Alsa => {
                flags.global.push("alsa".to_string());
            }
        }

        // Desktop environment / display server USE flags
        if let InstallProfile::Desktop(de) = &config.profile {
            // Display server flags
            match de {
                DesktopEnvironment::Sway | DesktopEnvironment::Hyprland => {
                    flags.global.push("wayland".to_string());
                }
                DesktopEnvironment::I3 => {
                    flags.global.push("X".to_string());
                }
                DesktopEnvironment::Gnome | DesktopEnvironment::Kde => {
                    flags.global.push("wayland".to_string());
                    flags.global.push("X".to_string()); // XWayland
                }
                DesktopEnvironment::Xfce
                | DesktopEnvironment::Mate
                | DesktopEnvironment::Cinnamon
                | DesktopEnvironment::LxQt => {
                    flags.global.push("X".to_string());
                }
                DesktopEnvironment::None => {}
            }

            // Toolkit flags
            match de {
                DesktopEnvironment::Gnome
                | DesktopEnvironment::Xfce
                | DesktopEnvironment::Mate
                | DesktopEnvironment::Cinnamon => {
                    flags.global.push("gtk".to_string());
                }
                DesktopEnvironment::Kde | DesktopEnvironment::LxQt => {
                    flags.global.push("qt5".to_string());
                    flags.global.push("qt6".to_string());
                }
                _ => {}
            }
        }

        // Hardware detection -> USE flags
        if config.hardware_info.has_bluetooth {
            flags.global.push("bluetooth".to_string());
        }

        let has_wifi = config
            .hardware_info
            .network_interfaces
            .iter()
            .any(|i| matches!(i.interface_type, NetworkInterfaceType::Wifi));
        if has_wifi {
            flags.global.push("wifi".to_string());
        }

        if config.hardware_info.has_touchscreen {
            flags.global.push("touchscreen".to_string());
            flags.input_devices.push("libinput".to_string());
            flags.input_devices.push("wacom".to_string());
        }

        // GPU detection -> VIDEO_CARDS USE_EXPAND
        for gpu in &config.hardware_info.gpus {
            match gpu.vendor {
                GpuVendor::Nvidia => {
                    flags.video_cards.push("nvidia".to_string());
                }
                GpuVendor::Amd => {
                    flags.video_cards.push("amdgpu".to_string());
                    flags.video_cards.push("radeon".to_string());
                }
                GpuVendor::Intel => {
                    flags.video_cards.push("intel".to_string());
                    flags.video_cards.push("i965".to_string());
                }
                GpuVendor::VirtualBox => {
                    flags.video_cards.push("virtualbox".to_string());
                }
                GpuVendor::VMware => {
                    flags.video_cards.push("vmware".to_string());
                }
                GpuVendor::Unknown => {}
            }
        }

        // Common USE flags for all installations
        for flag in &[
            "unicode", "nls", "ssl", "ipv6", "threads", "ncurses", "readline", "zlib", "pam",
            "udev", "dbus",
        ] {
            flags.global.push(flag.to_string());
        }

        // Per-package USE flag overrides
        flags
            .package_use
            .push(("dracut".to_string(), vec![init_flag.to_string()]));

        flags
    }

    /// Generate Starlark format for use_config.bzl
    pub fn to_starlark(&self) -> String {
        let use_flags_str = self
            .global
            .iter()
            .map(|f| format!("    \"{}\",", f))
            .collect::<Vec<_>>()
            .join("\n");

        let video_cards_str = self
            .video_cards
            .iter()
            .map(|f| format!("    \"{}\",", f))
            .collect::<Vec<_>>()
            .join("\n");

        let input_devices_str = self
            .input_devices
            .iter()
            .map(|f| format!("    \"{}\",", f))
            .collect::<Vec<_>>()
            .join("\n");

        let package_use_str = self
            .package_use
            .iter()
            .map(|(pkg, pkg_flags)| {
                let flags_str = pkg_flags
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("    \"{}\": [{}],", pkg, flags_str)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"# Auto-generated USE flag configuration
# Generated by BuckOS installer based on installation options.
# Do not edit manually - changes will be overwritten.

# Global USE flags for this installation
INSTALL_USE_FLAGS = [
{}
]

# USE_EXPAND: VIDEO_CARDS
INSTALL_VIDEO_CARDS = [
{}
]

# USE_EXPAND: INPUT_DEVICES
INSTALL_INPUT_DEVICES = [
{}
]

# Per-package USE flag overrides
INSTALL_PACKAGE_USE = {{
{}
}}
"#,
            use_flags_str, video_cards_str, input_devices_str, package_use_str
        )
    }

    /// Generate TOML format for /etc/buckos/buckos.toml
    pub fn to_toml(&self) -> String {
        let use_flags_toml = self
            .global
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(", ");

        // Detect architecture from hardware info or system
        let (arch, chost, march) = detect_target_arch();

        let jobs = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        format!(
            r#"# BuckOS Package Manager Configuration
# Generated by BuckOS installer
# Edit this file to customize package manager behavior

[general]
root = "/"
db_path = "/var/db/buckos"
cache_dir = "/var/cache/buckos"
buck_repo = "/var/db/repos/buckos-build"
buck_path = "/usr/bin/buck2"

[use_flags]
global = [{use_flags}]

[build]
arch = "{arch}"
chost = "{chost}"
cflags = "-O2 -pipe {march}"
cxxflags = "${{CFLAGS}}"
ldflags = "-Wl,-O1 -Wl,--as-needed"
makeopts = "-j{jobs}"

[features]
parallel-fetch = true
parallel-install = true
buildpkg = true
"#,
            use_flags = use_flags_toml,
            arch = arch,
            chost = chost,
            march = march,
            jobs = jobs
        )
    }
}

/// Detect target architecture and return (arch, chost, march flags)
fn detect_target_arch() -> (&'static str, &'static str, &'static str) {
    #[cfg(target_arch = "x86_64")]
    {
        (
            "amd64",
            "x86_64-buckos-linux-gnu",
            "-march=x86-64 -mtune=generic",
        )
    }
    #[cfg(target_arch = "aarch64")]
    {
        ("arm64", "aarch64-buckos-linux-gnu", "-march=armv8-a")
    }
    #[cfg(target_arch = "x86")]
    {
        ("x86", "i686-buckos-linux-gnu", "-march=i686 -mtune=generic")
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
    {
        ("unknown", "unknown-buckos-linux-gnu", "")
    }
}

// ============================================================================
// Installation Helper Functions
// ============================================================================

/// Check disk safety - ensure we're not installing on the running system's disk
fn check_disk_safety(disk_device: &str) -> anyhow::Result<()> {
    let root_device_result = Command::new("findmnt")
        .args(["-n", "-o", "SOURCE", "/"])
        .output();

    if let Ok(output) = root_device_result {
        if output.status.success() {
            let root_partition = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Use lsblk to get the parent disk device
            let lsblk_result = Command::new("lsblk")
                .args(["-no", "PKNAME", &root_partition])
                .output();

            let root_disk = if let Ok(lsblk_output) = lsblk_result {
                if lsblk_output.status.success() {
                    let pkname = String::from_utf8_lossy(&lsblk_output.stdout)
                        .trim()
                        .to_string();
                    if !pkname.is_empty() {
                        format!("/dev/{}", pkname)
                    } else {
                        root_partition.clone()
                    }
                } else {
                    root_partition.clone()
                }
            } else {
                root_partition.clone()
            };

            if disk_device == root_disk || root_disk.contains(disk_device) {
                anyhow::bail!(
                    "SAFETY CHECK FAILED: Cannot install on {} - it contains the running system's root filesystem ({}).\n\
                    This would destroy your running system!\n\
                    Please select a different disk or boot from a live USB/CD to install.",
                    disk_device, root_partition
                );
            }

            tracing::info!(
                "Safety check passed: root is on {}, installing to {}",
                root_disk,
                disk_device
            );
        }
    }
    Ok(())
}

/// Unmount all partitions from a target disk
fn unmount_disk_partitions(disk_device: &str) -> anyhow::Result<()> {
    let findmnt_output = Command::new("findmnt")
        .args(["-rno", "TARGET,SOURCE"])
        .output();

    let mut mount_points_to_unmount = Vec::new();

    if let Ok(output) = findmnt_output {
        if output.status.success() {
            let mount_list = String::from_utf8_lossy(&output.stdout);
            for line in mount_list.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mount_point = parts[0];
                    let device = parts[1];

                    if device.starts_with(disk_device) {
                        tracing::info!("Found mounted: {} on {}", device, mount_point);
                        mount_points_to_unmount.push((mount_point.to_string(), device.to_string()));
                    }
                }
            }
        }
    }

    tracing::info!(
        "Found {} mount points to unmount",
        mount_points_to_unmount.len()
    );

    // Sort mount points by depth (deepest first)
    mount_points_to_unmount.sort_by(|a, b| b.0.matches('/').count().cmp(&a.0.matches('/').count()));

    for (mount_point, device) in &mount_points_to_unmount {
        tracing::info!("Unmounting: {} ({})", mount_point, device);

        let umount_result = Command::new("umount").arg(mount_point).output();

        match umount_result {
            Ok(output) if output.status.success() => {
                tracing::info!("Successfully unmounted {}", mount_point);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    "Failed to unmount {}: {}, trying lazy unmount",
                    mount_point,
                    stderr
                );
                let _ = Command::new("umount").args(["-l", mount_point]).output();
            }
            Err(e) => {
                tracing::warn!("Failed to unmount {}: {}", mount_point, e);
            }
        }
    }

    Ok(())
}

/// Deactivate swap partitions on a target disk
fn deactivate_swap(disk_device: &str) {
    match Command::new("swapon")
        .arg("--show=NAME")
        .arg("--noheadings")
        .output()
    {
        Ok(output) if output.status.success() => {
            let swap_list = String::from_utf8_lossy(&output.stdout);
            for swap_device in swap_list.lines() {
                let swap_device = swap_device.trim();
                if swap_device.starts_with(disk_device) {
                    tracing::info!("Deactivating swap on {}", swap_device);
                    if let Err(e) = Command::new("swapoff").arg(swap_device).output() {
                        tracing::warn!("Failed to run swapoff: {}", e);
                    }
                }
            }
        }
        Ok(_) => tracing::debug!("swapon command returned non-zero status"),
        Err(e) => tracing::debug!("swapon command not available: {}", e),
    }
}

/// Kill processes using a disk device
fn kill_disk_processes(disk_device: &str) {
    match Command::new("fuser").args(["-m", disk_device]).output() {
        Ok(output) => {
            let users = String::from_utf8_lossy(&output.stdout);
            if !users.trim().is_empty() {
                tracing::warn!("Processes still using {}: {}", disk_device, users);
                let _ = Command::new("fuser").args(["-km", disk_device]).output();
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        Err(e) => {
            tracing::debug!("fuser command not available: {}", e);
        }
    }
}

/// Clean up device-mapper entries (LVM, dm-crypt, etc.) on a disk
fn cleanup_device_mapper(disk_device: &str) {
    tracing::info!("Cleaning up device-mapper entries on {}", disk_device);

    // Get the disk name without /dev/ prefix for matching
    let disk_name = disk_device.trim_start_matches("/dev/");

    // Deactivate LVM volume groups that use this disk
    if let Ok(output) = Command::new("pvs")
        .args(["--noheadings", "-o", "pv_name,vg_name"])
        .output()
    {
        if output.status.success() {
            let pv_list = String::from_utf8_lossy(&output.stdout);
            for line in pv_list.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0].contains(disk_name) {
                    let vg_name = parts[1];
                    tracing::info!("Deactivating LVM volume group: {}", vg_name);
                    let _ = Command::new("vgchange").args(["-an", vg_name]).output();
                }
            }
        }
    }

    // Remove device-mapper entries related to this disk
    if let Ok(output) = Command::new("dmsetup").args(["ls"]).output() {
        if output.status.success() {
            let dm_list = String::from_utf8_lossy(&output.stdout);
            for line in dm_list.lines() {
                if let Some(dm_name) = line.split_whitespace().next() {
                    // Check if this dm device is backed by our target disk
                    if let Ok(deps) = Command::new("dmsetup")
                        .args(["deps", "-o", "devname", dm_name])
                        .output()
                    {
                        let deps_str = String::from_utf8_lossy(&deps.stdout);
                        if deps_str.contains(disk_name) {
                            tracing::info!("Removing device-mapper entry: {}", dm_name);
                            let _ = Command::new("dmsetup")
                                .args(["remove", "--force", dm_name])
                                .output();
                        }
                    }
                }
            }
        }
    }
}

/// Prepare disk for partitioning by flushing buffers and settling udev
fn prepare_disk_for_partitioning(disk_device: &str) {
    tracing::info!("Preparing disk {} for partitioning", disk_device);

    // Sync filesystem buffers
    let _ = Command::new("sync").output();

    // Flush block device buffers
    if let Err(e) = Command::new("blockdev")
        .args(["--flushbufs", disk_device])
        .output()
    {
        tracing::debug!("blockdev --flushbufs failed: {}", e);
    }

    // Re-read partition table (clear kernel's cached partition info)
    if let Err(e) = Command::new("blockdev")
        .args(["--rereadpt", disk_device])
        .output()
    {
        tracing::debug!("blockdev --rereadpt failed: {}", e);
    }

    // Wait for udev to settle
    let _ = Command::new("udevadm")
        .args(["settle", "--timeout=5"])
        .output();

    // Small delay to ensure everything is settled
    std::thread::sleep(std::time::Duration::from_millis(500));
}

/// Create partition table on a disk
fn create_partition_table(disk_device: &str, use_gpt: bool) -> anyhow::Result<()> {
    let pt_type = if use_gpt { "gpt" } else { "msdos" };
    tracing::info!("Creating {} partition table on {}", pt_type, disk_device);

    // Retry logic - sometimes the kernel needs a moment to release the device
    let max_retries = 3;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        // Before each attempt, ensure the kernel's view is fresh
        let _ = Command::new("sync").output();
        let _ = Command::new("blockdev")
            .args(["--flushbufs", disk_device])
            .output();
        let _ = Command::new("blockdev")
            .args(["--rereadpt", disk_device])
            .output();
        let _ = Command::new("udevadm")
            .args(["settle", "--timeout=3"])
            .output();

        if attempt > 1 {
            tracing::info!(
                "Retry attempt {} of {} for partition table creation",
                attempt,
                max_retries
            );
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let output = Command::new("parted")
            .args(["-s", disk_device, "mklabel", pt_type])
            .output()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to execute parted command: {}. Make sure parted is installed and in PATH.",
                    e
                )
            })?;

        if output.status.success() {
            if attempt > 1 {
                tracing::info!("Partition table created successfully on retry {}", attempt);
            }
            return Ok(());
        }

        last_error = String::from_utf8_lossy(&output.stderr).to_string();
        tracing::warn!(
            "Attempt {} failed to create partition table: {}",
            attempt,
            last_error
        );
    }

    // All retries failed - provide diagnostics
    tracing::error!(
        "Failed to create partition table on {} after {} attempts",
        disk_device,
        max_retries
    );

    // Diagnostic: check what's still mounted
    if let Ok(mounts) = Command::new("findmnt")
        .args(["-rno", "TARGET,SOURCE"])
        .output()
    {
        if mounts.status.success() {
            let mount_list = String::from_utf8_lossy(&mounts.stdout);
            for line in mount_list.lines() {
                if line.contains(disk_device) {
                    tracing::error!("Still mounted: {}", line);
                }
            }
        }
    }

    // Check for holders (dm, md, etc.)
    let disk_name = disk_device.trim_start_matches("/dev/");
    let holders_path = format!("/sys/block/{}/holders", disk_name);
    if let Ok(entries) = std::fs::read_dir(&holders_path) {
        for entry in entries.flatten() {
            tracing::error!("Device has holder: {:?}", entry.file_name());
        }
    }

    anyhow::bail!(
        "Failed to create partition table after {} attempts: {}\n\
        Check logs for diagnostic information.",
        max_retries,
        last_error
    );
}

/// Create a single partition on a disk
fn create_partition(
    disk_device: &str,
    partition: &PartitionConfig,
    idx: usize,
    start_mb: u64,
    use_gpt: bool,
) -> anyhow::Result<u64> {
    let size_mb = if partition.size == 0 {
        "100%".to_string()
    } else {
        format!("{}MB", start_mb + (partition.size / 1024 / 1024))
    };

    let part_type = if use_gpt {
        match partition.mount_point {
            MountPoint::BootEfi => "fat32",
            MountPoint::Swap => "linux-swap",
            MountPoint::Boot if partition.filesystem == FilesystemType::None => "bios_grub",
            _ => "ext4",
        }
    } else {
        match partition.mount_point {
            MountPoint::BootEfi => "fat32",
            MountPoint::Swap => "linux-swap",
            _ => "ext4",
        }
    };

    let start = format!("{}MB", start_mb);
    let output = Command::new("parted")
        .args([
            "-s",
            disk_device,
            "mkpart",
            "primary",
            part_type,
            &start,
            &size_mb,
        ])
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute parted command for partition {}: {}",
                partition.device,
                e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to create partition {}: {}",
            partition.device,
            stderr
        );
    }

    // Set ESP flag for EFI partition
    if partition.mount_point == MountPoint::BootEfi && use_gpt {
        let part_num = (idx + 1).to_string();
        let output = Command::new("parted")
            .args(["-s", disk_device, "set", &part_num, "esp", "on"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute parted set esp: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                "Failed to set ESP flag on partition {}: {}",
                partition.device,
                stderr
            );
        }
    }

    // Set bios_grub flag for BIOS boot partition
    if partition.mount_point == MountPoint::Boot
        && partition.filesystem == FilesystemType::None
        && use_gpt
    {
        let part_num = (idx + 1).to_string();
        let output = Command::new("parted")
            .args(["-s", disk_device, "set", &part_num, "bios_grub", "on"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute parted set bios_grub: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                "Failed to set bios_grub flag on partition {}: {}",
                partition.device,
                stderr
            );
        }
    }

    // Return new start position for next partition
    if partition.size > 0 {
        Ok(start_mb + partition.size / 1024 / 1024)
    } else {
        Ok(start_mb)
    }
}

/// Format a partition with the specified filesystem
fn format_partition(partition: &PartitionConfig) -> anyhow::Result<()> {
    if !partition.format {
        return Ok(());
    }

    let (fs_cmd, args): (&str, Vec<&str>) = match partition.filesystem {
        FilesystemType::Ext4 => ("mkfs.ext4", vec!["-F", partition.device.as_str()]),
        FilesystemType::Btrfs => ("mkfs.btrfs", vec!["-f", partition.device.as_str()]),
        FilesystemType::Xfs => ("mkfs.xfs", vec!["-f", partition.device.as_str()]),
        FilesystemType::F2fs => ("mkfs.f2fs", vec!["-f", partition.device.as_str()]),
        FilesystemType::Fat32 => ("mkfs.vfat", vec!["-F", "32", partition.device.as_str()]),
        FilesystemType::Swap => ("mkswap", vec![partition.device.as_str()]),
        FilesystemType::None => return Ok(()),
    };

    let output = Command::new(fs_cmd).args(&args).output().map_err(|e| {
        anyhow::anyhow!(
            "Failed to execute {} command for {}: {}. Make sure {} is installed.",
            fs_cmd,
            partition.device,
            e,
            fs_cmd
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to create filesystem on {}: {}",
            partition.device,
            stderr
        );
    }

    Ok(())
}

/// Mount a partition to a target path
fn mount_partition(
    partition: &PartitionConfig,
    target_root: &std::path::Path,
) -> anyhow::Result<()> {
    let mount_path = partition.mount_point.path();

    if partition.filesystem == FilesystemType::Swap {
        let output = Command::new("swapon")
            .arg(&partition.device)
            .output()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to execute swapon command for {}: {}",
                    partition.device,
                    e
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Failed to activate swap on {}: {}",
                partition.device,
                stderr
            );
        }
        return Ok(());
    }

    let full_mount_path = if mount_path == "/" {
        target_root.to_path_buf()
    } else {
        target_root.join(mount_path.trim_start_matches('/'))
    };

    std::fs::create_dir_all(&full_mount_path)?;

    let mut mount_cmd = Command::new("mount");
    mount_cmd.arg(&partition.device);

    if !partition.mount_options.is_empty() {
        mount_cmd.arg("-o").arg(&partition.mount_options);
    }

    mount_cmd.arg(&full_mount_path);

    let output = mount_cmd.output().map_err(|e| {
        anyhow::anyhow!(
            "Failed to execute mount command for {} to {}: {}",
            partition.device,
            full_mount_path.display(),
            e
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to mount {} to {}: {}",
            partition.device,
            full_mount_path.display(),
            stderr
        );
    }

    Ok(())
}

/// Initialize essential system files (passwd, shadow, group)
fn init_system_files(target_root: &std::path::Path) -> anyhow::Result<()> {
    let etc_dir = target_root.join("etc");
    std::fs::create_dir_all(&etc_dir)?;

    let passwd_path = etc_dir.join("passwd");
    if !passwd_path.exists() {
        let passwd_content = "\
root:x:0:0:root:/root:/bin/bash
bin:x:1:1:bin:/bin:/sbin/nologin
daemon:x:2:2:daemon:/sbin:/sbin/nologin
nobody:x:65534:65534:Kernel Overflow User:/:/sbin/nologin
";
        std::fs::write(&passwd_path, passwd_content)?;
    }

    let shadow_path = etc_dir.join("shadow");
    if !shadow_path.exists() {
        let shadow_content = "\
root:!:19000:0:99999:7:::
bin:*:19000:0:99999:7:::
daemon:*:19000:0:99999:7:::
nobody:*:19000:0:99999:7:::
";
        std::fs::write(&shadow_path, shadow_content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&shadow_path, std::fs::Permissions::from_mode(0o600))?;
        }
    }

    let group_path = etc_dir.join("group");
    if !group_path.exists() {
        let group_content = "\
root:x:0:
bin:x:1:
daemon:x:2:
wheel:x:10:
nobody:x:65534:
";
        std::fs::write(&group_path, group_content)?;
    }

    Ok(())
}

/// Generate fstab from partition configuration
fn generate_fstab(
    target_root: &std::path::Path,
    partitions: &[PartitionConfig],
) -> anyhow::Result<()> {
    let fstab_path = target_root.join("etc/fstab");
    std::fs::create_dir_all(fstab_path.parent().unwrap())?;

    let mut fstab_content = String::from("# /etc/fstab: static file system information\n");
    fstab_content.push_str("# <device>  <mount point>  <type>  <options>  <dump>  <pass>\n\n");

    // Add essential virtual filesystems required by systemd
    fstab_content.push_str("# Virtual filesystems\n");
    fstab_content.push_str("proc      /proc      proc    defaults,nosuid,nodev,noexec  0  0\n");
    fstab_content.push_str("sysfs     /sys       sysfs   defaults,nosuid,nodev,noexec  0  0\n");
    fstab_content.push_str("devtmpfs  /dev       devtmpfs  mode=0755,nosuid             0  0\n");
    fstab_content.push_str("devpts    /dev/pts   devpts  mode=0620,gid=5,nosuid,noexec  0  0\n");
    fstab_content.push_str("tmpfs     /run       tmpfs   defaults,nosuid,nodev,mode=0755  0  0\n");
    fstab_content.push_str("tmpfs     /dev/shm   tmpfs   defaults,nosuid,nodev           0  0\n");
    fstab_content.push_str("tmpfs     /tmp       tmpfs   defaults,nosuid,nodev           0  0\n\n");

    // Add user-defined partitions
    fstab_content.push_str("# User partitions\n");
    for partition in partitions {
        let mount_path = partition.mount_point.path();
        let fs_type = partition.filesystem.as_str();
        let options = if partition.mount_options.is_empty() {
            "defaults".to_string()
        } else {
            partition.mount_options.clone()
        };

        let (dump, pass) = if mount_path == "/" {
            ("0", "1")
        } else if mount_path == "swap" {
            ("0", "0")
        } else {
            ("0", "2")
        };

        fstab_content.push_str(&format!(
            "{}  {}  {}  {}  {}  {}\n",
            partition.device, mount_path, fs_type, options, dump, pass
        ));
    }

    std::fs::write(&fstab_path, fstab_content)?;
    Ok(())
}

/// Configure system limits (ulimits and sysctl)
fn configure_system_limits(
    target_root: &std::path::Path,
    limits_config: &SystemLimitsConfig,
) -> anyhow::Result<()> {
    // Create limits.d directory
    if limits_config.apply_ulimits {
        let limits_dir = target_root.join("etc/security/limits.d");
        std::fs::create_dir_all(&limits_dir)?;

        let limits_path = limits_dir.join("99-buckos.conf");
        let limits_content = system::generate_limits_conf(limits_config);
        std::fs::write(&limits_path, limits_content)?;
        tracing::info!("Generated /etc/security/limits.d/99-buckos.conf");

        // Also create an audio group if realtime audio is enabled
        if limits_config.enable_realtime_audio {
            let group_path = target_root.join("etc/group");
            if let Ok(group_content) = std::fs::read_to_string(&group_path) {
                if !group_content.contains("audio:") {
                    // Add audio group if it doesn't exist
                    let mut new_content = group_content;
                    if !new_content.ends_with('\n') {
                        new_content.push('\n');
                    }
                    new_content.push_str("audio:x:18:\n");
                    std::fs::write(&group_path, new_content)?;
                    tracing::info!("Added audio group for realtime scheduling");
                }
            }
        }
    }

    // Create sysctl.d directory
    if limits_config.apply_sysctl {
        let sysctl_dir = target_root.join("etc/sysctl.d");
        std::fs::create_dir_all(&sysctl_dir)?;

        let sysctl_path = sysctl_dir.join("99-buckos.conf");
        let sysctl_content = system::generate_sysctl_conf(limits_config);
        std::fs::write(&sysctl_path, sysctl_content)?;
        tracing::info!("Generated /etc/sysctl.d/99-buckos.conf");
    }

    Ok(())
}

/// Configure locale settings
fn configure_locale(target_root: &std::path::Path, locale: &str) -> anyhow::Result<()> {
    let locale_conf_path = target_root.join("etc/locale.conf");
    std::fs::write(&locale_conf_path, format!("LANG={}\n", locale))?;

    let locale_gen_path = target_root.join("etc/locale.gen");
    std::fs::write(
        &locale_gen_path,
        format!("{} UTF-8\n", locale.trim_end_matches(".UTF-8")),
    )?;

    // Run locale-gen in chroot
    let _ = Command::new("chroot")
        .arg(target_root)
        .env_clear()
        .env(
            "PATH",
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        )
        .env("HOME", "/root")
        .env("TERM", "linux")
        .arg("locale-gen")
        .output();

    Ok(())
}

/// Configure timezone
fn configure_timezone(target_root: &std::path::Path, timezone: &str) -> anyhow::Result<()> {
    let timezone_src = format!("/usr/share/zoneinfo/{}", timezone);
    let timezone_dst = target_root.join("etc/localtime");

    let timezone_src_in_target = target_root.join(timezone_src.trim_start_matches('/'));
    if timezone_src_in_target.exists() {
        if timezone_dst.exists() {
            std::fs::remove_file(&timezone_dst)?;
        }
        std::os::unix::fs::symlink(&timezone_src, &timezone_dst)?;
    }

    let timezone_name_path = target_root.join("etc/timezone");
    std::fs::write(&timezone_name_path, format!("{}\n", timezone))?;

    Ok(())
}

/// Configure hostname and hosts file
fn configure_network(target_root: &std::path::Path, hostname: &str) -> anyhow::Result<()> {
    let hostname_path = target_root.join("etc/hostname");
    std::fs::write(&hostname_path, format!("{}\n", hostname))?;

    let hosts_path = target_root.join("etc/hosts");
    let hosts_content = format!(
        "127.0.0.1\tlocalhost\n\
         ::1\t\tlocalhost\n\
         127.0.1.1\t{}\n",
        hostname
    );
    std::fs::write(&hosts_path, hosts_content)?;

    Ok(())
}

/// Configure keyboard layout
fn configure_keyboard(target_root: &std::path::Path, keymap: &str) -> anyhow::Result<()> {
    let vconsole_path = target_root.join("etc/vconsole.conf");
    std::fs::write(&vconsole_path, format!("KEYMAP={}\n", keymap))?;
    Ok(())
}

/// Configure init system (systemd, OpenRC, etc.)
/// Configure binary package mirror in .buckconfig
fn configure_binary_mirror(buckos_build_path: &std::path::Path) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;

    let buckconfig_path = buckos_build_path.join(".buckconfig");

    // Read existing .buckconfig if it exists
    let mut config_content = if buckconfig_path.exists() {
        fs::read_to_string(&buckconfig_path)?
    } else {
        String::new()
    };

    // Check if [buckos] section already exists
    if config_content.contains("[buckos]") {
        tracing::info!("Binary mirror configuration already exists in .buckconfig");
        return Ok(());
    }

    // Append binary package mirror configuration
    let binary_mirror_config = r#"

# BuckOS Binary Package Configuration
[buckos]
# Mirror URL for precompiled binary packages
# Set this to your binary mirror to enable binary package downloads
# Default: Official BuckOS mirror
binary_mirror = https://mirror.buckos.org

# Whether to prefer binary packages over source builds (default: true)
# Set to false to always build from source even when binaries are available
prefer_binaries = true

"#;

    config_content.push_str(binary_mirror_config);

    // Write updated config
    let mut file = fs::File::create(&buckconfig_path)?;
    file.write_all(config_content.as_bytes())?;

    tracing::info!(
        "Configured binary package mirror in {}",
        buckconfig_path.display()
    );

    Ok(())
}

fn configure_init_system(
    target_root: &std::path::Path,
    init_system: &InitSystem,
) -> anyhow::Result<()> {
    match init_system {
        InitSystem::Systemd => {
            tracing::info!("Configuring systemd...");

            // Create essential systemd directories
            let systemd_dirs = [
                "etc/systemd/system",
                "etc/systemd/network",
                "etc/systemd/resolved.conf.d",
                "etc/systemd/journald.conf.d",
                "var/lib/systemd",
            ];
            for dir in &systemd_dirs {
                std::fs::create_dir_all(target_root.join(dir))?;
            }

            // Initialize machine-id (empty file tells systemd to generate one on first boot)
            let machine_id_path = target_root.join("etc/machine-id");
            if !machine_id_path.exists() {
                std::fs::write(&machine_id_path, "")?;
                tracing::info!("Created empty machine-id file (will be generated on first boot)");
            }

            // Create a symlink for journald runtime directory
            let journald_run_dir = target_root.join("var/log/journal");
            std::fs::create_dir_all(&journald_run_dir)?;

            // Enable basic systemd services by creating symlinks
            // multi-user.target is the default target
            let default_target_link = target_root.join("etc/systemd/system/default.target");
            if !default_target_link.exists() {
                std::os::unix::fs::symlink(
                    "/usr/lib/systemd/system/multi-user.target",
                    &default_target_link,
                )?;
                tracing::info!("Set default target to multi-user.target");
            }

            Ok(())
        }
        InitSystem::OpenRC => {
            tracing::info!("Configuring OpenRC...");
            // Create OpenRC directories
            let openrc_dirs = [
                "etc/runlevels/boot",
                "etc/runlevels/default",
                "etc/runlevels/shutdown",
                "etc/runlevels/sysinit",
            ];
            for dir in &openrc_dirs {
                std::fs::create_dir_all(target_root.join(dir))?;
            }
            Ok(())
        }
        InitSystem::Runit => {
            tracing::info!("Configuring runit...");
            let runit_dirs = ["etc/runit/runsvdir/default", "var/service"];
            for dir in &runit_dirs {
                std::fs::create_dir_all(target_root.join(dir))?;
            }
            Ok(())
        }
        InitSystem::S6 => {
            tracing::info!("Configuring s6...");
            let s6_dirs = ["etc/s6/sv", "etc/s6/rc"];
            for dir in &s6_dirs {
                std::fs::create_dir_all(target_root.join(dir))?;
            }
            Ok(())
        }
        InitSystem::SysVinit => {
            tracing::info!("Configuring SysVinit...");
            let sysvinit_dirs = [
                "etc/init.d",
                "etc/rc0.d",
                "etc/rc1.d",
                "etc/rc2.d",
                "etc/rc3.d",
                "etc/rc4.d",
                "etc/rc5.d",
                "etc/rc6.d",
            ];
            for dir in &sysvinit_dirs {
                std::fs::create_dir_all(target_root.join(dir))?;
            }
            Ok(())
        }
        InitSystem::Dinit => {
            tracing::info!("Configuring dinit...");
            let dinit_dirs = ["etc/dinit.d"];
            for dir in &dinit_dirs {
                std::fs::create_dir_all(target_root.join(dir))?;
            }
            Ok(())
        }
        InitSystem::BusyBoxInit => {
            tracing::info!("Configuring BusyBox init...");
            // BusyBox init uses /etc/inittab
            let inittab_path = target_root.join("etc/inittab");
            if !inittab_path.exists() {
                let inittab_content = "\
# /etc/inittab for BusyBox init

::sysinit:/etc/init.d/rcS
::respawn:/sbin/getty 38400 tty1
::ctrlaltdel:/sbin/reboot
::shutdown:/sbin/swapoff -a
::shutdown:/bin/umount -a -r
::restart:/sbin/init
";
                std::fs::write(&inittab_path, inittab_content)?;
            }
            Ok(())
        }
    }
}

/// Set up chroot bind mounts for bootloader installation
fn setup_chroot_mounts(
    target_root: &std::path::Path,
) -> anyhow::Result<Vec<(&'static str, &'static str)>> {
    let bind_mounts = vec![("/dev", "dev"), ("/proc", "proc"), ("/sys", "sys")];

    for (source, target) in &bind_mounts {
        let target_path = target_root.join(target);
        std::fs::create_dir_all(&target_path)?;

        let output = Command::new("mount")
            .args(["--bind", source, target_path.to_str().unwrap()])
            .output()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to bind mount {} to {}: {}",
                    source,
                    target_path.display(),
                    e
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to bind mount {}: {}", source, stderr);
        } else {
            tracing::info!("Bind mounted {} to {}", source, target_path.display());
        }
    }

    // Mount /run if it exists
    if PathBuf::from("/run").exists() {
        let run_target = target_root.join("run");
        std::fs::create_dir_all(&run_target)?;
        let _ = Command::new("mount")
            .args(["--bind", "/run", run_target.to_str().unwrap()])
            .output();
    }

    Ok(bind_mounts)
}

/// Clean up chroot bind mounts
fn cleanup_chroot_mounts(target_root: &std::path::Path, bind_mounts: &[(&str, &str)]) {
    // Unmount /run if we mounted it
    if PathBuf::from("/run").exists() {
        let run_target = target_root.join("run");
        let _ = Command::new("umount").arg(&run_target).output();
    }

    for (_, target) in bind_mounts.iter().rev() {
        let target_path = target_root.join(target);
        let output = Command::new("umount").arg(&target_path).output();

        match output {
            Ok(out) if out.status.success() => {
                tracing::info!("Unmounted {}", target_path.display());
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                tracing::warn!("Failed to unmount {}: {}", target_path.display(), stderr);
                // Try lazy unmount
                let _ = Command::new("umount")
                    .args(["-l", target_path.to_str().unwrap()])
                    .output();
            }
            Err(e) => {
                tracing::warn!("Error unmounting {}: {}", target_path.display(), e);
            }
        }
    }
}

/// Detect kernel version from installed modules
fn detect_kernel_version(target_root: &std::path::Path) -> String {
    let modules_dir = target_root.join("lib/modules");
    std::fs::read_dir(&modules_dir)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
        })
        .or_else(|| {
            let boot_dir = target_root.join("boot");
            std::fs::read_dir(&boot_dir).ok().and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.file_name().to_string_lossy().starts_with("vmlinuz-"))
                    .and_then(|e| {
                        let name = e.file_name();
                        let name_str = name.to_string_lossy();
                        name_str.strip_prefix("vmlinuz-").map(|v| v.to_string())
                    })
            })
        })
        .unwrap_or_else(|| "6.12.6".to_string())
}

/// Rename kernel files to include version suffix
fn rename_kernel_files(target_root: &std::path::Path, kernel_version: &str) -> anyhow::Result<()> {
    let boot_dir = target_root.join("boot");

    let vmlinuz_path = boot_dir.join("vmlinuz");
    let vmlinuz_versioned = boot_dir.join(format!("vmlinuz-{}", kernel_version));
    if vmlinuz_path.exists() && !vmlinuz_versioned.exists() {
        std::fs::rename(&vmlinuz_path, &vmlinuz_versioned).map_err(|e| {
            anyhow::anyhow!(
                "Failed to rename vmlinuz to {}: {}",
                vmlinuz_versioned.display(),
                e
            )
        })?;
        tracing::info!("Renamed vmlinuz to vmlinuz-{}", kernel_version);
    }

    let system_map_path = boot_dir.join("System.map");
    let system_map_versioned = boot_dir.join(format!("System.map-{}", kernel_version));
    if system_map_path.exists() && !system_map_versioned.exists() {
        std::fs::rename(&system_map_path, &system_map_versioned).map_err(|e| {
            anyhow::anyhow!(
                "Failed to rename System.map to {}: {}",
                system_map_versioned.display(),
                e
            )
        })?;
        tracing::info!("Renamed System.map to System.map-{}", kernel_version);
    }

    Ok(())
}

/// Set up PAM configuration
fn setup_pam_config(target_root: &std::path::Path) -> anyhow::Result<()> {
    let pam_d_path = target_root.join("etc/pam.d");
    std::fs::create_dir_all(&pam_d_path)?;

    let system_auth_path = pam_d_path.join("system-auth");
    if !system_auth_path.exists() {
        let system_auth_content = "#%PAM-1.0
# System-wide authentication configuration

# Authentication
auth       required   pam_unix.so     try_first_pass nullok
auth       optional   pam_permit.so

# Account management
account    required   pam_unix.so
account    optional   pam_permit.so

# Password management
password   required   pam_unix.so     try_first_pass nullok sha512
password   optional   pam_permit.so

# Session management
session    required   pam_unix.so
session    optional   pam_permit.so
";
        std::fs::write(&system_auth_path, system_auth_content)?;
        tracing::info!("Created /etc/pam.d/system-auth");
    }

    Ok(())
}

/// Set password for a user using chpasswd
fn set_user_password(
    target_root: &std::path::Path,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let passwd_cmd = format!("{}:{}", username, password);
    let output = Command::new("chroot")
        .arg(target_root)
        .env_clear()
        .env(
            "PATH",
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        )
        .env("HOME", "/root")
        .env("TERM", "linux")
        .arg("/usr/sbin/chpasswd")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(passwd_cmd.as_bytes())?;
            }
            child.wait_with_output()
        })
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute chroot/chpasswd command for {}: {}",
                username,
                e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to set password for {}: {}", username, stderr);
    }

    Ok(())
}

/// Create a user account
fn create_user_account(
    target_root: &std::path::Path,
    username: &str,
    full_name: &str,
    shell: &str,
    is_admin: bool,
) -> anyhow::Result<()> {
    let mut useradd_cmd = Command::new("chroot");
    useradd_cmd
        .arg(target_root)
        .env_clear()
        .env(
            "PATH",
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        )
        .env("HOME", "/root")
        .env("TERM", "linux")
        .arg("useradd")
        .arg("-m")
        .arg("-s")
        .arg(shell);

    if !full_name.is_empty() {
        useradd_cmd.arg("-c").arg(full_name);
    }

    if is_admin {
        useradd_cmd.arg("-G").arg("wheel,sudo");
    }

    useradd_cmd.arg(username);

    let output = useradd_cmd.output().map_err(|e| {
        anyhow::anyhow!(
            "Failed to execute chroot/useradd command for user {}: {}",
            username,
            e
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create user {}: {}", username, stderr);
    }

    Ok(())
}

/// Parse buck2 output to extract progress information
/// Returns a progress value between 0.0 and 1.0 if progress info is found
pub fn parse_buck2_progress(line: &str) -> Option<f32> {
    // Buck2 with --console simple outputs lines like:
    // "[timestamp] Waiting on root//packages/linux/boot/grub:grub (...) -- action (ebuild grub) [local_execute], and 1 other actions"
    // "[timestamp] BUILD SUCCEEDED"

    // Try to parse "Waiting on" lines with action counts
    if line.contains("Waiting on") {
        // Extract the "X other actions" count if present
        if let Some(and_pos) = line.find(", and ") {
            let after_and = &line[and_pos + 6..];
            if let Some(space_pos) = after_and.find(' ') {
                let count_str = &after_and[..space_pos];
                if let Ok(remaining) = count_str.parse::<u32>() {
                    // More actions remaining = earlier in build
                    // Use logarithmic scale: 1-2 actions = ~90%, 10 actions = ~70%, 100 actions = ~50%
                    let progress = if remaining == 0 {
                        0.95
                    } else if remaining == 1 {
                        0.90
                    } else {
                        let log_progress = 1.0 - ((remaining as f32).log10() / 2.5);
                        log_progress.max(0.1).min(0.85)
                    };
                    return Some(progress);
                }
            }
        } else {
            // No "X other actions", so we're near the end (just one action remaining)
            return Some(0.95);
        }
    }

    // Try to parse "BUILD SUCCEEDED" or "BUILD FAILED"
    if line.contains("BUILD SUCCEEDED") || line.contains("BUILD FAILED") {
        return Some(1.0);
    }

    // Try to parse "Jobs completed: X" format (older buck2 versions)
    if line.contains("Jobs completed:") {
        if let Some(start) = line.find("Jobs completed:") {
            let after = &line[start + 15..].trim();
            if let Some(end) = after.find('.') {
                let num_str = &after[..end].trim();
                if let Ok(completed) = num_str.parse::<u32>() {
                    // Estimate progress based on number of jobs
                    // We don't know the total, so use a logarithmic scale
                    let progress = (completed as f32).log10() / 3.0; // Assumes ~1000 max jobs
                    return Some(progress.min(0.95)); // Cap at 95%
                }
            }
        }
    }

    // Try to parse "[X/Y]" or "(X/Y)" format
    if let Some(bracket_start) = line.rfind('[').or_else(|| line.rfind('(')) {
        let bracket_end = if line[bracket_start..].starts_with('[') {
            line[bracket_start..].find(']')
        } else {
            line[bracket_start..].find(')')
        };

        if let Some(end) = bracket_end {
            let bracket_content = &line[bracket_start + 1..bracket_start + end];
            if let Some(slash_pos) = bracket_content.find('/') {
                let current_str = bracket_content[..slash_pos].trim();
                let total_str = bracket_content[slash_pos + 1..].trim();

                if let (Ok(current), Ok(total)) =
                    (current_str.parse::<u32>(), total_str.parse::<u32>())
                {
                    if total > 0 {
                        return Some((current as f32) / (total as f32));
                    }
                }
            }
        }
    }

    None
}

/// Run the installation process in the background
pub fn run_installation(config: InstallConfig, progress: Arc<Mutex<InstallProgress>>) {
    use anyhow::Result;

    // Helper to update progress and log
    let update_progress = |operation: &str, overall: f32, step: f32, log_msg: &str| {
        if let Ok(mut p) = progress.lock() {
            p.update(operation, overall, step);
            p.add_log(log_msg);
        }
    };

    // Helper to log error
    let log_error = |error_msg: &str| {
        if let Ok(mut p) = progress.lock() {
            p.add_error(error_msg);
            p.add_log(format!("ERROR: {}", error_msg));
        }
    };

    // Wrapper to run installation steps
    let run_step = || -> Result<()> {
        // Ensure PATH includes system directories for commands like parted, wipefs, etc.
        let current_path = std::env::var("PATH").unwrap_or_default();
        let system_paths = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
        let new_path = if current_path.is_empty() {
            system_paths.to_string()
        } else {
            format!("{}:{}", system_paths, current_path)
        };
        std::env::set_var("PATH", new_path);

        // Step 1: Pre-installation checks (2%)
        update_progress(
            "Pre-installation checks",
            0.01,
            0.0,
            "Starting installation...",
        );
        update_progress(
            "Pre-installation checks",
            0.01,
            0.5,
            "Checking system requirements...",
        );

        if !system::is_root() {
            anyhow::bail!("Installation must be run as root");
        }

        update_progress(
            "Pre-installation checks",
            0.02,
            1.0,
            "✓ Pre-installation checks complete",
        );

        // Step 2: Disk partitioning (2-5%)
        update_progress(
            "Disk partitioning",
            0.02,
            0.0,
            "Preparing disk partitioning...",
        );

        if let Some(disk_config) = &config.disk {
            // Safety check: Prevent installing on the disk containing the running system
            update_progress("Disk partitioning", 0.02, 0.05, "Checking disk safety...");
            check_disk_safety(&disk_config.device)?;

            update_progress(
                "Disk partitioning",
                0.025,
                0.1,
                format!("Preparing disk: {}", disk_config.device).as_str(),
            );

            // Unmount any partitions from the target disk
            update_progress(
                "Disk partitioning",
                0.03,
                0.2,
                "Unmounting existing partitions...",
            );
            unmount_disk_partitions(&disk_config.device)?;

            // Deactivate any swap on the target disk
            update_progress("Disk partitioning", 0.032, 0.4, "Deactivating swap...");
            deactivate_swap(&disk_config.device);

            // Check if any processes are still using the disk
            update_progress(
                "Disk partitioning",
                0.033,
                0.45,
                "Checking for processes using disk...",
            );
            kill_disk_processes(&disk_config.device);

            // Clean up device-mapper entries (LVM, dm-crypt) that may be holding the disk
            update_progress(
                "Disk partitioning",
                0.034,
                0.45,
                "Cleaning up device-mapper entries...",
            );
            cleanup_device_mapper(&disk_config.device);

            // Prepare disk by flushing buffers and settling udev
            prepare_disk_for_partitioning(&disk_config.device);

            tracing::info!("Starting disk partitioning on {}", disk_config.device);
            update_progress(
                "Disk partitioning",
                0.035,
                0.3,
                format!("Partitioning disk: {}", disk_config.device).as_str(),
            );

            if disk_config.wipe_disk {
                tracing::info!("Wiping disk signatures with wipefs");
                update_progress(
                    "Disk partitioning",
                    0.04,
                    0.5,
                    format!("Wiping disk: {}", disk_config.device).as_str(),
                );

                // Wipe partition table signatures (optional, parted mklabel will also clear the table)
                match Command::new("wipefs")
                    .args(["-a", &disk_config.device])
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            tracing::info!("wipefs completed successfully");
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            tracing::warn!("wipefs failed: {}, continuing anyway", stderr);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("wipefs not available: {}, skipping (not critical)", e);
                    }
                }
            }

            // Create partition table
            let pt_type = if disk_config.use_gpt { "gpt" } else { "msdos" };
            update_progress(
                "Disk partitioning",
                0.045,
                0.7,
                format!("Creating {} partition table", pt_type).as_str(),
            );
            create_partition_table(&disk_config.device, disk_config.use_gpt)?;

            // Create partitions
            let mut start_mb: u64 = 1; // Start at 1MB to align partitions
            for (idx, partition) in disk_config.partitions.iter().enumerate() {
                let step = 0.7 + ((idx as f32 + 1.0) / disk_config.partitions.len() as f32) * 0.3;
                update_progress(
                    "Disk partitioning",
                    0.045 + (step * 0.005),
                    step,
                    format!("Creating partition {}", partition.device).as_str(),
                );
                start_mb = create_partition(
                    &disk_config.device,
                    partition,
                    idx,
                    start_mb,
                    disk_config.use_gpt,
                )?;
            }

            // Inform kernel of partition table changes
            update_progress(
                "Disk partitioning",
                0.148,
                0.95,
                "Updating partition table...",
            );
            let partprobe_result = Command::new("partprobe").arg(&disk_config.device).output();

            if let Err(e) = partprobe_result {
                // partprobe might not be available, try alternative
                tracing::warn!("partprobe failed: {}, trying blockdev --rereadpt", e);
                let _ = Command::new("blockdev")
                    .args(["--rereadpt", &disk_config.device])
                    .output();
            }

            // Wait a moment for devices to appear
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        update_progress(
            "Disk partitioning",
            0.05,
            1.0,
            "✓ Disk partitioning complete",
        );

        // Step 3: Filesystem creation (5-7%)
        update_progress("Filesystem creation", 0.05, 0.0, "Creating filesystems...");

        if let Some(disk_config) = &config.disk {
            for (idx, partition) in disk_config.partitions.iter().enumerate() {
                let step = (idx as f32 + 1.0) / disk_config.partitions.len() as f32;

                if partition.format {
                    update_progress(
                        "Filesystem creation",
                        0.05 + (step * 0.02),
                        step,
                        format!(
                            "Creating {} on {}",
                            partition.filesystem.as_str(),
                            partition.device
                        )
                        .as_str(),
                    );
                    format_partition(partition)?;
                }
            }
        }

        update_progress(
            "Filesystem creation",
            0.07,
            1.0,
            "✓ Filesystem creation complete",
        );

        // Step 4: Mounting filesystems (7-10%)
        update_progress(
            "Mounting filesystems",
            0.07,
            0.0,
            "Preparing to mount filesystems...",
        );

        if let Some(disk_config) = &config.disk {
            // Sort partitions by mount order (root first, then nested mounts)
            let mut sorted_partitions: Vec<&PartitionConfig> =
                disk_config.partitions.iter().collect();
            sorted_partitions.sort_by(|a, b| {
                let a_path = a.mount_point.path();
                let b_path = b.mount_point.path();

                // Swap goes last
                if a_path == "swap" {
                    return std::cmp::Ordering::Greater;
                }
                if b_path == "swap" {
                    return std::cmp::Ordering::Less;
                }

                // Root goes first
                if a_path == "/" {
                    return std::cmp::Ordering::Less;
                }
                if b_path == "/" {
                    return std::cmp::Ordering::Greater;
                }

                // Sort by path depth (shorter paths first)
                a_path
                    .matches('/')
                    .count()
                    .cmp(&b_path.matches('/').count())
                    .then_with(|| a_path.cmp(b_path))
            });

            for (idx, partition) in sorted_partitions.iter().enumerate() {
                let step = (idx as f32 + 1.0) / sorted_partitions.len() as f32;
                let mount_path = partition.mount_point.path();

                update_progress(
                    "Mounting filesystems",
                    0.07 + (step * 0.03),
                    step,
                    format!("Mounting {} to {}", partition.device, mount_path).as_str(),
                );
                mount_partition(partition, &config.target_root)?;
            }

            // Handle btrfs subvolumes for BtrfsSubvolumes layout
            if config.disk_layout == crate::types::DiskLayoutPreset::BtrfsSubvolumes {
                update_progress(
                    "Mounting filesystems",
                    0.09,
                    0.8,
                    "Creating btrfs subvolumes...",
                );

                // Find the root partition
                if let Some(root_part) = disk_config
                    .partitions
                    .iter()
                    .find(|p| p.mount_point == crate::types::MountPoint::Root)
                {
                    if root_part.filesystem == FilesystemType::Btrfs {
                        // Create subvolumes: @, @home, @snapshots
                        let btrfs_root = &config.target_root;

                        for subvol in &["@home", "@snapshots"] {
                            let output = Command::new("btrfs")
                                .args(["subvolume", "create"])
                                .arg(btrfs_root.join(subvol.trim_start_matches('@')))
                                .output()
                                .map_err(|e| anyhow::anyhow!(
                                    "Failed to execute btrfs command for subvolume {}: {}. Make sure btrfs-progs is installed.",
                                    subvol, e
                                ))?;

                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                update_progress(
                                    "Mounting filesystems",
                                    0.09,
                                    0.9,
                                    format!(
                                        "Warning: Failed to create subvolume {}: {}",
                                        subvol, stderr
                                    )
                                    .as_str(),
                                );
                            }
                        }
                    }
                }
            }
        }

        update_progress("Mounting filesystems", 0.10, 1.0, "✓ Filesystems mounted");

        // Step 5: Build rootfs with Buck2 (10-85% - this is the longest step)
        update_progress(
            "Building rootfs",
            0.10,
            0.0,
            "Generating USE flags config...",
        );

        // Generate USE flags configuration based on all installation options
        let config_dir = config.buckos_build_path.join("config");
        std::fs::create_dir_all(&config_dir)?;
        let use_config_path = config_dir.join("use_config.bzl");

        // Build USE flags from installer configuration
        let use_flags = UseFlags::from_config(&config);

        // Write Starlark config for Buck2 build
        std::fs::write(&use_config_path, use_flags.to_starlark())?;
        tracing::info!(
            "Generated USE flags config with {} flags, {} video cards, {} input devices",
            use_flags.global.len(),
            use_flags.video_cards.len(),
            use_flags.input_devices.len()
        );

        update_progress(
            "Building rootfs",
            0.10,
            0.0,
            "Generating custom rootfs target...",
        );

        // Generate a custom BUCK file with rootfs based on user selections
        // Create an install directory for the dynamic rootfs target
        let install_dir = config.buckos_build_path.join("install");
        std::fs::create_dir_all(&install_dir)?;
        let install_buck_path = install_dir.join("BUCK");
        let mut rootfs_packages = Vec::new();

        // Add system packages
        rootfs_packages.push("\"//packages/linux/system/apps:coreutils\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/util-linux:util-linux\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/procps-ng:procps-ng\"".to_string());
        rootfs_packages.push("\"//packages/linux/system/apps/shadow:shadow\"".to_string());
        rootfs_packages.push("\"//packages/linux/system/security/auth/pam:pam\"".to_string());
        // Add PAM dependencies explicitly (needed for pam_unix.so to load)
        rootfs_packages.push("\"//packages/linux/system/libs/network/libnsl:libnsl\"".to_string());
        rootfs_packages.push("\"//packages/linux/system/libs/ipc/libtirpc:libtirpc\"".to_string());
        rootfs_packages
            .push("\"//packages/linux/system/libs/crypto/libxcrypt:libxcrypt\"".to_string()); // libcrypt.so.2 for password hashing
        rootfs_packages.push("\"//packages/linux/core/file:file\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/bash:bash\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/zlib:zlib\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/xz:xz\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/glibc:glibc\"".to_string());

        // Add Linux kernel (user-selected channel)
        rootfs_packages.push(config.kernel_channel.package_target().to_string());

        // Add linux-firmware for hardware driver support
        rootfs_packages
            .push("\"//packages/linux/system/firmware/linux-firmware:linux-firmware\"".to_string());

        // Add dracut for initramfs generation
        rootfs_packages.push("\"//packages/linux/system/initramfs/dracut:dracut\"".to_string());

        // Add dependencies required for dracut initramfs generation
        rootfs_packages.push("\"//packages/linux/system/libs/cpio:cpio\"".to_string()); // Required for creating cpio archives
        rootfs_packages.push("\"//packages/linux/system/libs/compression/lz4:lz4\"".to_string()); // Compression
        rootfs_packages.push("\"//packages/linux/system/security/audit:audit\"".to_string()); // libaudit (pulls in libcap-ng)

        // Add GRUB bootloader based on system type (EFI or BIOS)
        // Note: xz is automatically included as a dependency of GRUB
        let is_efi = system::is_efi_system();
        if is_efi {
            rootfs_packages.push("\"//packages/linux/boot/grub:grub\"".to_string());
            // efibootmgr is required for EFI systems to manage boot entries
            rootfs_packages.push("\"//packages/linux/boot/efibootmgr:efibootmgr\"".to_string());
        } else {
            rootfs_packages.push("\"//packages/linux/boot/grub:grub-bios\"".to_string());
        }

        // Add init system
        let init_target = match config.init_system {
            crate::types::InitSystem::Systemd => "\"//packages/linux/system/init:systemd\"",
            crate::types::InitSystem::OpenRC => "\"//packages/linux/system/init:openrc\"",
            crate::types::InitSystem::Runit => "\"//packages/linux/system/init:runit\"",
            crate::types::InitSystem::S6 => "\"//packages/linux/system/init:s6\"",
            crate::types::InitSystem::SysVinit => "\"//packages/linux/system/init:sysvinit\"",
            crate::types::InitSystem::Dinit => "\"//packages/linux/system/init:dinit\"",
            crate::types::InitSystem::BusyBoxInit => "\"//packages/linux/core:busybox\"",
        };
        rootfs_packages.push(init_target.to_string());

        // Add networking basics
        rootfs_packages.push("\"//packages/linux/network:openssl\"".to_string());
        rootfs_packages.push("\"//packages/linux/network:curl\"".to_string());
        rootfs_packages.push("\"//packages/linux/network:iproute2\"".to_string());

        // Add profile-specific packages based on selection
        match &config.profile {
            crate::types::InstallProfile::Minimal => {
                // Minimal already has system packages
            }
            crate::types::InstallProfile::Server => {
                rootfs_packages.push("\"//packages/linux/network:openssh\"".to_string());
                rootfs_packages.push("\"//packages/linux/editors:vim\"".to_string());
            }
            crate::types::InstallProfile::Desktop(_) => {
                rootfs_packages.push("\"//packages/linux/network:openssh\"".to_string());
                rootfs_packages.push("\"//packages/linux/editors:vim\"".to_string());
                // Desktop packages would be added here when available
            }
            crate::types::InstallProfile::Handheld(_) => {
                rootfs_packages.push("\"//packages/linux/network:openssh\"".to_string());
                rootfs_packages.push("\"//packages/linux/editors:vim\"".to_string());
                // Gaming-specific packages would be added here
            }
            crate::types::InstallProfile::Custom => {
                // Custom profile - user will select packages manually
            }
        }

        // Generate BUCK file content
        let buck_content = format!(
            r#"load("//defs:package_defs.bzl", "rootfs")

rootfs(
    name = "installer-rootfs",
    packages = [
        {},
    ],
    visibility = ["PUBLIC"],
)
"#,
            rootfs_packages.join(",\n        ")
        );

        update_progress(
            "Building rootfs",
            0.11,
            0.05,
            "Writing custom BUCK target...",
        );
        std::fs::write(&install_buck_path, buck_content)?;
        tracing::info!(
            "Generated installer BUCK file at: {}",
            install_buck_path.display()
        );

        // Write kernel config fragment if hardware-specific config was generated
        if let Some(ref kernel_config) = config.kernel_config_fragment {
            let kernel_config_path = config.buckos_build_path.join("hardware-kernel.config");
            std::fs::write(&kernel_config_path, kernel_config)?;
            tracing::info!(
                "Wrote hardware-specific kernel config to: {}",
                kernel_config_path.display()
            );
            update_progress(
                "Building rootfs",
                0.115,
                0.06,
                "Saved hardware-specific kernel config",
            );
        }

        // Check if Buck2 cache exists (for live CD installations with pre-built packages)
        let buck_out_path = config.buckos_build_path.join("buck-out");
        if buck_out_path.exists() {
            tracing::info!(
                "Buck2 cache directory found at: {}",
                buck_out_path.display()
            );
            update_progress(
                "Building rootfs",
                0.12,
                0.08,
                "✓ Found Buck2 cache (using pre-built packages)",
            );
        } else {
            tracing::info!("No Buck2 cache found, will build packages from scratch");
            update_progress(
                "Building rootfs",
                0.12,
                0.08,
                "Building packages from scratch (no cache found)",
            );
        }

        // Build the rootfs with Buck2
        // Buck2 will automatically use cached artifacts from buck-out if available
        update_progress("Building rootfs", 0.15, 0.1, "Running buck2 build...");

        // Buck2 refuses to run as root, so we need to run it as the original user
        // who invoked the installer (typically via sudo)
        let mut buck2_cmd = if system::is_root() {
            // Get the original user from SUDO_USER environment variable
            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                tracing::info!("Running buck2 as user: {}", sudo_user);

                // Run buck2 as the original user using sudo -u
                let mut cmd = Command::new("sudo");
                cmd.arg("-u")
                    .arg(&sudo_user)
                    .arg("buck2")
                    .arg("build")
                    .arg("//install:installer-rootfs")
                    .arg("--target-platforms")
                    .arg("//platforms:default")
                    .arg("--console")
                    .arg("simple")
                    .current_dir(&config.buckos_build_path)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
                cmd
            } else {
                // No SUDO_USER found, try running as root anyway (may fail)
                tracing::warn!("SUDO_USER not found, attempting to run buck2 as root (may fail)");
                let mut cmd = Command::new("buck2");
                cmd.arg("build")
                    .arg("//install:installer-rootfs")
                    .arg("--target-platforms")
                    .arg("//platforms:default")
                    .arg("--console")
                    .arg("simple")
                    .current_dir(&config.buckos_build_path)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
                cmd
            }
        } else {
            // Not running as root, execute buck2 directly
            let mut cmd = Command::new("buck2");
            cmd.arg("build")
                .arg("//install:installer-rootfs")
                .arg("--target-platforms")
                .arg("//platforms:default")
                .arg("--console")
                .arg("simple")
                .current_dir(&config.buckos_build_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            cmd
        };

        let mut child = buck2_cmd.spawn().map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute buck2 command: {}. Make sure buck2 is installed and in PATH.",
                e
            )
        })?;

        // Capture and process buck2 output in real-time
        use std::io::{BufRead, BufReader};

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture buck2 stderr"))?;
        let reader = BufReader::new(stderr);

        let mut last_progress_update = std::time::Instant::now();
        let mut accumulated_output = String::new();

        for line in reader.lines() {
            let line = line?;
            accumulated_output.push_str(&line);
            accumulated_output.push('\n');

            // Parse buck2 progress from the line
            // Buck2 outputs lines like "Jobs completed: 5. Time elapsed: 1.2s."
            // or "Action: xyz [1/100]"
            if let Some(progress_info) = parse_buck2_progress(&line) {
                let elapsed = last_progress_update.elapsed();
                // Throttle updates to avoid overwhelming the UI (update at most every 100ms)
                if elapsed.as_millis() > 100 {
                    // Map buck2 progress (0.0-1.0) to our step progress (0.1-0.8)
                    let step_progress = 0.1 + (progress_info * 0.7);
                    update_progress(
                        "Building rootfs",
                        0.15 + (progress_info * 0.60),
                        step_progress,
                        &format!("Building: {}", line),
                    );
                    last_progress_update = std::time::Instant::now();
                }
            }

            // Log the output for debugging
            tracing::debug!("buck2: {}", line);
        }

        let output = child
            .wait_with_output()
            .map_err(|e| anyhow::anyhow!("Failed to wait for buck2 process: {}", e))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "Failed to build rootfs with buck2:\nstdout: {}\nstderr: {}",
                stdout,
                accumulated_output
            );
        }

        update_progress("Building rootfs", 0.75, 0.8, "✓ Rootfs built successfully");

        // Find the built rootfs directory
        update_progress("Building rootfs", 0.77, 0.85, "Locating built rootfs...");

        // Run buck2 as the original user if we're root (same approach as above)
        let mut show_output_cmd = if system::is_root() {
            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                let mut cmd = Command::new("sudo");
                cmd.arg("-u")
                    .arg(&sudo_user)
                    .arg("buck2")
                    .arg("build")
                    .arg("//install:installer-rootfs")
                    .arg("--show-output")
                    .arg("--target-platforms")
                    .arg("//platforms:default")
                    .current_dir(&config.buckos_build_path);
                cmd
            } else {
                let mut cmd = Command::new("buck2");
                cmd.arg("build")
                    .arg("//install:installer-rootfs")
                    .arg("--show-output")
                    .arg("--target-platforms")
                    .arg("//platforms:default")
                    .current_dir(&config.buckos_build_path);
                cmd
            }
        } else {
            let mut cmd = Command::new("buck2");
            cmd.arg("build")
                .arg("//install:installer-rootfs")
                .arg("--show-output")
                .arg("--target-platforms")
                .arg("//platforms:default")
                .current_dir(&config.buckos_build_path);
            cmd
        };

        let show_output = show_output_cmd
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to get buck2 output path: {}", e))?;

        let output_str = String::from_utf8_lossy(&show_output.stdout);
        let stderr_str = String::from_utf8_lossy(&show_output.stderr);
        tracing::debug!("buck2 --show-output stdout: {:?}", output_str);
        tracing::debug!("buck2 --show-output stderr: {:?}", stderr_str);
        tracing::debug!("buck2 --show-output exit status: {:?}", show_output.status);
        let rootfs_path = output_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .next_back()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to parse buck2 output path. stdout={:?}, stderr={:?}, status={:?}",
                    output_str,
                    stderr_str,
                    show_output.status
                )
            })?;

        tracing::info!("Built rootfs at: {}", rootfs_path);

        // Copy the rootfs to target
        update_progress(
            "Building rootfs",
            0.80,
            0.9,
            "Extracting rootfs to target...",
        );

        // Buck2 returns a relative path from buckos_build_path, so make it absolute
        let rootfs_src = config.buckos_build_path.join(rootfs_path);
        if rootfs_src.is_dir() {
            // Copy directory contents (not the directory itself)
            // Use rsync or cp with /* to copy contents
            let rootfs_path_with_contents = format!("{}/*", rootfs_src.display());
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "cp -a {} {}",
                    rootfs_path_with_contents,
                    config.target_root.display()
                ))
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to copy rootfs: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to copy rootfs to target: {}", stderr);
            }
        } else {
            anyhow::bail!(
                "Expected rootfs directory at {}, but it doesn't exist or is not a directory",
                rootfs_src.display()
            );
        }

        update_progress(
            "Building rootfs",
            0.82,
            1.0,
            "✓ Rootfs installation complete",
        );

        // Step 5.5: Install buckos-build repo and buckos binary (82-85%)
        update_progress(
            "Installing package repo",
            0.82,
            0.0,
            "Setting up BuckOS package repository...",
        );

        let target_repo_path = config.target_root.join("var/db/repos/buckos-build");
        std::fs::create_dir_all(&target_repo_path)?;

        // Check if we have a local buckos-build repo to copy, otherwise clone from GitHub
        let source_repo_exists =
            config.buckos_build_path.exists() && config.buckos_build_path.join(".git").exists();

        if source_repo_exists {
            // Copy the local buckos-build repo to the target
            update_progress(
                "Installing package repo",
                0.82,
                0.2,
                "Copying buckos-build repository...",
            );
            tracing::info!(
                "Copying buckos-build from {} to {}",
                config.buckos_build_path.display(),
                target_repo_path.display()
            );

            // Use rsync for efficient copying, excluding buck-out
            let output = Command::new("rsync")
                .args([
                    "-a",
                    "--exclude=buck-out",
                    "--exclude=.git/objects/pack/*.pack",
                    &format!("{}/", config.buckos_build_path.display()),
                    &format!("{}/", target_repo_path.display()),
                ])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to copy buckos-build repo: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("rsync warning: {}", stderr);
                // Fall back to cp if rsync fails
                let output = Command::new("cp")
                    .args([
                        "-a",
                        config.buckos_build_path.to_string_lossy().as_ref(),
                        target_repo_path
                            .parent()
                            .unwrap()
                            .to_string_lossy()
                            .as_ref(),
                    ])
                    .output()?;
                if !output.status.success() {
                    anyhow::bail!(
                        "Failed to copy buckos-build repo: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
        } else {
            // Clone from GitHub
            update_progress(
                "Installing package repo",
                0.82,
                0.2,
                "Cloning buckos-build repository from GitHub...",
            );
            tracing::info!(
                "Cloning buckos-build from GitHub to {}",
                target_repo_path.display()
            );

            let output = Command::new("git")
                .args([
                    "clone",
                    "--depth=1",
                    "https://github.com/hodgesds/buckos-build.git",
                    target_repo_path.to_string_lossy().as_ref(),
                ])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to clone buckos-build repo: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    "Failed to clone buckos-build repo: {}. Package management will need manual setup.",
                    stderr
                );
            }
        }

        // Install BuckOS specifications to standard location
        update_progress(
            "Installing package repo",
            0.825,
            0.4,
            "Installing BuckOS specifications...",
        );

        let source_specs_path = config.buckos_build_path.join("specs");
        let target_specs_path = config.target_root.join("usr/share/buckos/specs");

        if source_specs_path.exists() {
            std::fs::create_dir_all(&target_specs_path)?;

            tracing::info!(
                "Copying BuckOS specifications from {} to {}",
                source_specs_path.display(),
                target_specs_path.display()
            );

            // Copy specs directory to target
            let output = Command::new("cp")
                .args([
                    "-a",
                    "-r",
                    &format!("{}/.", source_specs_path.display()),
                    target_specs_path.to_string_lossy().as_ref(),
                ])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to copy specs: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Failed to copy specs: {}", stderr);
            } else {
                tracing::info!("✓ BuckOS specifications installed to /usr/share/buckos/specs");
            }
        } else {
            tracing::warn!(
                "Specs directory not found at {}. Specifications will not be available.",
                source_specs_path.display()
            );
        }

        // Install buckos binary to the target system
        update_progress(
            "Installing package repo",
            0.83,
            0.5,
            "Installing buckos package manager...",
        );

        // Try to find the buckos binary - check common locations
        let buckos_binary_candidates = [
            PathBuf::from("/usr/bin/buckos"),
            PathBuf::from("/usr/local/bin/buckos"),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("buckos")))
                .unwrap_or_default(),
        ];

        let buckos_binary = buckos_binary_candidates.iter().find(|p| p.exists());

        if let Some(binary_path) = buckos_binary {
            let target_bin_dir = config.target_root.join("usr/bin");
            std::fs::create_dir_all(&target_bin_dir)?;
            let target_binary = target_bin_dir.join("buckos");

            std::fs::copy(binary_path, &target_binary)
                .map_err(|e| anyhow::anyhow!("Failed to install buckos binary: {}", e))?;

            // Make it executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&target_binary, std::fs::Permissions::from_mode(0o755))?;
            }
            tracing::info!("Installed buckos binary to {}", target_binary.display());
        } else {
            tracing::warn!("buckos binary not found. Package management will need manual setup.");
        }

        // Configure binary package mirror in .buckconfig
        update_progress(
            "Installing package repo",
            0.835,
            0.7,
            "Configuring binary package mirror...",
        );

        configure_binary_mirror(&target_repo_path)?;
        tracing::info!("Configured binary package mirror");

        update_progress(
            "Installing package repo",
            0.84,
            1.0,
            "✓ Package repository installed",
        );

        // Step 5.6: Build and install profile-specific packages (84-90%)
        let package_sets = config.profile.package_sets();
        let package_sets_to_build: Vec<&str> = package_sets
            .iter()
            .filter(|s| **s != "@system") // Skip @system, already in base rootfs
            .copied()
            .collect();

        if !package_sets_to_build.is_empty() {
            update_progress(
                "Installing profile packages",
                0.84,
                0.0,
                &format!(
                    "Building {} package sets for profile...",
                    package_sets_to_build.len()
                ),
            );

            let total_sets = package_sets_to_build.len();
            for (idx, package_set) in package_sets_to_build.iter().enumerate() {
                // Map package set name to Buck target in buckos-build
                let buck_target = match *package_set {
                    "@desktop" => Some("//packages/linux/desktop:desktop-foundation"),
                    "@audio" => Some("//packages/linux/audio:essential"),
                    "@network" => Some("//packages/linux/network:network-tools"),
                    "@server" => Some("//packages/linux/network:remote-access"),
                    "@gaming" => Some("//packages/linux/gaming:gaming"),
                    "@steam" => Some("//packages/linux/gaming:launchers"),
                    "@gnome" => Some("//packages/linux/desktop/gnome:gnome"),
                    "@kde" => Some("//packages/linux/desktop/kde:kde-plasma"),
                    "@xfce" => Some("//packages/linux/desktop/xfce:xfce"),
                    "@mate" => Some("//packages/linux/desktop/mate:mate"),
                    "@cinnamon" => Some("//packages/linux/desktop/cinnamon:cinnamon-desktop"),
                    "@lxqt" => Some("//packages/linux/desktop/lxqt:lxqt"),
                    "@i3" => Some("//packages/linux/desktop:i3-complete"),
                    "@sway" => Some("//packages/linux/desktop:sway-complete"),
                    "@hyprland" => Some("//packages/linux/desktop:hyprland-complete"),
                    "@xorg-minimal" => Some("//packages/linux/desktop/xorg-server:xorg-server"),
                    _ => {
                        tracing::warn!("Unknown package set: {}", package_set);
                        None
                    }
                };

                if let Some(target) = buck_target {
                    let progress = 0.84 + (0.05 * (idx as f32 / total_sets as f32));
                    update_progress(
                        "Installing profile packages",
                        progress,
                        idx as f32 / total_sets as f32,
                        &format!("Building {}...", package_set),
                    );

                    tracing::info!("Building package set {} ({})", package_set, target);

                    // Build the package set
                    let mut buck2_cmd = if system::is_root() {
                        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                            let mut cmd = Command::new("sudo");
                            cmd.arg("-u")
                                .arg(&sudo_user)
                                .arg("buck2")
                                .arg("build")
                                .arg(target)
                                .arg("--target-platforms")
                                .arg("//platforms:default")
                                .current_dir(&config.buckos_build_path);
                            cmd
                        } else {
                            let mut cmd = Command::new("buck2");
                            cmd.arg("build")
                                .arg(target)
                                .arg("--target-platforms")
                                .arg("//platforms:default")
                                .current_dir(&config.buckos_build_path);
                            cmd
                        }
                    } else {
                        let mut cmd = Command::new("buck2");
                        cmd.arg("build")
                            .arg(target)
                            .arg("--target-platforms")
                            .arg("//platforms:default")
                            .current_dir(&config.buckos_build_path);
                        cmd
                    };

                    let output = buck2_cmd.output();
                    match output {
                        Ok(out) if out.status.success() => {
                            // Get the output path and copy to target rootfs
                            let mut show_cmd = Command::new("buck2");
                            show_cmd
                                .arg("build")
                                .arg(target)
                                .arg("--show-output")
                                .arg("--target-platforms")
                                .arg("//platforms:default")
                                .current_dir(&config.buckos_build_path);

                            if let Ok(show_out) = show_cmd.output() {
                                let output_str = String::from_utf8_lossy(&show_out.stdout);
                                if let Some(pkg_path) = output_str
                                    .lines()
                                    .filter(|l| !l.trim().is_empty())
                                    .next_back()
                                    .and_then(|l| l.split_whitespace().nth(1))
                                {
                                    let pkg_src = config.buckos_build_path.join(pkg_path);
                                    if pkg_src.is_dir() {
                                        // Merge into target rootfs using shell for glob expansion
                                        let merge_output = Command::new("sh")
                                            .arg("-c")
                                            .arg(format!(
                                                "cp -a {}/* {}",
                                                pkg_src.display(),
                                                config.target_root.display()
                                            ))
                                            .output();

                                        match merge_output {
                                            Ok(out) if !out.status.success() => {
                                                let stderr = String::from_utf8_lossy(&out.stderr);
                                                tracing::warn!(
                                                    "Failed to merge {}: {}",
                                                    package_set,
                                                    stderr
                                                );
                                            }
                                            Err(e) => {
                                                tracing::warn!(
                                                    "Failed to merge {}: {}",
                                                    package_set,
                                                    e
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            tracing::info!("Successfully built {}", package_set);
                        }
                        Ok(out) => {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            tracing::warn!(
                                "Failed to build package set {}: {}. Skipping.",
                                package_set,
                                stderr
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to run buck2 for {}: {}. Skipping.",
                                package_set,
                                e
                            );
                        }
                    }
                }
            }

            update_progress(
                "Installing profile packages",
                0.90,
                1.0,
                "✓ Profile packages installed",
            );
        }

        // Step 6: System configuration (90-95%)
        update_progress("System configuration", 0.90, 0.0, "Configuring system...");

        // Create essential system files (/etc/passwd and /etc/shadow)
        // These are required before running dracut which uses grep on these files
        init_system_files(&config.target_root)?;

        // Generate fstab
        update_progress(
            "System configuration",
            0.85,
            0.1,
            "Generating /etc/fstab...",
        );
        if let Some(disk_config) = &config.disk {
            generate_fstab(&config.target_root, &disk_config.partitions)?;
            update_progress("System configuration", 0.86, 0.2, "✓ Generated /etc/fstab");
        }

        // Configure locale
        update_progress("System configuration", 0.86, 0.3, "Configuring locale...");
        configure_locale(&config.target_root, &config.locale.locale)?;
        update_progress("System configuration", 0.87, 0.4, "✓ Configured locale");

        // Configure timezone
        update_progress("System configuration", 0.87, 0.5, "Configuring timezone...");
        configure_timezone(&config.target_root, &config.timezone.timezone)?;
        update_progress("System configuration", 0.88, 0.6, "✓ Configured timezone");

        // Configure hostname
        update_progress("System configuration", 0.88, 0.7, "Configuring network...");
        configure_network(&config.target_root, &config.network.hostname)?;
        update_progress("System configuration", 0.89, 0.8, "✓ Configured network");

        // Configure keyboard layout
        update_progress("System configuration", 0.89, 0.9, "Configuring keyboard...");
        configure_keyboard(&config.target_root, &config.locale.keyboard)?;

        // Configure init system
        update_progress(
            "System configuration",
            0.90,
            0.95,
            "Configuring init system...",
        );
        configure_init_system(&config.target_root, &config.init_system)?;
        update_progress(
            "System configuration",
            0.91,
            1.0,
            "✓ Configured init system",
        );

        // Configure system limits (ulimits and sysctl)
        update_progress(
            "System configuration",
            0.915,
            0.0,
            "Configuring system limits...",
        );
        configure_system_limits(&config.target_root, &config.system_limits)?;
        if config.system_limits.apply_ulimits || config.system_limits.apply_sysctl {
            update_progress(
                "System configuration",
                0.92,
                1.0,
                "✓ Configured system limits",
            );
        }

        // Generate /etc/buckos/buckos.toml for package manager on target system
        let buckos_config_dir = config.target_root.join("etc/buckos");
        std::fs::create_dir_all(&buckos_config_dir)?;
        let buckos_toml_path = buckos_config_dir.join("buckos.toml");
        let target_use_flags = UseFlags::from_config(&config);
        std::fs::write(&buckos_toml_path, target_use_flags.to_toml())?;
        tracing::info!(
            "Generated /etc/buckos/buckos.toml with {} USE flags",
            target_use_flags.global.len()
        );

        // Enable serial console for systemd (useful for VMs and headless systems)
        if matches!(config.init_system, crate::types::InitSystem::Systemd) {
            let getty_target_dir = config
                .target_root
                .join("etc/systemd/system/getty.target.wants");
            std::fs::create_dir_all(&getty_target_dir)?;

            // Enable serial-getty@ttyS0.service
            let serial_getty_link = getty_target_dir.join("serial-getty@ttyS0.service");
            if !serial_getty_link.exists() {
                std::os::unix::fs::symlink(
                    "/usr/lib/systemd/system/serial-getty@.service",
                    &serial_getty_link,
                )?;
                tracing::info!("Enabled serial console on ttyS0");
            }
        }

        update_progress(
            "System configuration",
            0.90,
            1.0,
            "✓ System configuration complete",
        );

        // Step 7: Bootloader installation (90-95%)
        update_progress(
            "Installing bootloader",
            0.90,
            0.0,
            "Installing bootloader...",
        );

        let bootloader_name = match config.bootloader {
            crate::types::BootloaderType::Grub => "GRUB",
            crate::types::BootloaderType::Systemdboot => "systemd-boot",
            crate::types::BootloaderType::Refind => "rEFInd",
            crate::types::BootloaderType::Limine => "Limine",
            crate::types::BootloaderType::Efistub => "EFISTUB",
            crate::types::BootloaderType::None => "None",
        };

        if config.bootloader != crate::types::BootloaderType::None {
            update_progress(
                "Installing bootloader",
                0.905,
                0.1,
                format!("Installing {} bootloader...", bootloader_name).as_str(),
            );

            match config.bootloader {
                crate::types::BootloaderType::Grub => {
                    // Verify GRUB binaries exist in the target system
                    let grub_install_path = config.target_root.join("usr/sbin/grub-install");
                    let grub_mkconfig_path = config.target_root.join("usr/sbin/grub-mkconfig");

                    if !grub_install_path.exists() || !grub_mkconfig_path.exists() {
                        let warning_msg = format!(
                            "⚠ Skipping GRUB installation: Required GRUB binaries not found.\n\
                            Looking for:\n  {}\n  {}\n\
                            The @grub package may not be available in the repository yet.\n\
                            You will need to manually install and configure a bootloader.",
                            grub_install_path.display(),
                            grub_mkconfig_path.display()
                        );
                        update_progress("Installing bootloader", 0.95, 0.5, &warning_msg);
                        tracing::warn!("{}", warning_msg);
                    } else {
                        // GRUB binaries found, proceed with installation
                        tracing::info!("Found GRUB binaries, proceeding with installation");

                        // Find boot device (disk, not partition)
                        let (boot_device, is_removable) = if let Some(disk_config) = &config.disk {
                            (disk_config.device.clone(), disk_config.removable)
                        } else {
                            anyhow::bail!("No disk configuration found for GRUB installation");
                        };

                        let is_efi = system::is_efi_system();

                        // Verify boot/EFI partition is mounted
                        update_progress(
                            "Installing bootloader",
                            0.905,
                            0.1,
                            "Verifying boot partition...",
                        );

                        let boot_mount_check = if is_efi {
                            config.target_root.join("boot/efi").exists()
                        } else {
                            config.target_root.join("boot").exists()
                        };

                        if !boot_mount_check {
                            anyhow::bail!(
                                "Boot partition not properly mounted at {}",
                                if is_efi { "/boot/efi" } else { "/boot" }
                            );
                        }

                        // Set up bind mounts for chroot (required for grub-install)
                        update_progress(
                            "Installing bootloader",
                            0.91,
                            0.15,
                            "Preparing chroot environment...",
                        );

                        let bind_mounts = setup_chroot_mounts(&config.target_root)?;

                        // Mount efivarfs for EFI systems (required for efibootmgr)
                        // Skip for removable media to prevent modifying host system's EFI variables
                        if is_efi
                            && !is_removable
                            && PathBuf::from("/sys/firmware/efi/efivars").exists()
                        {
                            let efivars_target =
                                config.target_root.join("sys/firmware/efi/efivars");
                            std::fs::create_dir_all(&efivars_target)?;
                            let output = Command::new("mount")
                                .args([
                                    "-t",
                                    "efivarfs",
                                    "efivarfs",
                                    efivars_target.to_str().unwrap(),
                                ])
                                .output();

                            if let Ok(output) = output {
                                if output.status.success() {
                                    tracing::info!("Mounted efivarfs in chroot");
                                } else {
                                    tracing::warn!(
                                        "Failed to mount efivarfs: {}",
                                        String::from_utf8_lossy(&output.stderr)
                                    );
                                }
                            }
                        } else if is_removable {
                            tracing::info!("Skipping efivarfs mount for removable media to protect host EFI variables");
                        }

                        update_progress(
                            "Installing bootloader",
                            0.915,
                            0.15,
                            "Configuring dynamic linker...",
                        );

                        // Create /etc/ld.so.conf if it doesn't exist (required for ldconfig to find libraries in /usr/lib64)
                        let ld_so_conf_path = config.target_root.join("etc/ld.so.conf");
                        if !ld_so_conf_path.exists() {
                            let ld_so_conf_content = "# Multilib support\n/usr/lib64\n/usr/lib\n/lib64\n/lib\ninclude /etc/ld.so.conf.d/*.conf\n";
                            std::fs::write(&ld_so_conf_path, ld_so_conf_content).map_err(|e| {
                                anyhow::anyhow!("Failed to create /etc/ld.so.conf: {}", e)
                            })?;
                            tracing::info!("Created /etc/ld.so.conf");

                            // Create the ld.so.conf.d directory
                            let ld_so_conf_d_path = config.target_root.join("etc/ld.so.conf.d");
                            std::fs::create_dir_all(&ld_so_conf_d_path).map_err(|e| {
                                anyhow::anyhow!("Failed to create /etc/ld.so.conf.d: {}", e)
                            })?;
                        }

                        update_progress(
                            "Installing bootloader",
                            0.92,
                            0.16,
                            "Updating library cache...",
                        );

                        // Run ldconfig to rebuild the dynamic linker cache from scratch
                        // Use -X to avoid reading the existing cache (prevents RPATH conflicts)
                        let ldconfig_output = Command::new("chroot")
                            .arg(&config.target_root)
                            .env_clear()
                            .env(
                                "PATH",
                                "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                            )
                            .env("HOME", "/root")
                            .env("TERM", "linux")
                            .args(["ldconfig", "-X"])
                            .output()
                            .map_err(|e| anyhow::anyhow!("Failed to run ldconfig: {}", e))?;

                        if !ldconfig_output.status.success() {
                            let stderr = String::from_utf8_lossy(&ldconfig_output.stderr);
                            tracing::warn!("ldconfig warning: {}", stderr);
                        } else {
                            tracing::info!("ldconfig completed successfully");
                        }

                        update_progress(
                            "Installing bootloader",
                            0.925,
                            0.2,
                            "Running grub-install...",
                        );

                        // Create GRUB directory if it doesn't exist
                        let grub_dir = if is_efi {
                            config.target_root.join("boot/efi/EFI/BuckOS")
                        } else {
                            config.target_root.join("boot/grub")
                        };
                        std::fs::create_dir_all(&grub_dir)?;

                        // Install GRUB
                        // Build the grub-install command with proper environment setup inside the chroot
                        let mut grub_args = vec!["grub-install"];

                        if is_efi {
                            grub_args.extend_from_slice(&[
                                "--target=x86_64-efi",
                                "--efi-directory=/boot/efi",
                                "--bootloader-id=BuckOS",
                                "--recheck",
                            ]);

                            // For removable media, use --no-nvram to prevent modifying host EFI variables
                            if is_removable {
                                grub_args.push("--no-nvram");
                                grub_args.push("--removable");
                                tracing::info!(
                                    "Installing GRUB for removable media (--no-nvram --removable)"
                                );
                            }

                            grub_args.push(&boot_device);
                        } else {
                            grub_args.extend_from_slice(&[
                                "--target=i386-pc",
                                "--recheck",
                                &boot_device,
                            ]);
                        }

                        // Execute grub-install in the chroot with proper environment
                        let grub_cmd = grub_args.join(" ");

                        let mut grub_install_cmd = Command::new("chroot");
                        grub_install_cmd
                            .arg(&config.target_root)
                            .env_clear()
                            .env(
                                "PATH",
                                "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                            )
                            .env("HOME", "/root")
                            .env("TERM", "linux")
                            .arg("/bin/sh")
                            .arg("-c")
                            .arg(&grub_cmd);

                        let output = grub_install_cmd.output()
                            .map_err(|e| anyhow::anyhow!(
                                "Failed to execute grub-install command: {}. Make sure chroot is available.",
                                e
                            ))?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);

                            // Cleanup bind mounts before failing
                            cleanup_chroot_mounts(&config.target_root, &bind_mounts);

                            anyhow::bail!(
                                "Failed to install GRUB:\nstderr: {}\nstdout: {}\n\
                                Make sure the @grub package includes all necessary GRUB modules.",
                                stderr,
                                stdout
                            );
                        }

                        tracing::info!("grub-install completed successfully");
                        update_progress(
                            "Installing bootloader",
                            0.93,
                            0.5,
                            "Generating initramfs...",
                        );

                        // Detect kernel version from installed kernel modules
                        let kernel_version = detect_kernel_version(&config.target_root);
                        tracing::info!("Detected kernel version: {}", kernel_version);

                        // Rename kernel files to include version suffix (required for grub-mkconfig detection)
                        rename_kernel_files(&config.target_root, &kernel_version)?;

                        // Create required temporary directories for dracut
                        let tmp_dir = config.target_root.join("tmp");
                        let var_tmp_dir = config.target_root.join("var/tmp");
                        std::fs::create_dir_all(&tmp_dir)?;
                        std::fs::create_dir_all(&var_tmp_dir)?;

                        // Set proper permissions (1777 = sticky bit + rwxrwxrwx)
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let perms = std::fs::Permissions::from_mode(0o1777);
                            std::fs::set_permissions(&tmp_dir, perms.clone())?;
                            std::fs::set_permissions(&var_tmp_dir, perms)?;
                        }
                        tracing::info!("Created temporary directories for dracut");

                        // Create symlink from /usr/lib/dracut to /usr/lib64/dracut if needed
                        let lib_dracut = config.target_root.join("usr/lib/dracut");
                        let lib64_dracut = config.target_root.join("usr/lib64/dracut");
                        if lib64_dracut.exists() && !lib_dracut.exists() {
                            std::os::unix::fs::symlink("../lib64/dracut", &lib_dracut)?;
                            tracing::info!(
                                "Created symlink from /usr/lib/dracut to /usr/lib64/dracut"
                            );
                        }

                        // Generate initramfs images with dracut (default + fallback like Arch/CachyOS)
                        // Default: optimized for current hardware (hostonly for fixed installs)
                        // Fallback: includes all modules for portability (VMs, hardware changes, rescue)
                        tracing::info!(
                            "Generating initramfs images with dracut for kernel {}",
                            kernel_version
                        );

                        // Helper to run dracut with given arguments
                        let run_dracut = |initramfs_path: &str,
                                          use_hostonly: bool,
                                          description: &str|
                         -> Result<(), anyhow::Error> {
                            tracing::info!(
                                "Generating {} initramfs: {}",
                                description,
                                initramfs_path
                            );
                            let mut cmd = Command::new("chroot");
                            cmd.arg(&config.target_root)
                                .env_clear()
                                .env(
                                    "PATH",
                                    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                                )
                                .env("HOME", "/root")
                                .env("TERM", "linux")
                                .arg("/usr/bin/dracut")
                                .arg("--force");

                            if use_hostonly {
                                cmd.arg("--hostonly");
                            } else {
                                cmd.arg("--no-hostonly");
                            }

                            // Add systemd modules when using systemd as init system
                            // These are required for proper initrd.target and switch-root functionality
                            if matches!(config.init_system, crate::types::InitSystem::Systemd) {
                                cmd.arg("--add").arg("systemd systemd-initrd");
                            }

                            // Check for available dracut modules and add useful ones
                            // Dracut modules are in /usr/lib/dracut/modules.d/ or /usr/lib64/dracut/modules.d/
                            let mut additional_modules = Vec::new();
                            let dracut_modules_paths = [
                                config.target_root.join("usr/lib/dracut/modules.d"),
                                config.target_root.join("usr/lib64/dracut/modules.d"),
                            ];

                            // List of potentially useful dracut modules for better boot support
                            let desired_modules = [
                                "base",           // Base dracut functionality
                                "bash",           // Bash shell for emergency mode
                                "fs-lib",         // Filesystem library support
                                "rootfs-block",   // Block device root filesystem
                                "kernel-modules", // Kernel modules loading
                                "udev-rules",     // Udev rules for device management
                                "usrmount",       // Mount /usr if separate
                                "resume",         // Resume from hibernation
                            ];

                            for module_name in &desired_modules {
                                for dracut_path in &dracut_modules_paths {
                                    if dracut_path.exists() {
                                        // Dracut modules are directories like "90kernel-modules" or "99base"
                                        if let Ok(entries) = std::fs::read_dir(dracut_path) {
                                            for entry in entries.flatten() {
                                                if let Ok(file_name) =
                                                    entry.file_name().into_string()
                                                {
                                                    // Module dirs are named like "90kernel-modules", "99base", etc.
                                                    if file_name.ends_with(module_name)
                                                        && !additional_modules
                                                            .contains(&module_name.to_string())
                                                    {
                                                        additional_modules
                                                            .push(module_name.to_string());
                                                        tracing::info!(
                                                            "Found dracut module: {}",
                                                            module_name
                                                        );
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if !additional_modules.is_empty() {
                                let modules_str = additional_modules.join(" ");
                                cmd.arg("--add").arg(modules_str);
                                tracing::info!(
                                    "Adding dracut modules: {}",
                                    additional_modules.join(", ")
                                );
                            }

                            // Add sulogin for emergency shell if it exists
                            let sulogin_path = config.target_root.join("usr/bin/sulogin");
                            if sulogin_path.exists() {
                                cmd.arg("--install").arg("/usr/bin/sulogin");
                                tracing::info!("Adding sulogin to initramfs for emergency shell");
                            } else {
                                tracing::warn!("sulogin not found at /usr/bin/sulogin - emergency shell may not work");
                            }

                            cmd.arg(initramfs_path).arg("--kver").arg(&kernel_version);

                            let output = cmd.output()
                                .map_err(|e| anyhow::anyhow!(
                                    "Failed to execute dracut command: {}. Make sure dracut is installed in the rootfs.",
                                    e
                                ))?;

                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                anyhow::bail!(
                                    "Failed to generate {} initramfs with dracut:\nstdout: {}\nstderr: {}",
                                    description, stdout, stderr
                                );
                            }
                            Ok(())
                        };

                        // Generate default initramfs
                        let initramfs_path = format!("/boot/initramfs-{}.img", kernel_version);
                        let use_hostonly_default = !config.include_all_firmware && !is_removable;
                        if use_hostonly_default {
                            tracing::info!("Default initramfs: hostonly mode (smaller, optimized for this machine)");
                        } else {
                            tracing::info!(
                                "Default initramfs: no-hostonly mode (portable across machines)"
                            );
                        }

                        if let Err(e) = run_dracut(&initramfs_path, use_hostonly_default, "default")
                        {
                            // Cleanup bind mounts before failing
                            cleanup_chroot_mounts(&config.target_root, &bind_mounts);
                            anyhow::bail!(
                                "{}\n\
                                The system will not boot without an initramfs. Please ensure:\n\
                                1. The dracut package is properly installed in the rootfs\n\
                                2. The getopt utility is available (provided by util-linux)\n\
                                3. All required kernel modules and firmware are present",
                                e
                            );
                        }
                        tracing::info!("Default initramfs generated successfully");

                        // Generate fallback initramfs (always includes all modules)
                        let initramfs_fallback_path =
                            format!("/boot/initramfs-{}-fallback.img", kernel_version);
                        tracing::info!("Fallback initramfs: no-hostonly mode (includes all modules for rescue/VMs)");

                        if let Err(e) = run_dracut(&initramfs_fallback_path, false, "fallback") {
                            // Fallback generation failure is a warning, not fatal
                            tracing::warn!(
                                "Failed to generate fallback initramfs: {}. \
                                The system will still boot with the default initramfs, but fallback option won't be available.",
                                e
                            );
                        } else {
                            tracing::info!("Fallback initramfs generated successfully");
                        }

                        tracing::info!("Initramfs generation complete");

                        update_progress(
                            "Installing bootloader",
                            0.935,
                            0.6,
                            "Generating GRUB configuration...",
                        );

                        // Create /etc/default/grub if it doesn't exist (required for grub-mkconfig)
                        let default_grub_path = config.target_root.join("etc/default/grub");
                        if !default_grub_path.exists() {
                            // Build kernel command line based on installation type
                            let cmdline_default = if is_removable {
                                // For removable media: rootwait waits for USB to initialize
                                "rootwait"
                            } else {
                                ""
                            };

                            let default_grub_content = format!(
                                r#"# GRUB configuration for BuckOS
GRUB_DEFAULT=0
GRUB_TIMEOUT=5
GRUB_DISTRIBUTOR="BuckOS"
GRUB_CMDLINE_LINUX_DEFAULT="{}"
GRUB_CMDLINE_LINUX=""
GRUB_TERMINAL_OUTPUT="console"
"#,
                                cmdline_default
                            );
                            std::fs::create_dir_all(default_grub_path.parent().unwrap())?;
                            std::fs::write(&default_grub_path, default_grub_content).map_err(
                                |e| anyhow::anyhow!("Failed to create /etc/default/grub: {}", e),
                            )?;
                            tracing::info!("Created /etc/default/grub");
                        }

                        // Create custom GRUB script for fallback initramfs entries
                        // This script runs after 10_linux and adds fallback boot entries
                        let grub_d_path = config.target_root.join("etc/grub.d");
                        std::fs::create_dir_all(&grub_d_path)?;
                        let fallback_script_path = grub_d_path.join("11_linux_fallback");
                        let fallback_script_content = r#"#!/bin/sh
set -e

# Generate fallback boot entries for BuckOS
# This script adds entries for initramfs-*-fallback.img files

. "$pkgdatadir/grub-mkconfig_lib"

CLASS="--class gnu-linux --class gnu --class os"
GRUB_CMDLINE_LINUX="${GRUB_CMDLINE_LINUX:-}"
GRUB_CMDLINE_LINUX_DEFAULT="${GRUB_CMDLINE_LINUX_DEFAULT:-}"

# Find all fallback initramfs images
for fallback in /boot/initramfs-*-fallback.img; do
    [ -f "$fallback" ] || continue

    # Extract kernel version from fallback image name
    # e.g., /boot/initramfs-6.12.6-buckos-fallback.img -> 6.12.6-buckos
    basename=$(basename "$fallback")
    version=$(echo "$basename" | sed 's/initramfs-\(.*\)-fallback\.img/\1/')

    # Find corresponding kernel
    linux="/boot/vmlinuz-${version}"
    [ -f "$linux" ] || continue

    # Get root device
    GRUB_DEVICE=$(${grub_probe} --target=device /)
    GRUB_DEVICE_UUID=$(${grub_probe} --device ${GRUB_DEVICE} --target=fs_uuid 2>/dev/null || true)

    if [ "x${GRUB_DEVICE_UUID}" = "x" ]; then
        LINUX_ROOT_DEVICE="${GRUB_DEVICE}"
    else
        LINUX_ROOT_DEVICE="UUID=${GRUB_DEVICE_UUID}"
    fi

    echo "Found fallback initramfs: $fallback" >&2

    cat << EOF
menuentry 'BuckOS (${version}, fallback initramfs)' ${CLASS} {
    load_video
    set gfxpayload=keep
    insmod gzio
    insmod part_gpt
    insmod ext2
EOF

    if [ "x${GRUB_DEVICE_UUID}" != "x" ]; then
        cat << EOF
    search --no-floppy --fs-uuid --set=root ${GRUB_DEVICE_UUID}
EOF
    fi

    cat << EOF
    echo 'Loading Linux ${version} with fallback initramfs...'
    linux ${linux} root=${LINUX_ROOT_DEVICE} ro ${GRUB_CMDLINE_LINUX} ${GRUB_CMDLINE_LINUX_DEFAULT}
    echo 'Loading fallback initial ramdisk...'
    initrd ${fallback}
}
EOF

done
"#;
                        std::fs::write(&fallback_script_path, fallback_script_content).map_err(
                            |e| anyhow::anyhow!("Failed to create GRUB fallback script: {}", e),
                        )?;

                        // Make the script executable
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let perms = std::fs::Permissions::from_mode(0o755);
                            std::fs::set_permissions(&fallback_script_path, perms)?;
                        }
                        tracing::info!(
                            "Created GRUB fallback script at {}",
                            fallback_script_path.display()
                        );

                        // Generate GRUB configuration
                        let output = Command::new("chroot")
                            .arg(&config.target_root)
                            .env_clear()
                            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                            .env("HOME", "/root")
                            .env("TERM", "linux")
                            .arg("/usr/sbin/grub-mkconfig")
                            .arg("-o")
                            .arg("/boot/grub/grub.cfg")
                            .output()
                            .map_err(|e| anyhow::anyhow!(
                                "Failed to execute grub-mkconfig command: {}. Make sure chroot is available.",
                                e
                            ))?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);

                            // Cleanup bind mounts before failing
                            cleanup_chroot_mounts(&config.target_root, &bind_mounts);

                            anyhow::bail!(
                                "Failed to generate GRUB config:\nstderr: {}\nstdout: {}",
                                stderr,
                                stdout
                            );
                        }

                        tracing::info!("grub-mkconfig completed successfully");

                        // Verify GRUB config was created
                        let grub_cfg_path = config.target_root.join("boot/grub/grub.cfg");
                        if !grub_cfg_path.exists() {
                            tracing::warn!(
                                "GRUB config file not found at {}",
                                grub_cfg_path.display()
                            );
                        } else {
                            tracing::info!("GRUB config created at {}", grub_cfg_path.display());
                        }

                        // Cleanup: Unmount bind mounts in reverse order
                        update_progress(
                            "Installing bootloader",
                            0.945,
                            0.9,
                            "Cleaning up chroot environment...",
                        );
                        cleanup_chroot_mounts(&config.target_root, &bind_mounts);

                        tracing::info!("GRUB installation completed successfully");
                    }
                }

                crate::types::BootloaderType::Systemdboot => {
                    update_progress(
                        "Installing bootloader",
                        0.92,
                        0.3,
                        "Running bootctl install...",
                    );

                    // Install systemd-boot
                    let output = Command::new("chroot")
                        .arg(&config.target_root)
                        .env_clear()
                        .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                        .env("HOME", "/root")
                        .env("TERM", "linux")
                        .arg("bootctl")
                        .arg("install")
                        .output()
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to execute chroot/bootctl command: {}. Make sure chroot is available.",
                            e
                        ))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        anyhow::bail!("Failed to install systemd-boot: {}", stderr);
                    }

                    update_progress(
                        "Installing bootloader",
                        0.935,
                        0.6,
                        "Creating systemd-boot entries...",
                    );

                    // Create loader configuration
                    let loader_conf_path = config.target_root.join("boot/loader/loader.conf");
                    std::fs::create_dir_all(loader_conf_path.parent().unwrap())?;
                    let loader_conf =
                        "default buckos.conf\ntimeout 3\nconsole-mode max\neditor no\n";
                    std::fs::write(&loader_conf_path, loader_conf)?;

                    // Create boot entry
                    let entries_dir = config.target_root.join("boot/loader/entries");
                    std::fs::create_dir_all(&entries_dir)?;

                    // Find root partition UUID
                    let root_uuid = if let Some(disk_config) = &config.disk {
                        if let Some(root_part) = disk_config
                            .partitions
                            .iter()
                            .find(|p| p.mount_point == crate::types::MountPoint::Root)
                        {
                            // Get UUID using blkid
                            let output = Command::new("blkid")
                                .arg("-s").arg("UUID")
                                .arg("-o").arg("value")
                                .arg(&root_part.device)
                                .output()
                                .map_err(|e| anyhow::anyhow!(
                                    "Failed to execute blkid command: {}. Make sure blkid is installed.",
                                    e
                                ))?;

                            if output.status.success() {
                                String::from_utf8_lossy(&output.stdout).trim().to_string()
                            } else {
                                root_part.device.clone()
                            }
                        } else {
                            "/dev/root".to_string()
                        }
                    } else {
                        "/dev/root".to_string()
                    };

                    let entry_path = entries_dir.join("buckos.conf");
                    let entry_content = format!(
                        "title   BuckOS\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux.img\noptions root=UUID={} rw\n",
                        root_uuid
                    );
                    std::fs::write(&entry_path, entry_content)?;
                }

                crate::types::BootloaderType::Refind => {
                    update_progress("Installing bootloader", 0.92, 0.3, "Installing rEFInd...");

                    // Install rEFInd
                    let output = Command::new("chroot")
                        .arg(&config.target_root)
                        .env_clear()
                        .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                        .env("HOME", "/root")
                        .env("TERM", "linux")
                        .arg("refind-install")
                        .output()
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to execute chroot/refind-install command: {}. Make sure chroot is available.",
                            e
                        ))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        anyhow::bail!("Failed to install rEFInd: {}", stderr);
                    }

                    update_progress("Installing bootloader", 0.935, 0.6, "Configuring rEFInd...");

                    // Create basic refind.conf if it doesn't exist
                    let refind_conf_path =
                        config.target_root.join("boot/efi/EFI/refind/refind.conf");
                    if !refind_conf_path.exists() {
                        let refind_conf = "timeout 5\nuse_graphics_for linux\nscanfor manual,external,optical,internal\n";
                        std::fs::write(&refind_conf_path, refind_conf)?;
                    }
                }

                crate::types::BootloaderType::Limine => {
                    update_progress("Installing bootloader", 0.92, 0.3, "Installing Limine...");

                    // Find boot device
                    let boot_device = if let Some(disk_config) = &config.disk {
                        disk_config.device.clone()
                    } else {
                        anyhow::bail!("No disk configuration found for Limine installation");
                    };

                    // Copy Limine binaries
                    let limine_deploy = Command::new("chroot")
                        .arg(&config.target_root)
                        .env_clear()
                        .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                        .env("HOME", "/root")
                        .env("TERM", "linux")
                        .arg("limine-deploy")
                        .arg(&boot_device)
                        .output()
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to execute chroot/limine-deploy command: {}. Make sure chroot is available.",
                            e
                        ))?;

                    if !limine_deploy.status.success() {
                        let stderr = String::from_utf8_lossy(&limine_deploy.stderr);
                        anyhow::bail!("Failed to deploy Limine: {}", stderr);
                    }

                    update_progress(
                        "Installing bootloader",
                        0.935,
                        0.6,
                        "Creating Limine configuration...",
                    );

                    // Create Limine configuration
                    let limine_cfg_path = config.target_root.join("boot/limine.cfg");
                    let limine_cfg = "TIMEOUT=5\n\n:BuckOS\nPROTOCOL=linux\nKERNEL_PATH=boot:///vmlinuz-linux\nKERNEL_CMDLINE=root=/dev/sda2 rw\nMODULE_PATH=boot:///initramfs-linux.img\n";
                    std::fs::write(&limine_cfg_path, limine_cfg)?;
                }

                crate::types::BootloaderType::Efistub => {
                    update_progress(
                        "Installing bootloader",
                        0.92,
                        0.3,
                        "Creating EFISTUB boot entry...",
                    );

                    // Check if installing to removable media
                    let is_removable = config.disk.as_ref().map(|d| d.removable).unwrap_or(false);

                    if is_removable {
                        tracing::warn!("EFISTUB bootloader is not recommended for removable media");
                        tracing::warn!(
                            "Skipping efibootmgr to prevent modifying host EFI variables"
                        );
                        update_progress("Installing bootloader", 0.95, 1.0,
                            "⚠ EFISTUB installation skipped for removable media. Use GRUB or systemd-boot instead.");
                    } else {
                        // Find root partition
                        let root_dev = if let Some(disk_config) = &config.disk {
                            if let Some(root_part) = disk_config
                                .partitions
                                .iter()
                                .find(|p| p.mount_point == crate::types::MountPoint::Root)
                            {
                                root_part.device.clone()
                            } else {
                                "/dev/sda2".to_string()
                            }
                        } else {
                            "/dev/sda2".to_string()
                        };

                        // Create UEFI boot entry using efibootmgr
                        let output = Command::new("chroot")
                        .arg(&config.target_root)
                        .env_clear()
                        .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                        .env("HOME", "/root")
                        .env("TERM", "linux")
                        .arg("efibootmgr")
                        .arg("--create")
                        .arg("--disk").arg("/dev/sda")
                        .arg("--part").arg("1")
                        .arg("--label").arg("buckos")
                        .arg("--loader").arg("/vmlinuz-linux")
                        .arg("--unicode")
                        .arg(format!("root={} rw initrd=\\initramfs-linux.img", root_dev))
                        .output()
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to execute chroot/efibootmgr command: {}. Make sure chroot is available.",
                            e
                        ))?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            anyhow::bail!("Failed to create EFISTUB boot entry: {}", stderr);
                        }
                    }
                }

                crate::types::BootloaderType::None => {}
            }

            update_progress(
                "Installing bootloader",
                0.95,
                1.0,
                format!("✓ {} bootloader installed", bootloader_name).as_str(),
            );
        } else {
            update_progress(
                "Installing bootloader",
                0.95,
                1.0,
                "✓ Skipped bootloader installation",
            );
        }

        // Step 8: User creation (95-98%)
        update_progress(
            "Creating users",
            0.95,
            0.0,
            "Verifying user management utilities...",
        );

        // Verify required utilities exist in the target system
        let chpasswd_path = config.target_root.join("usr/sbin/chpasswd");
        let useradd_path = config.target_root.join("usr/sbin/useradd");

        if !chpasswd_path.exists() {
            anyhow::bail!(
                "CRITICAL: chpasswd utility not found at {}.\n\
                The @system package must be installed and must include shadow-utils or equivalent.\n\
                User management utilities (chpasswd, useradd, passwd) are required for installation.\n\
                Please ensure the @system package includes these utilities.",
                chpasswd_path.display()
            );
        }

        if !useradd_path.exists() {
            anyhow::bail!(
                "CRITICAL: useradd utility not found at {}.\n\
                The @system package must be installed and must include shadow-utils or equivalent.\n\
                User management utilities (chpasswd, useradd, passwd) are required for installation.\n\
                Please ensure the @system package includes these utilities.",
                useradd_path.display()
            );
        }

        tracing::info!("User management utilities verified: chpasswd and useradd found");
        update_progress(
            "Creating users",
            0.92,
            0.1,
            "✓ User management utilities verified",
        );

        // Initialize /etc/passwd, /etc/shadow, and /etc/group if they don't exist
        update_progress(
            "Creating users",
            0.92,
            0.15,
            "Initializing user database...",
        );
        // Note: init_system_files was already called earlier in system config step

        // Set up PAM configuration
        setup_pam_config(&config.target_root)?;

        update_progress("Creating users", 0.96, 0.2, "Setting root password...");
        set_user_password(&config.target_root, "root", &config.root_password)?;
        update_progress("Creating users", 0.965, 0.3, "✓ Root password set");

        // Create user accounts
        for (idx, user) in config.users.iter().enumerate() {
            let step = 0.3 + ((idx as f32 + 1.0) / config.users.len() as f32) * 0.7;
            update_progress(
                "Creating users",
                0.965 + (step * 0.015),
                step,
                format!("Creating user: {}", user.username).as_str(),
            );

            // Create user with useradd
            if let Err(e) = create_user_account(
                &config.target_root,
                &user.username,
                &user.full_name,
                &user.shell,
                user.is_admin,
            ) {
                update_progress(
                    "Creating users",
                    0.965 + (step * 0.015),
                    step,
                    format!("Warning: Failed to create user {}: {}", user.username, e).as_str(),
                );
                continue;
            }

            // Set user password
            if let Err(e) = set_user_password(&config.target_root, &user.username, &user.password) {
                update_progress(
                    "Creating users",
                    0.93 + (step * 0.02),
                    step,
                    format!(
                        "Warning: Failed to set password for {}: {}",
                        user.username, e
                    )
                    .as_str(),
                );
            }
        }

        update_progress("Creating users", 0.98, 1.0, "✓ User accounts created");

        // Step 9: Finalization (98-100%)
        update_progress("Finalizing installation", 0.98, 0.5, "Cleaning up...");
        update_progress(
            "Finalizing installation",
            0.99,
            0.9,
            "Unmounting filesystems...",
        );
        update_progress(
            "Installation complete",
            1.0,
            1.0,
            "✓ Installation completed successfully!",
        );

        Ok(())
    };

    // Run the installation and handle errors
    if let Err(e) = run_step() {
        // Display the full error chain for better debugging
        let error_msg = format!("Installation failed: {:?}", e);
        log_error(&error_msg);
        tracing::error!("Installation failed: {:?}", e);

        // Also log the error chain separately for clarity
        let mut current = e.source();
        while let Some(err) = current {
            tracing::error!("  Caused by: {}", err);
            current = err.source();
        }
    }
}
