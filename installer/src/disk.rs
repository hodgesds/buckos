//! Disk partitioning and configuration

use crate::system;
use crate::types::{
    DiskConfig, DiskInfo, DiskLayoutPreset, FilesystemType, MountPoint, PartitionConfig,
};

/// Construct the correct partition device path for a given disk and partition number.
/// NVMe devices (nvmeXnY) and some other devices (mmcblk, loop) require a 'p' separator
/// before the partition number, while SATA/SCSI devices (sdX) do not.
fn partition_device_path(disk_device: &str, part_num: u32) -> String {
    // Devices that need 'p' separator before partition number
    let needs_p_separator = disk_device.contains("nvme")
        || disk_device.contains("mmcblk")
        || disk_device.contains("loop");

    if needs_p_separator {
        format!("{}p{}", disk_device, part_num)
    } else {
        format!("{}{}", disk_device, part_num)
    }
}

/// Create automatic partition configuration for a disk
pub fn create_auto_partition_config(
    disk: &DiskInfo,
    layout: &DiskLayoutPreset,
    root_filesystem: FilesystemType,
) -> DiskConfig {
    let is_efi = system::is_efi_system();
    let mut partitions = Vec::new();
    let mut part_num = 1;

    // Use user-selected filesystem, or force Btrfs for BtrfsSubvolumes layout
    let root_fs = match layout {
        DiskLayoutPreset::BtrfsSubvolumes => FilesystemType::Btrfs,
        _ => root_filesystem,
    };

    // Boot/EFI partition
    if is_efi {
        partitions.push(PartitionConfig {
            device: partition_device_path(&disk.device, part_num),
            size: 512 * 1024 * 1024, // 512 MB
            filesystem: FilesystemType::Fat32,
            mount_point: MountPoint::BootEfi,
            format: true,
            mount_options: String::new(),
        });
        part_num += 1;
    } else {
        partitions.push(PartitionConfig {
            device: partition_device_path(&disk.device, part_num),
            size: 1024 * 1024, // 1 MB for BIOS boot
            filesystem: FilesystemType::None,
            mount_point: MountPoint::Boot,
            format: false,
            mount_options: String::new(),
        });
        part_num += 1;
    }

    // Swap partition (for all layouts except Simple)
    if !matches!(layout, DiskLayoutPreset::Simple) {
        // Use smaller swap for removable media (1GB) vs internal drives (min of 8GB or 2x RAM)
        tracing::info!(
            "Disk {} is {}",
            disk.device,
            if disk.removable {
                "removable"
            } else {
                "non-removable"
            }
        );

        let desired_swap = if disk.removable {
            1 * 1024 * 1024 * 1024 // 1 GB for USB drives
        } else {
            std::cmp::min(
                8 * 1024 * 1024 * 1024,
                sysinfo::System::new_all().total_memory() * 2,
            )
        };

        // Safety check: Cap swap at 20% of total disk size to prevent partition errors
        let max_swap_for_disk = (disk.size as f64 * 0.20) as u64;
        let swap_size = std::cmp::min(desired_swap, max_swap_for_disk);
        tracing::info!(
            "Swap size: desired={} MB, max_for_disk={} MB, final={} MB",
            desired_swap / 1024 / 1024,
            max_swap_for_disk / 1024 / 1024,
            swap_size / 1024 / 1024
        );
        partitions.push(PartitionConfig {
            device: partition_device_path(&disk.device, part_num),
            size: swap_size,
            filesystem: FilesystemType::Swap,
            mount_point: MountPoint::Swap,
            format: true,
            mount_options: String::new(),
        });
        part_num += 1;
    }

    // Layout-specific partitions
    match layout {
        DiskLayoutPreset::Simple => {
            // Single root partition
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 0,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
        }
        DiskLayoutPreset::Standard => {
            // Root partition only
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 0,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
        }
        DiskLayoutPreset::SeparateHome => {
            // Root partition (50GB or 50% of remaining, whichever is smaller)
            let root_size = std::cmp::min(50 * 1024 * 1024 * 1024, disk.size / 2);
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: root_size,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
            part_num += 1;

            // Home partition (remaining space)
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 0,
                filesystem: root_fs,
                mount_point: MountPoint::Home,
                format: true,
                mount_options: String::new(),
            });
        }
        DiskLayoutPreset::Server => {
            // Root partition (30GB)
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 30 * 1024 * 1024 * 1024,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
            part_num += 1;

            // Var partition (20GB)
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 20 * 1024 * 1024 * 1024,
                filesystem: root_fs,
                mount_point: MountPoint::Var,
                format: true,
                mount_options: String::new(),
            });
            part_num += 1;

            // Home partition (remaining)
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 0,
                filesystem: root_fs,
                mount_point: MountPoint::Home,
                format: true,
                mount_options: String::new(),
            });
        }
        DiskLayoutPreset::BtrfsSubvolumes => {
            // Single btrfs partition with subvolumes
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 0,
                filesystem: FilesystemType::Btrfs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: "subvol=@,compress=zstd".to_string(),
            });
            // Note: Subvolumes (@, @home, @snapshots) will be created during installation
        }
        DiskLayoutPreset::Custom => {
            // Custom layout - user will configure manually
            // Just create a basic root partition as placeholder
            partitions.push(PartitionConfig {
                device: partition_device_path(&disk.device, part_num),
                size: 0,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
        }
    }

    DiskConfig {
        device: disk.device.clone(),
        use_gpt: is_efi,
        partitions,
        wipe_disk: true,
        removable: disk.removable,
    }
}
