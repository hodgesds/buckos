//! Installation logic and helpers

use crate::system;
use crate::types::{FilesystemType, InstallConfig, InstallProgress, MountPoint, PartitionConfig};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

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

                if let (Ok(current), Ok(total)) = (current_str.parse::<u32>(), total_str.parse::<u32>()) {
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

        // Step 1: Pre-installation checks (5%)
        update_progress("Pre-installation checks", 0.05, 0.0, "Starting installation...");
        update_progress("Pre-installation checks", 0.05, 0.5, "Checking system requirements...");

        if !system::is_root() {
            anyhow::bail!("Installation must be run as root");
        }

        update_progress("Pre-installation checks", 0.05, 1.0, "✓ Pre-installation checks complete");

        // Step 2: Disk partitioning (15%)
        update_progress("Disk partitioning", 0.10, 0.0, "Preparing disk partitioning...");

        if let Some(disk_config) = &config.disk {
            // Safety check: Prevent installing on the disk containing the running system
            update_progress("Disk partitioning", 0.10, 0.05, "Checking disk safety...");

            let root_device_result = Command::new("findmnt")
                .args(&["-n", "-o", "SOURCE", "/"])
                .output();

            if let Ok(output) = root_device_result {
                if output.status.success() {
                    let root_partition = String::from_utf8_lossy(&output.stdout).trim().to_string();

                    // Use lsblk to get the parent disk device - more reliable than string manipulation
                    let lsblk_result = Command::new("lsblk")
                        .args(&["-no", "PKNAME", &root_partition])
                        .output();

                    let root_disk = if let Ok(lsblk_output) = lsblk_result {
                        if lsblk_output.status.success() {
                            let pkname = String::from_utf8_lossy(&lsblk_output.stdout).trim().to_string();
                            if !pkname.is_empty() {
                                format!("/dev/{}", pkname)
                            } else {
                                // lsblk didn't return a parent, the device might be the disk itself
                                root_partition.clone()
                            }
                        } else {
                            root_partition.clone()
                        }
                    } else {
                        root_partition.clone()
                    };

                    // Check if target disk matches the root disk
                    if disk_config.device == root_disk || root_disk.contains(&disk_config.device) {
                        anyhow::bail!(
                            "SAFETY CHECK FAILED: Cannot install on {} - it contains the running system's root filesystem ({}).\n\
                            This would destroy your running system!\n\
                            Please select a different disk or boot from a live USB/CD to install.",
                            disk_config.device, root_partition
                        );
                    }

                    tracing::info!("Safety check passed: root is on {}, installing to {}", root_disk, disk_config.device);
                }
            }

            update_progress("Disk partitioning", 0.11, 0.1,
                format!("Preparing disk: {}", disk_config.device).as_str());

            // Unmount any partitions from the target disk
            update_progress("Disk partitioning", 0.115, 0.2, "Unmounting existing partitions...");

            // Use findmnt to get a reliable list of mounted filesystems
            let findmnt_output = Command::new("findmnt")
                .args(&["-rno", "TARGET,SOURCE"])
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

                            // Check if this device is on our target disk
                            if device.starts_with(&disk_config.device) {
                                tracing::info!("Found mounted: {} on {}", device, mount_point);
                                mount_points_to_unmount.push((mount_point.to_string(), device.to_string()));
                            }
                        }
                    }
                }
            } else {
                tracing::warn!("findmnt command failed or not available");
            }

            tracing::info!("Found {} mount points to unmount", mount_points_to_unmount.len());

            // Sort mount points by depth (deepest first) to unmount in correct order
            mount_points_to_unmount.sort_by(|a, b| {
                b.0.matches('/').count().cmp(&a.0.matches('/').count())
            });

            // Unmount each mount point
            for (mount_point, device) in &mount_points_to_unmount {
                update_progress("Disk partitioning", 0.115, 0.3,
                    format!("Unmounting {} ({})", mount_point, device).as_str());

                tracing::info!("Unmounting: {} ({})", mount_point, device);

                // Try normal unmount first
                let umount_result = Command::new("umount")
                    .arg(mount_point)
                    .output();

                match umount_result {
                    Ok(output) if output.status.success() => {
                        tracing::info!("Successfully unmounted {}", mount_point);
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::warn!("Failed to unmount {}: {}, trying lazy unmount", mount_point, stderr);

                        // Try lazy unmount as fallback
                        let _ = Command::new("umount")
                            .args(&["-l", mount_point])
                            .output();
                    }
                    Err(e) => {
                        tracing::warn!("Failed to unmount {}: {}", mount_point, e);
                    }
                }
            }

            // Deactivate any swap on the target disk
            match Command::new("swapon").arg("--show=NAME").arg("--noheadings").output() {
                Ok(output) if output.status.success() => {
                    let swap_list = String::from_utf8_lossy(&output.stdout);
                    for swap_device in swap_list.lines() {
                        let swap_device = swap_device.trim();
                        if swap_device.starts_with(&disk_config.device) {
                            update_progress("Disk partitioning", 0.118, 0.4,
                                format!("Deactivating swap on {}...", swap_device).as_str());

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

            // Check if any processes are still using the disk (optional - fuser might not be available)
            update_progress("Disk partitioning", 0.119, 0.45, "Checking for processes using disk...");

            match Command::new("fuser").args(&["-m", &disk_config.device]).output() {
                Ok(output) => {
                    let users = String::from_utf8_lossy(&output.stdout);
                    if !users.trim().is_empty() {
                        tracing::warn!("Processes still using {}: {}", disk_config.device, users);
                        update_progress("Disk partitioning", 0.119, 0.5,
                            "Terminating processes using disk...");

                        // Kill processes using the disk
                        let _ = Command::new("fuser")
                            .args(&["-km", &disk_config.device])
                            .output();

                        // Wait a moment for processes to die
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
                Err(e) => {
                    tracing::debug!("fuser command not available: {}", e);
                }
            }

            tracing::info!("Starting disk partitioning on {}", disk_config.device);
            update_progress("Disk partitioning", 0.12, 0.3,
                format!("Partitioning disk: {}", disk_config.device).as_str());

            if disk_config.wipe_disk {
                tracing::info!("Wiping disk signatures with wipefs");
                update_progress("Disk partitioning", 0.13, 0.5,
                    format!("Wiping disk: {}", disk_config.device).as_str());

                // Wipe partition table signatures (optional, parted mklabel will also clear the table)
                match Command::new("wipefs").args(&["-a", &disk_config.device]).output() {
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
            tracing::info!("Creating {} partition table on {}", pt_type, disk_config.device);
            update_progress("Disk partitioning", 0.14, 0.7,
                format!("Creating {} partition table", pt_type).as_str());

            let output = Command::new("parted")
                .args(&["-s", &disk_config.device, "mklabel", pt_type])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to execute parted command: {}. Make sure parted is installed and in PATH.", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Provide detailed diagnostics on failure
                tracing::error!("Failed to create partition table on {}", disk_config.device);

                // Check what's still mounted (optional diagnostic)
                match Command::new("findmnt").args(&["-rno", "TARGET,SOURCE"]).output() {
                    Ok(mounts) if mounts.status.success() => {
                        let mount_list = String::from_utf8_lossy(&mounts.stdout);
                        for line in mount_list.lines() {
                            if line.contains(&disk_config.device) {
                                tracing::error!("Still mounted: {}", line);
                            }
                        }
                    }
                    Ok(_) => tracing::debug!("findmnt returned non-zero status"),
                    Err(e) => tracing::debug!("findmnt not available: {}", e),
                }

                // Check what processes are using it (optional diagnostic)
                match Command::new("fuser").args(&["-v", &disk_config.device]).output() {
                    Ok(fuser) => {
                        let users = String::from_utf8_lossy(&fuser.stderr); // fuser outputs to stderr
                        if !users.trim().is_empty() {
                            tracing::error!("Processes using disk: {}", users);
                        }
                    }
                    Err(e) => tracing::debug!("fuser not available: {}", e),
                }

                // Check for active swaps (optional diagnostic)
                match Command::new("swapon").arg("--show").output() {
                    Ok(swaps) if swaps.status.success() => {
                        let swap_list = String::from_utf8_lossy(&swaps.stdout);
                        if swap_list.contains(&disk_config.device) {
                            tracing::error!("Active swap on disk: {}", swap_list);
                        }
                    }
                    Ok(_) => tracing::debug!("swapon returned non-zero status"),
                    Err(e) => tracing::debug!("swapon not available: {}", e),
                }

                anyhow::bail!(
                    "Failed to create partition table: {}\n\
                    The disk may still be in use. Check the logs for details.\n\
                    You may need to manually unmount partitions or reboot before installing.",
                    stderr
                );
            }

            // Create partitions
            let mut start_mb: u64 = 1; // Start at 1MB to align partitions
            for (idx, partition) in disk_config.partitions.iter().enumerate() {
                let step = 0.7 + ((idx as f32 + 1.0) / disk_config.partitions.len() as f32) * 0.3;
                update_progress("Disk partitioning", 0.14 + (step * 0.01), step,
                    format!("Creating partition {}", partition.device).as_str());

                let size_mb = if partition.size == 0 {
                    // Use remaining space
                    "100%".to_string()
                } else {
                    format!("{}MB", start_mb + (partition.size / 1024 / 1024))
                };

                // Determine partition type for GPT
                let part_type = if disk_config.use_gpt {
                    match partition.mount_point {
                        MountPoint::BootEfi => "fat32",
                        MountPoint::Swap => "linux-swap",
                        MountPoint::Boot if partition.filesystem == FilesystemType::None => "bios_grub",
                        _ => "ext4", // Generic Linux filesystem
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
                    .args(&[
                        "-s",
                        &disk_config.device,
                        "mkpart",
                        "primary",
                        part_type,
                        &start,
                        &size_mb,
                    ])
                    .output()
                    .map_err(|e| anyhow::anyhow!(
                        "Failed to execute parted command for partition {}: {}. Make sure parted is available.",
                        partition.device, e
                    ))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("Failed to create partition {}: {}", partition.device, stderr);
                }

                // Set boot flag for EFI partition
                if partition.mount_point == MountPoint::BootEfi && disk_config.use_gpt {
                    let part_num = (idx + 1).to_string();
                    let output = Command::new("parted")
                        .args(&["-s", &disk_config.device, "set", &part_num, "esp", "on"])
                        .output()
                        .map_err(|e| anyhow::anyhow!("Failed to execute parted set esp: {}", e))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::warn!("Failed to set ESP flag on partition {}: {}", partition.device, stderr);
                    }
                }

                // Set bios_grub flag for BIOS boot partition
                if partition.mount_point == MountPoint::Boot
                    && partition.filesystem == FilesystemType::None
                    && disk_config.use_gpt
                {
                    let part_num = (idx + 1).to_string();
                    let output = Command::new("parted")
                        .args(&["-s", &disk_config.device, "set", &part_num, "bios_grub", "on"])
                        .output()
                        .map_err(|e| anyhow::anyhow!("Failed to execute parted set bios_grub: {}", e))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::warn!("Failed to set bios_grub flag on partition {}: {}", partition.device, stderr);
                    }
                }

                // Update start for next partition
                if partition.size > 0 {
                    start_mb += partition.size / 1024 / 1024;
                }
            }

            // Inform kernel of partition table changes
            update_progress("Disk partitioning", 0.148, 0.95, "Updating partition table...");
            let partprobe_result = Command::new("partprobe")
                .arg(&disk_config.device)
                .output();

            if let Err(e) = partprobe_result {
                // partprobe might not be available, try alternative
                tracing::warn!("partprobe failed: {}, trying blockdev --rereadpt", e);
                let _ = Command::new("blockdev")
                    .args(&["--rereadpt", &disk_config.device])
                    .output();
            }

            // Wait a moment for devices to appear
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        update_progress("Disk partitioning", 0.15, 1.0, "✓ Disk partitioning complete");

        // Step 3: Filesystem creation (25%)
        update_progress("Filesystem creation", 0.20, 0.0, "Creating filesystems...");

        if let Some(disk_config) = &config.disk {
            for (idx, partition) in disk_config.partitions.iter().enumerate() {
                let step = (idx as f32 + 1.0) / disk_config.partitions.len() as f32;

                if partition.format {
                    update_progress("Filesystem creation", 0.20 + (step * 0.05), step,
                        format!("Creating {} on {}", partition.filesystem.as_str(), partition.device).as_str());

                    // Build filesystem creation command with appropriate arguments
                    let (fs_cmd, args): (&str, Vec<&str>) = match partition.filesystem {
                        FilesystemType::Ext4 => {
                            ("mkfs.ext4", vec!["-F", partition.device.as_str()])
                        },
                        FilesystemType::Btrfs => {
                            ("mkfs.btrfs", vec!["-f", partition.device.as_str()])
                        },
                        FilesystemType::Xfs => {
                            ("mkfs.xfs", vec!["-f", partition.device.as_str()])
                        },
                        FilesystemType::F2fs => {
                            ("mkfs.f2fs", vec!["-f", partition.device.as_str()])
                        },
                        FilesystemType::Fat32 => {
                            ("mkfs.vfat", vec!["-F", "32", partition.device.as_str()])
                        },
                        FilesystemType::Swap => {
                            ("mkswap", vec![partition.device.as_str()])
                        },
                        FilesystemType::None => {
                            // Skip formatting for None filesystem type
                            continue;
                        }
                    };

                    let output = Command::new(fs_cmd)
                        .args(&args)
                        .output()
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to execute {} command for {}: {}. Make sure {} is installed and in PATH.",
                            fs_cmd, partition.device, e, fs_cmd
                        ))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        anyhow::bail!("Failed to create filesystem on {}: {}", partition.device, stderr);
                    }
                }
            }
        }

        update_progress("Filesystem creation", 0.25, 1.0, "✓ Filesystem creation complete");

        // Step 4: Mounting filesystems (30%)
        update_progress("Mounting filesystems", 0.28, 0.0, "Preparing to mount filesystems...");

        if let Some(disk_config) = &config.disk {
            // Sort partitions by mount order (root first, then nested mounts)
            let mut sorted_partitions: Vec<&PartitionConfig> = disk_config.partitions.iter().collect();
            sorted_partitions.sort_by(|a, b| {
                let a_path = a.mount_point.path();
                let b_path = b.mount_point.path();

                // Swap goes last
                if a_path == "swap" { return std::cmp::Ordering::Greater; }
                if b_path == "swap" { return std::cmp::Ordering::Less; }

                // Root goes first
                if a_path == "/" { return std::cmp::Ordering::Less; }
                if b_path == "/" { return std::cmp::Ordering::Greater; }

                // Sort by path depth (shorter paths first)
                a_path.matches('/').count().cmp(&b_path.matches('/').count())
                    .then_with(|| a_path.cmp(b_path))
            });

            for (idx, partition) in sorted_partitions.iter().enumerate() {
                let step = (idx as f32 + 1.0) / sorted_partitions.len() as f32;
                let mount_path = partition.mount_point.path();

                if partition.filesystem == FilesystemType::Swap {
                    update_progress("Mounting filesystems", 0.28 + (step * 0.02), step,
                        format!("Activating swap on {}", partition.device).as_str());

                    let output = Command::new("swapon")
                        .arg(&partition.device)
                        .output()
                        .map_err(|e| anyhow::anyhow!(
                            "Failed to execute swapon command for {}: {}. Make sure swapon is installed.",
                            partition.device, e
                        ))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        anyhow::bail!("Failed to activate swap on {}: {}", partition.device, stderr);
                    }
                    continue;
                }

                // Create mount point
                let full_mount_path = if mount_path == "/" {
                    config.target_root.clone()
                } else {
                    config.target_root.join(mount_path.trim_start_matches('/'))
                };

                update_progress("Mounting filesystems", 0.28 + (step * 0.02), step,
                    format!("Mounting {} to {}", partition.device, mount_path).as_str());

                std::fs::create_dir_all(&full_mount_path)?;

                // Mount the partition
                let mut mount_cmd = Command::new("mount");
                mount_cmd.arg(&partition.device);

                if !partition.mount_options.is_empty() {
                    mount_cmd.arg("-o").arg(&partition.mount_options);
                }

                mount_cmd.arg(&full_mount_path);

                let output = mount_cmd.output()
                    .map_err(|e| anyhow::anyhow!(
                        "Failed to execute mount command for {} to {}: {}. Make sure mount is installed.",
                        partition.device, full_mount_path.display(), e
                    ))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("Failed to mount {} to {}: {}",
                        partition.device, full_mount_path.display(), stderr);
                }
            }

            // Handle btrfs subvolumes for BtrfsSubvolumes layout
            if config.disk_layout == crate::types::DiskLayoutPreset::BtrfsSubvolumes {
                update_progress("Mounting filesystems", 0.29, 0.8, "Creating btrfs subvolumes...");

                // Find the root partition
                if let Some(root_part) = disk_config.partitions.iter()
                    .find(|p| p.mount_point == crate::types::MountPoint::Root)
                {
                    if root_part.filesystem == FilesystemType::Btrfs {
                        // Create subvolumes: @, @home, @snapshots
                        let btrfs_root = &config.target_root;

                        for subvol in &["@home", "@snapshots"] {
                            let output = Command::new("btrfs")
                                .args(&["subvolume", "create"])
                                .arg(btrfs_root.join(subvol.trim_start_matches('@')))
                                .output()
                                .map_err(|e| anyhow::anyhow!(
                                    "Failed to execute btrfs command for subvolume {}: {}. Make sure btrfs-progs is installed.",
                                    subvol, e
                                ))?;

                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                update_progress("Mounting filesystems", 0.29, 0.9,
                                    format!("Warning: Failed to create subvolume {}: {}", subvol, stderr).as_str());
                            }
                        }
                    }
                }
            }
        }

        update_progress("Mounting filesystems", 0.30, 1.0, "✓ Filesystems mounted");

        // Step 5: Build rootfs with Buck2 (70% - this is the longest step)
        update_progress("Building rootfs", 0.35, 0.0, "Generating custom rootfs target...");

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
        rootfs_packages.push("\"//packages/linux/core/file:file\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/bash:bash\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/zlib:zlib\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/glibc:glibc\"".to_string());

        // Add Linux kernel (user-selected channel)
        rootfs_packages.push(config.kernel_channel.package_target().to_string());

        // Add linux-firmware for hardware driver support
        rootfs_packages.push("\"//packages/linux/system/firmware/linux-firmware:linux-firmware\"".to_string());

        // Add dracut for initramfs generation
        rootfs_packages.push("\"//packages/linux/system/initramfs/dracut:dracut\"".to_string());

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

        update_progress("Building rootfs", 0.36, 0.05, "Writing custom BUCK target...");
        std::fs::write(&install_buck_path, buck_content)?;
        tracing::info!("Generated installer BUCK file at: {}", install_buck_path.display());

        // Write kernel config fragment if hardware-specific config was generated
        if let Some(ref kernel_config) = config.kernel_config_fragment {
            let kernel_config_path = config.buckos_build_path.join("hardware-kernel.config");
            std::fs::write(&kernel_config_path, kernel_config)?;
            tracing::info!("Wrote hardware-specific kernel config to: {}", kernel_config_path.display());
            update_progress("Building rootfs", 0.361, 0.06, "Saved hardware-specific kernel config");
        }

        // Check if Buck2 cache exists (for live CD installations with pre-built packages)
        let buck_out_path = config.buckos_build_path.join("buck-out");
        if buck_out_path.exists() {
            tracing::info!("Buck2 cache directory found at: {}", buck_out_path.display());
            update_progress("Building rootfs", 0.365, 0.08, "✓ Found Buck2 cache (using pre-built packages)");
        } else {
            tracing::info!("No Buck2 cache found, will build packages from scratch");
            update_progress("Building rootfs", 0.365, 0.08, "Building packages from scratch (no cache found)");
        }

        // Build the rootfs with Buck2
        // Buck2 will automatically use cached artifacts from buck-out if available
        update_progress("Building rootfs", 0.37, 0.1, "Running buck2 build...");

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

        let mut child = buck2_cmd.spawn()
            .map_err(|e| anyhow::anyhow!(
                "Failed to execute buck2 command: {}. Make sure buck2 is installed and in PATH.",
                e
            ))?;

        // Capture and process buck2 output in real-time
        use std::io::{BufRead, BufReader};

        let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("Failed to capture buck2 stderr"))?;
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
                    // Map buck2 progress (0.0-1.0) to our step progress (0.1-0.7)
                    let step_progress = 0.1 + (progress_info * 0.6);
                    update_progress("Building rootfs", 0.37 + (progress_info * 0.23), step_progress, &format!("Building: {}", line));
                    last_progress_update = std::time::Instant::now();
                }
            }

            // Log the output for debugging
            tracing::debug!("buck2: {}", line);
        }

        let output = child.wait_with_output()
            .map_err(|e| anyhow::anyhow!("Failed to wait for buck2 process: {}", e))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "Failed to build rootfs with buck2:\nstdout: {}\nstderr: {}",
                stdout, accumulated_output
            );
        }

        update_progress("Building rootfs", 0.60, 0.7, "✓ Rootfs built successfully");

        // Find the built rootfs directory
        update_progress("Building rootfs", 0.61, 0.75, "Locating built rootfs...");

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

        let show_output = show_output_cmd.output()
            .map_err(|e| anyhow::anyhow!("Failed to get buck2 output path: {}", e))?;

        let output_str = String::from_utf8_lossy(&show_output.stdout);
        let stderr_str = String::from_utf8_lossy(&show_output.stderr);
        tracing::debug!("buck2 --show-output stdout: {:?}", output_str);
        tracing::debug!("buck2 --show-output stderr: {:?}", stderr_str);
        tracing::debug!("buck2 --show-output exit status: {:?}", show_output.status);
        let rootfs_path = output_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .last()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| anyhow::anyhow!("Failed to parse buck2 output path. stdout={:?}, stderr={:?}, status={:?}",
                output_str, stderr_str, show_output.status))?;

        tracing::info!("Built rootfs at: {}", rootfs_path);

        // Copy the rootfs to target
        update_progress("Building rootfs", 0.62, 0.8, "Extracting rootfs to target...");

        // Buck2 returns a relative path from buckos_build_path, so make it absolute
        let rootfs_src = config.buckos_build_path.join(rootfs_path);
        if rootfs_src.is_dir() {
            // Copy directory contents (not the directory itself)
            // Use rsync or cp with /* to copy contents
            let rootfs_path_with_contents = format!("{}/*", rootfs_src.display());
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("cp -a {} {}", rootfs_path_with_contents, config.target_root.display()))
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to copy rootfs: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to copy rootfs to target: {}", stderr);
            }
        } else {
            anyhow::bail!("Expected rootfs directory at {}, but it doesn't exist or is not a directory", rootfs_src.display());
        }

        update_progress("Building rootfs", 0.70, 1.0, "✓ Rootfs installation complete");

        // Step 6: System configuration (80%)
        update_progress("System configuration", 0.72, 0.0, "Configuring system...");

        // Generate fstab
        update_progress("System configuration", 0.72, 0.1, "Generating /etc/fstab...");
        if let Some(disk_config) = &config.disk {
            let fstab_path = config.target_root.join("etc/fstab");
            std::fs::create_dir_all(fstab_path.parent().unwrap())?;

            let mut fstab_content = String::from("# /etc/fstab: static file system information\n");
            fstab_content.push_str("# <device>  <mount point>  <type>  <options>  <dump>  <pass>\n\n");

            for partition in &disk_config.partitions {
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
            update_progress("System configuration", 0.73, 0.2, "✓ Generated /etc/fstab");
        }

        // Configure locale
        update_progress("System configuration", 0.74, 0.3, "Configuring locale...");
        let locale_conf_path = config.target_root.join("etc/locale.conf");
        std::fs::write(&locale_conf_path,
            format!("LANG={}\n", config.locale.locale))?;

        let locale_gen_path = config.target_root.join("etc/locale.gen");
        std::fs::write(&locale_gen_path,
            format!("{} UTF-8\n", config.locale.locale.trim_end_matches(".UTF-8")))?;

        // Run locale-gen in chroot
        let _ = Command::new("chroot")
            .arg(&config.target_root)
            .env_clear()
            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
            .env("HOME", "/root")
            .env("TERM", "linux")
            .arg("locale-gen")
            .output();

        update_progress("System configuration", 0.75, 0.4, "✓ Configured locale");

        // Configure timezone
        update_progress("System configuration", 0.76, 0.5, "Configuring timezone...");
        let timezone_src = format!("/usr/share/zoneinfo/{}", config.timezone.timezone);
        let timezone_dst = config.target_root.join("etc/localtime");

        // Create symlink to timezone file
        let timezone_src_in_target = config.target_root.join(&timezone_src.trim_start_matches('/'));
        if timezone_src_in_target.exists() {
            if timezone_dst.exists() {
                std::fs::remove_file(&timezone_dst)?;
            }
            std::os::unix::fs::symlink(&timezone_src, &timezone_dst)?;
        }

        // Write timezone name
        let timezone_name_path = config.target_root.join("etc/timezone");
        std::fs::write(&timezone_name_path, format!("{}\n", config.timezone.timezone))?;

        update_progress("System configuration", 0.77, 0.6, "✓ Configured timezone");

        // Configure hostname
        update_progress("System configuration", 0.78, 0.7, "Configuring network...");
        let hostname_path = config.target_root.join("etc/hostname");
        std::fs::write(&hostname_path, format!("{}\n", config.network.hostname))?;

        // Write /etc/hosts
        let hosts_path = config.target_root.join("etc/hosts");
        let hosts_content = format!(
            "127.0.0.1\tlocalhost\n\
             ::1\t\tlocalhost\n\
             127.0.1.1\t{}\n",
            config.network.hostname
        );
        std::fs::write(&hosts_path, hosts_content)?;

        update_progress("System configuration", 0.79, 0.8, "✓ Configured network");

        // Configure keyboard layout
        update_progress("System configuration", 0.80, 0.9, "Configuring keyboard...");
        let vconsole_path = config.target_root.join("etc/vconsole.conf");
        std::fs::write(&vconsole_path,
            format!("KEYMAP={}\n", config.locale.keyboard))?;

        update_progress("System configuration", 0.80, 1.0, "✓ System configuration complete");

        // Step 7: Bootloader installation (90%)
        update_progress("Installing bootloader", 0.82, 0.0, "Installing bootloader...");

        let bootloader_name = match config.bootloader {
            crate::types::BootloaderType::Grub => "GRUB",
            crate::types::BootloaderType::Systemdboot => "systemd-boot",
            crate::types::BootloaderType::Refind => "rEFInd",
            crate::types::BootloaderType::Limine => "Limine",
            crate::types::BootloaderType::Efistub => "EFISTUB",
            crate::types::BootloaderType::None => "None",
        };

        if config.bootloader != crate::types::BootloaderType::None {
            update_progress("Installing bootloader", 0.83, 0.1,
                format!("Installing {} bootloader...", bootloader_name).as_str());

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
                        update_progress("Installing bootloader", 0.90, 0.5, &warning_msg);
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
                        update_progress("Installing bootloader", 0.83, 0.1, "Verifying boot partition...");

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
                        update_progress("Installing bootloader", 0.835, 0.15, "Preparing chroot environment...");

                        let bind_mounts = vec![
                            ("/dev", "dev"),
                            ("/proc", "proc"),
                            ("/sys", "sys"),
                        ];

                        // Mount /dev, /proc, /sys into chroot
                        for (source, target) in &bind_mounts {
                            let target_path = config.target_root.join(target);
                            std::fs::create_dir_all(&target_path)?;

                            let output = Command::new("mount")
                                .args(&["--bind", source, target_path.to_str().unwrap()])
                                .output()
                                .map_err(|e| anyhow::anyhow!(
                                    "Failed to bind mount {} to {}: {}",
                                    source, target_path.display(), e
                                ))?;

                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                tracing::warn!("Failed to bind mount {}: {}", source, stderr);
                            } else {
                                tracing::info!("Bind mounted {} to {}", source, target_path.display());
                            }
                        }

                        // Also mount /run if it exists (needed for some GRUB configurations)
                        if PathBuf::from("/run").exists() {
                            let run_target = config.target_root.join("run");
                            std::fs::create_dir_all(&run_target)?;
                            let _ = Command::new("mount")
                                .args(&["--bind", "/run", run_target.to_str().unwrap()])
                                .output();
                        }

                        // Mount efivarfs for EFI systems (required for efibootmgr)
                        // Skip for removable media to prevent modifying host system's EFI variables
                        if is_efi && !is_removable && PathBuf::from("/sys/firmware/efi/efivars").exists() {
                            let efivars_target = config.target_root.join("sys/firmware/efi/efivars");
                            std::fs::create_dir_all(&efivars_target)?;
                            let output = Command::new("mount")
                                .args(&["-t", "efivarfs", "efivarfs", efivars_target.to_str().unwrap()])
                                .output();

                            if let Ok(output) = output {
                                if output.status.success() {
                                    tracing::info!("Mounted efivarfs in chroot");
                                } else {
                                    tracing::warn!("Failed to mount efivarfs: {}", String::from_utf8_lossy(&output.stderr));
                                }
                            }
                        } else if is_removable {
                            tracing::info!("Skipping efivarfs mount for removable media to protect host EFI variables");
                        }

                        update_progress("Installing bootloader", 0.84, 0.15, "Updating library cache...");

                        // Run ldconfig to update the dynamic linker cache
                        let ldconfig_output = Command::new("chroot")
                            .arg(&config.target_root)
                            .env_clear()
                            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                            .env("HOME", "/root")
                            .env("TERM", "linux")
                            .arg("ldconfig")
                            .output()
                            .map_err(|e| anyhow::anyhow!("Failed to run ldconfig: {}", e))?;

                        if !ldconfig_output.status.success() {
                            let stderr = String::from_utf8_lossy(&ldconfig_output.stderr);
                            tracing::warn!("ldconfig warning: {}", stderr);
                        } else {
                            tracing::info!("ldconfig completed successfully");
                        }

                        update_progress("Installing bootloader", 0.845, 0.2, "Running grub-install...");

                        // Create GRUB directory if it doesn't exist
                        let grub_dir = if is_efi {
                            config.target_root.join("boot/efi/EFI/BuckOS")
                        } else {
                            config.target_root.join("boot/grub")
                        };
                        std::fs::create_dir_all(&grub_dir)?;

                        // Install GRUB
                        let mut grub_install_cmd = Command::new("chroot");
                        grub_install_cmd
                            .arg(&config.target_root)
                            .env_clear()
                            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                            .env("HOME", "/root")
                            .env("TERM", "linux")
                            .arg("grub-install");

                        if is_efi {
                            grub_install_cmd
                                .arg("--target=x86_64-efi")
                                .arg("--efi-directory=/boot/efi")
                                .arg("--bootloader-id=BuckOS")
                                .arg("--recheck");

                            // For removable media, use --no-nvram to prevent modifying host EFI variables
                            if is_removable {
                                grub_install_cmd.arg("--no-nvram");
                                grub_install_cmd.arg("--removable");
                                tracing::info!("Installing GRUB for removable media (--no-nvram --removable)");
                            }

                            grub_install_cmd.arg(&boot_device);
                        } else {
                            grub_install_cmd
                                .arg("--target=i386-pc")
                                .arg("--recheck")
                                .arg(&boot_device);
                        }

                        let output = grub_install_cmd.output()
                            .map_err(|e| anyhow::anyhow!(
                                "Failed to execute grub-install command: {}. Make sure chroot is available.",
                                e
                            ))?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);

                            // Cleanup bind mounts before failing
                            for (_, target) in bind_mounts.iter().rev() {
                                let target_path = config.target_root.join(target);
                                let _ = Command::new("umount").arg(&target_path).output();
                            }

                            anyhow::bail!(
                                "Failed to install GRUB:\nstderr: {}\nstdout: {}\n\
                                Make sure the @grub package includes all necessary GRUB modules.",
                                stderr, stdout
                            );
                        }

                        tracing::info!("grub-install completed successfully");
                        update_progress("Installing bootloader", 0.86, 0.5, "Generating initramfs...");

                        // Detect kernel version from installed kernel modules
                        // First try /lib/modules/ directory (most reliable)
                        let modules_dir = config.target_root.join("lib/modules");
                        let kernel_version = std::fs::read_dir(&modules_dir)
                            .ok()
                            .and_then(|entries| {
                                entries
                                    .filter_map(|e| e.ok())
                                    .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                                    .map(|e| e.file_name().to_string_lossy().to_string())
                            })
                            .or_else(|| {
                                // Fallback: try to extract from vmlinuz filename
                                let boot_dir = config.target_root.join("boot");
                                std::fs::read_dir(&boot_dir)
                                    .ok()
                                    .and_then(|entries| {
                                        entries
                                            .filter_map(|e| e.ok())
                                            .find(|e| {
                                                e.file_name().to_string_lossy().starts_with("vmlinuz-")
                                            })
                                            .and_then(|e| {
                                                let name = e.file_name();
                                                let name_str = name.to_string_lossy();
                                                name_str.strip_prefix("vmlinuz-").map(|v| v.to_string())
                                            })
                                    })
                            })
                            .unwrap_or_else(|| "6.12.6".to_string());

                        tracing::info!("Detected kernel version: {}", kernel_version);

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
                            tracing::info!("Created symlink from /usr/lib/dracut to /usr/lib64/dracut");
                        }

                        // Generate initramfs with dracut
                        tracing::info!("Generating initramfs with dracut for kernel {}", kernel_version);
                        let initramfs_path = format!("/boot/initramfs-{}.img", kernel_version);
                        let dracut_output = Command::new("chroot")
                            .arg(&config.target_root)
                            .env_clear()
                            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                            .env("HOME", "/root")
                            .env("TERM", "linux")
                            .arg("/usr/bin/dracut")
                            .arg("--force")
                            .arg("--hostonly")
                            .arg(&initramfs_path)
                            .arg("--kver")
                            .arg(&kernel_version)
                            .output()
                            .map_err(|e| anyhow::anyhow!(
                                "Failed to execute dracut command: {}. Make sure dracut is installed in the rootfs.",
                                e
                            ))?;

                        if !dracut_output.status.success() {
                            let stderr = String::from_utf8_lossy(&dracut_output.stderr);
                            let stdout = String::from_utf8_lossy(&dracut_output.stdout);

                            // Cleanup bind mounts before failing
                            for (_, target) in bind_mounts.iter().rev() {
                                let target_path = config.target_root.join(target);
                                let _ = Command::new("umount").arg(&target_path).output();
                            }

                            anyhow::bail!(
                                "Failed to generate initramfs with dracut:\nstdout: {}\nstderr: {}\n\
                                The system will not boot without an initramfs. Please ensure:\n\
                                1. The dracut package is properly installed in the rootfs\n\
                                2. The getopt utility is available (provided by util-linux)\n\
                                3. All required kernel modules and firmware are present",
                                stdout, stderr
                            );
                        }

                        tracing::info!("Initramfs generated successfully");

                        update_progress("Installing bootloader", 0.87, 0.6, "Generating GRUB configuration...");

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
                            for (_, target) in bind_mounts.iter().rev() {
                                let target_path = config.target_root.join(target);
                                let _ = Command::new("umount").arg(&target_path).output();
                            }

                            anyhow::bail!(
                                "Failed to generate GRUB config:\nstderr: {}\nstdout: {}",
                                stderr, stdout
                            );
                        }

                        tracing::info!("grub-mkconfig completed successfully");

                        // Verify GRUB config was created
                        let grub_cfg_path = config.target_root.join("boot/grub/grub.cfg");
                        if !grub_cfg_path.exists() {
                            tracing::warn!("GRUB config file not found at {}", grub_cfg_path.display());
                        } else {
                            tracing::info!("GRUB config created at {}", grub_cfg_path.display());
                        }

                        // Cleanup: Unmount bind mounts in reverse order
                        update_progress("Installing bootloader", 0.89, 0.9, "Cleaning up chroot environment...");

                        // Unmount /run if we mounted it
                        if PathBuf::from("/run").exists() {
                            let run_target = config.target_root.join("run");
                            let _ = Command::new("umount").arg(&run_target).output();
                        }

                        for (_, target) in bind_mounts.iter().rev() {
                            let target_path = config.target_root.join(target);
                            let output = Command::new("umount")
                                .arg(&target_path)
                                .output();

                            match output {
                                Ok(out) if out.status.success() => {
                                    tracing::info!("Unmounted {}", target_path.display());
                                }
                                Ok(out) => {
                                    let stderr = String::from_utf8_lossy(&out.stderr);
                                    tracing::warn!("Failed to unmount {}: {}", target_path.display(), stderr);
                                    // Try lazy unmount
                                    let _ = Command::new("umount")
                                        .args(&["-l", target_path.to_str().unwrap()])
                                        .output();
                                }
                                Err(e) => {
                                    tracing::warn!("Error unmounting {}: {}", target_path.display(), e);
                                }
                            }
                        }

                        tracing::info!("GRUB installation completed successfully");
                    }
                }

                crate::types::BootloaderType::Systemdboot => {
                    update_progress("Installing bootloader", 0.84, 0.3, "Running bootctl install...");

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

                    update_progress("Installing bootloader", 0.87, 0.6, "Creating systemd-boot entries...");

                    // Create loader configuration
                    let loader_conf_path = config.target_root.join("boot/loader/loader.conf");
                    std::fs::create_dir_all(loader_conf_path.parent().unwrap())?;
                    let loader_conf = "default buckos.conf\ntimeout 3\nconsole-mode max\neditor no\n";
                    std::fs::write(&loader_conf_path, loader_conf)?;

                    // Create boot entry
                    let entries_dir = config.target_root.join("boot/loader/entries");
                    std::fs::create_dir_all(&entries_dir)?;

                    // Find root partition UUID
                    let root_uuid = if let Some(disk_config) = &config.disk {
                        if let Some(root_part) = disk_config.partitions.iter()
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
                        "title   BuckOs\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux.img\noptions root=UUID={} rw\n",
                        root_uuid
                    );
                    std::fs::write(&entry_path, entry_content)?;
                }

                crate::types::BootloaderType::Refind => {
                    update_progress("Installing bootloader", 0.84, 0.3, "Installing rEFInd...");

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

                    update_progress("Installing bootloader", 0.87, 0.6, "Configuring rEFInd...");

                    // Create basic refind.conf if it doesn't exist
                    let refind_conf_path = config.target_root.join("boot/efi/EFI/refind/refind.conf");
                    if !refind_conf_path.exists() {
                        let refind_conf = "timeout 5\nuse_graphics_for linux\nscanfor manual,external,optical,internal\n";
                        std::fs::write(&refind_conf_path, refind_conf)?;
                    }
                }

                crate::types::BootloaderType::Limine => {
                    update_progress("Installing bootloader", 0.84, 0.3, "Installing Limine...");

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

                    update_progress("Installing bootloader", 0.87, 0.6, "Creating Limine configuration...");

                    // Create Limine configuration
                    let limine_cfg_path = config.target_root.join("boot/limine.cfg");
                    let limine_cfg = "TIMEOUT=5\n\n:BuckOs\nPROTOCOL=linux\nKERNEL_PATH=boot:///vmlinuz-linux\nKERNEL_CMDLINE=root=/dev/sda2 rw\nMODULE_PATH=boot:///initramfs-linux.img\n";
                    std::fs::write(&limine_cfg_path, limine_cfg)?;
                }

                crate::types::BootloaderType::Efistub => {
                    update_progress("Installing bootloader", 0.84, 0.3, "Creating EFISTUB boot entry...");

                    // Check if installing to removable media
                    let is_removable = config.disk.as_ref().map(|d| d.removable).unwrap_or(false);

                    if is_removable {
                        tracing::warn!("EFISTUB bootloader is not recommended for removable media");
                        tracing::warn!("Skipping efibootmgr to prevent modifying host EFI variables");
                        update_progress("Installing bootloader", 0.90, 1.0,
                            "⚠ EFISTUB installation skipped for removable media. Use GRUB or systemd-boot instead.");
                    } else {
                        // Find root partition
                        let root_dev = if let Some(disk_config) = &config.disk {
                            if let Some(root_part) = disk_config.partitions.iter()
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

            update_progress("Installing bootloader", 0.90, 1.0,
                format!("✓ {} bootloader installed", bootloader_name).as_str());
        } else {
            update_progress("Installing bootloader", 0.90, 1.0, "✓ Skipped bootloader installation");
        }

        // Step 8: User creation (95%)
        update_progress("Creating users", 0.92, 0.0, "Verifying user management utilities...");

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
        update_progress("Creating users", 0.92, 0.1, "✓ User management utilities verified");

        // Initialize /etc/passwd, /etc/shadow, and /etc/group if they don't exist
        update_progress("Creating users", 0.92, 0.15, "Initializing user database...");
        let passwd_path = config.target_root.join("etc/passwd");
        let shadow_path = config.target_root.join("etc/shadow");
        let group_path = config.target_root.join("etc/group");

        if !passwd_path.exists() {
            // Create basic /etc/passwd with root user
            let passwd_content = "root:x:0:0:root:/root:/bin/bash\n";
            std::fs::write(&passwd_path, passwd_content)
                .map_err(|e| anyhow::anyhow!("Failed to create /etc/passwd: {}", e))?;
            tracing::info!("Created /etc/passwd");
        }

        if !shadow_path.exists() {
            // Create basic /etc/shadow with root user (locked password)
            let shadow_content = "root:!:19000:0:99999:7:::\n";
            std::fs::write(&shadow_path, shadow_content)
                .map_err(|e| anyhow::anyhow!("Failed to create /etc/shadow: {}", e))?;
            // Set proper permissions on /etc/shadow (600)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o600);
                std::fs::set_permissions(&shadow_path, perms)
                    .map_err(|e| anyhow::anyhow!("Failed to set permissions on /etc/shadow: {}", e))?;
            }
            tracing::info!("Created /etc/shadow with proper permissions");
        }

        if !group_path.exists() {
            // Create basic /etc/group with root group
            let group_content = "root:x:0:\n";
            std::fs::write(&group_path, group_content)
                .map_err(|e| anyhow::anyhow!("Failed to create /etc/group: {}", e))?;
            tracing::info!("Created /etc/group");
        }

        // Create PAM system-auth configuration if it doesn't exist
        let pam_d_path = config.target_root.join("etc/pam.d");
        std::fs::create_dir_all(&pam_d_path)
            .map_err(|e| anyhow::anyhow!("Failed to create /etc/pam.d: {}", e))?;

        let system_auth_path = pam_d_path.join("system-auth");
        if !system_auth_path.exists() {
            // Create basic system-auth PAM configuration
            // Use module names without paths - PAM will search in /lib64/security (configured via --enable-securedir)
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
            std::fs::write(&system_auth_path, system_auth_content)
                .map_err(|e| anyhow::anyhow!("Failed to create /etc/pam.d/system-auth: {}", e))?;
            tracing::info!("Created /etc/pam.d/system-auth");
        }

        update_progress("Creating users", 0.92, 0.2, "Setting root password...");

        // Set root password using chpasswd
        let root_passwd_cmd = format!("root:{}", config.root_password);
        let output = Command::new("chroot")
            .arg(&config.target_root)
            .env_clear()
            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
            .env("HOME", "/root")
            .env("TERM", "linux")
            .arg("/usr/sbin/chpasswd")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(root_passwd_cmd.as_bytes())?;
                }
                child.wait_with_output()
            })
            .map_err(|e| anyhow::anyhow!(
                "Failed to execute chroot/chpasswd command for root: {}. Make sure chroot and passwd utilities are available.",
                e
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to set root password: {}", stderr);
        }

        update_progress("Creating users", 0.93, 0.3, "✓ Root password set");

        // Create user accounts
        for (idx, user) in config.users.iter().enumerate() {
            let step = 0.3 + ((idx as f32 + 1.0) / config.users.len() as f32) * 0.7;
            update_progress("Creating users", 0.93 + (step * 0.02), step,
                format!("Creating user: {}", user.username).as_str());

            // Create user with useradd
            let mut useradd_cmd = Command::new("chroot");
            useradd_cmd
                .arg(&config.target_root)
                .env_clear()
                .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                .env("HOME", "/root")
                .env("TERM", "linux")
                .arg("useradd")
                .arg("-m") // Create home directory
                .arg("-s").arg(&user.shell);

            if !user.full_name.is_empty() {
                useradd_cmd.arg("-c").arg(&user.full_name);
            }

            if user.is_admin {
                useradd_cmd.arg("-G").arg("wheel,sudo");
            }

            useradd_cmd.arg(&user.username);

            let output = useradd_cmd.output()
                .map_err(|e| anyhow::anyhow!(
                    "Failed to execute chroot/useradd command for user {}: {}. Make sure chroot and user utilities are available.",
                    user.username, e
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                update_progress("Creating users", 0.93 + (step * 0.02), step,
                    format!("Warning: Failed to create user {}: {}", user.username, stderr).as_str());
                continue;
            }

            // Set user password
            let user_passwd_cmd = format!("{}:{}", user.username, user.password);
            let output = Command::new("chroot")
                .arg(&config.target_root)
                .env_clear()
                .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                .env("HOME", "/root")
                .env("TERM", "linux")
                .arg("/usr/sbin/chpasswd")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(user_passwd_cmd.as_bytes())?;
                    }
                    child.wait_with_output()
                })
                .map_err(|e| anyhow::anyhow!(
                    "Failed to execute chroot/chpasswd command for user {}: {}. Make sure chroot and passwd utilities are available.",
                    user.username, e
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                update_progress("Creating users", 0.93 + (step * 0.02), step,
                    format!("Warning: Failed to set password for {}: {}", user.username, stderr).as_str());
            }
        }

        update_progress("Creating users", 0.95, 1.0, "✓ User accounts created");

        // Step 9: Finalization (100%)
        update_progress("Finalizing installation", 0.97, 0.5, "Cleaning up...");
        update_progress("Finalizing installation", 0.99, 0.9, "Unmounting filesystems...");
        update_progress("Installation complete", 1.0, 1.0, "✓ Installation completed successfully!");

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
