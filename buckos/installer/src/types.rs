//! Common types for the Buckos installer

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents an installation profile/preset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallProfile {
    /// Minimal system with just base utilities
    Minimal,
    /// Desktop environment with common applications
    Desktop,
    /// Server configuration with services
    Server,
    /// Custom selection
    Custom,
}

impl InstallProfile {
    pub fn description(&self) -> &'static str {
        match self {
            InstallProfile::Minimal => "Base system with essential utilities only",
            InstallProfile::Desktop => "Full desktop environment with common applications",
            InstallProfile::Server => "Server configuration with common services",
            InstallProfile::Custom => "Select packages manually",
        }
    }

    pub fn package_sets(&self) -> Vec<&'static str> {
        match self {
            InstallProfile::Minimal => vec!["@system"],
            InstallProfile::Desktop => vec!["@system", "@desktop", "@audio", "@network"],
            InstallProfile::Server => vec!["@system", "@server", "@network"],
            InstallProfile::Custom => vec!["@system"],
        }
    }
}

/// Filesystem type for partitions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FilesystemType {
    Ext4,
    Btrfs,
    Xfs,
    F2fs,
    Swap,
    Fat32,
    None,
}

impl FilesystemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilesystemType::Ext4 => "ext4",
            FilesystemType::Btrfs => "btrfs",
            FilesystemType::Xfs => "xfs",
            FilesystemType::F2fs => "f2fs",
            FilesystemType::Swap => "swap",
            FilesystemType::Fat32 => "vfat",
            FilesystemType::None => "none",
        }
    }

    pub fn mkfs_command(&self) -> Option<&'static str> {
        match self {
            FilesystemType::Ext4 => Some("mkfs.ext4"),
            FilesystemType::Btrfs => Some("mkfs.btrfs"),
            FilesystemType::Xfs => Some("mkfs.xfs"),
            FilesystemType::F2fs => Some("mkfs.f2fs"),
            FilesystemType::Swap => Some("mkswap"),
            FilesystemType::Fat32 => Some("mkfs.vfat"),
            FilesystemType::None => None,
        }
    }
}

/// Mount point configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MountPoint {
    Root,
    Boot,
    BootEfi,
    Home,
    Var,
    Swap,
    Custom(String),
}

impl MountPoint {
    pub fn path(&self) -> &str {
        match self {
            MountPoint::Root => "/",
            MountPoint::Boot => "/boot",
            MountPoint::BootEfi => "/boot/efi",
            MountPoint::Home => "/home",
            MountPoint::Var => "/var",
            MountPoint::Swap => "swap",
            MountPoint::Custom(p) => p,
        }
    }
}

/// Partition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    /// Device path (e.g., /dev/sda1)
    pub device: String,
    /// Size in bytes (0 = use remaining space)
    pub size: u64,
    /// Filesystem type
    pub filesystem: FilesystemType,
    /// Mount point
    pub mount_point: MountPoint,
    /// Format this partition
    pub format: bool,
    /// Mount options
    pub mount_options: String,
}

/// Disk configuration for installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskConfig {
    /// Target disk device (e.g., /dev/sda)
    pub device: String,
    /// Use GPT partition table
    pub use_gpt: bool,
    /// Partitions to create/use
    pub partitions: Vec<PartitionConfig>,
    /// Wipe the entire disk
    pub wipe_disk: bool,
}

/// Bootloader type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BootloaderType {
    Grub,
    Systemdboot,
    None,
}

impl BootloaderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BootloaderType::Grub => "GRUB",
            BootloaderType::Systemdboot => "systemd-boot",
            BootloaderType::None => "None (manual)",
        }
    }
}

/// User account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Username
    pub username: String,
    /// Full name
    pub full_name: String,
    /// Password (will be hashed)
    pub password: String,
    /// Add to wheel/sudo group
    pub is_admin: bool,
    /// Shell (e.g., /bin/bash)
    pub shell: String,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Hostname
    pub hostname: String,
    /// Use DHCP for network configuration
    pub use_dhcp: bool,
    /// Static IP address (if not using DHCP)
    pub static_ip: Option<String>,
    /// Gateway (if not using DHCP)
    pub gateway: Option<String>,
    /// DNS servers
    pub dns_servers: Vec<String>,
}

/// Timezone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneConfig {
    /// Timezone (e.g., "America/New_York")
    pub timezone: String,
    /// Use NTP for time synchronization
    pub use_ntp: bool,
}

/// Locale configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleConfig {
    /// System locale (e.g., "en_US.UTF-8")
    pub locale: String,
    /// Keyboard layout (e.g., "us")
    pub keyboard: String,
}

/// Complete installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Target installation directory
    pub target_root: PathBuf,
    /// Installation profile
    pub profile: InstallProfile,
    /// Disk configuration
    pub disk: Option<DiskConfig>,
    /// Bootloader configuration
    pub bootloader: BootloaderType,
    /// Root password
    pub root_password: String,
    /// User accounts to create
    pub users: Vec<UserConfig>,
    /// Network configuration
    pub network: NetworkConfig,
    /// Timezone configuration
    pub timezone: TimezoneConfig,
    /// Locale configuration
    pub locale: LocaleConfig,
    /// Additional packages to install
    pub extra_packages: Vec<String>,
    /// Dry run mode
    pub dry_run: bool,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            target_root: PathBuf::from("/mnt/buckos"),
            profile: InstallProfile::Desktop,
            disk: None,
            bootloader: BootloaderType::Grub,
            root_password: String::new(),
            users: Vec::new(),
            network: NetworkConfig {
                hostname: "buckos".to_string(),
                use_dhcp: true,
                static_ip: None,
                gateway: None,
                dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            },
            timezone: TimezoneConfig {
                timezone: "UTC".to_string(),
                use_ntp: true,
            },
            locale: LocaleConfig {
                locale: "en_US.UTF-8".to_string(),
                keyboard: "us".to_string(),
            },
            extra_packages: Vec::new(),
            dry_run: false,
        }
    }
}

/// Represents the current installation step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStep {
    Welcome,
    DiskSetup,
    ProfileSelection,
    UserSetup,
    NetworkSetup,
    Timezone,
    Summary,
    Installing,
    Complete,
}

impl InstallStep {
    pub fn title(&self) -> &'static str {
        match self {
            InstallStep::Welcome => "Welcome",
            InstallStep::DiskSetup => "Disk Setup",
            InstallStep::ProfileSelection => "Profile Selection",
            InstallStep::UserSetup => "User Setup",
            InstallStep::NetworkSetup => "Network Setup",
            InstallStep::Timezone => "Timezone & Locale",
            InstallStep::Summary => "Summary",
            InstallStep::Installing => "Installing",
            InstallStep::Complete => "Complete",
        }
    }

    pub fn next(&self) -> Option<InstallStep> {
        match self {
            InstallStep::Welcome => Some(InstallStep::DiskSetup),
            InstallStep::DiskSetup => Some(InstallStep::ProfileSelection),
            InstallStep::ProfileSelection => Some(InstallStep::UserSetup),
            InstallStep::UserSetup => Some(InstallStep::NetworkSetup),
            InstallStep::NetworkSetup => Some(InstallStep::Timezone),
            InstallStep::Timezone => Some(InstallStep::Summary),
            InstallStep::Summary => Some(InstallStep::Installing),
            InstallStep::Installing => Some(InstallStep::Complete),
            InstallStep::Complete => None,
        }
    }

    pub fn prev(&self) -> Option<InstallStep> {
        match self {
            InstallStep::Welcome => None,
            InstallStep::DiskSetup => Some(InstallStep::Welcome),
            InstallStep::ProfileSelection => Some(InstallStep::DiskSetup),
            InstallStep::UserSetup => Some(InstallStep::ProfileSelection),
            InstallStep::NetworkSetup => Some(InstallStep::UserSetup),
            InstallStep::Timezone => Some(InstallStep::NetworkSetup),
            InstallStep::Summary => Some(InstallStep::Timezone),
            InstallStep::Installing => None, // Can't go back during installation
            InstallStep::Complete => None,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            InstallStep::Welcome => 0,
            InstallStep::DiskSetup => 1,
            InstallStep::ProfileSelection => 2,
            InstallStep::UserSetup => 3,
            InstallStep::NetworkSetup => 4,
            InstallStep::Timezone => 5,
            InstallStep::Summary => 6,
            InstallStep::Installing => 7,
            InstallStep::Complete => 8,
        }
    }

    pub fn total_steps() -> usize {
        9
    }
}

/// Disk information from system
#[derive(Debug, Clone)]
pub struct DiskInfo {
    /// Device path
    pub device: String,
    /// Model name
    pub model: String,
    /// Size in bytes
    pub size: u64,
    /// Is removable media
    pub removable: bool,
    /// Existing partitions
    pub partitions: Vec<PartitionInfo>,
}

/// Partition information from system
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    /// Device path
    pub device: String,
    /// Size in bytes
    pub size: u64,
    /// Filesystem type (if detected)
    pub filesystem: Option<String>,
    /// Mount point (if mounted)
    pub mount_point: Option<String>,
    /// Partition label
    pub label: Option<String>,
}

/// Installation progress information
#[derive(Debug, Clone)]
pub struct InstallProgress {
    /// Current operation description
    pub operation: String,
    /// Overall progress (0.0 - 1.0)
    pub overall_progress: f32,
    /// Current step progress (0.0 - 1.0)
    pub step_progress: f32,
    /// Log messages
    pub log: Vec<String>,
    /// Errors encountered
    pub errors: Vec<String>,
}

impl Default for InstallProgress {
    fn default() -> Self {
        Self {
            operation: "Initializing...".to_string(),
            overall_progress: 0.0,
            step_progress: 0.0,
            log: Vec::new(),
            errors: Vec::new(),
        }
    }
}
