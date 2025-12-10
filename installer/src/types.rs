//! Common types for the BuckOS installer

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Desktop environment choices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Xfce,
    Mate,
    Cinnamon,
    LxQt,
    I3,
    Sway,
    Hyprland,
    None,
}

impl DesktopEnvironment {
    pub fn name(&self) -> &'static str {
        match self {
            DesktopEnvironment::Gnome => "GNOME",
            DesktopEnvironment::Kde => "KDE Plasma",
            DesktopEnvironment::Xfce => "Xfce",
            DesktopEnvironment::Mate => "MATE",
            DesktopEnvironment::Cinnamon => "Cinnamon",
            DesktopEnvironment::LxQt => "LXQt",
            DesktopEnvironment::I3 => "i3 (tiling WM)",
            DesktopEnvironment::Sway => "Sway (Wayland tiling)",
            DesktopEnvironment::Hyprland => "Hyprland (Wayland)",
            DesktopEnvironment::None => "None (minimal X/Wayland)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DesktopEnvironment::Gnome => "Modern, user-friendly desktop with GNOME Shell",
            DesktopEnvironment::Kde => "Feature-rich desktop with extensive customization",
            DesktopEnvironment::Xfce => "Lightweight, fast, and traditional desktop",
            DesktopEnvironment::Mate => "Traditional desktop, continuation of GNOME 2",
            DesktopEnvironment::Cinnamon => "Modern desktop with traditional layout",
            DesktopEnvironment::LxQt => "Lightweight Qt-based desktop environment",
            DesktopEnvironment::I3 => "Tiling window manager for power users",
            DesktopEnvironment::Sway => "Wayland compositor compatible with i3",
            DesktopEnvironment::Hyprland => "Dynamic tiling Wayland compositor",
            DesktopEnvironment::None => "Only basic display server, configure manually",
        }
    }

    pub fn package_set(&self) -> &'static str {
        match self {
            DesktopEnvironment::Gnome => "@gnome",
            DesktopEnvironment::Kde => "@kde",
            DesktopEnvironment::Xfce => "@xfce",
            DesktopEnvironment::Mate => "@mate",
            DesktopEnvironment::Cinnamon => "@cinnamon",
            DesktopEnvironment::LxQt => "@lxqt",
            DesktopEnvironment::I3 => "@i3",
            DesktopEnvironment::Sway => "@sway",
            DesktopEnvironment::Hyprland => "@hyprland",
            DesktopEnvironment::None => "@xorg-minimal",
        }
    }

    pub fn all() -> Vec<DesktopEnvironment> {
        vec![
            DesktopEnvironment::Gnome,
            DesktopEnvironment::Kde,
            DesktopEnvironment::Xfce,
            DesktopEnvironment::Mate,
            DesktopEnvironment::Cinnamon,
            DesktopEnvironment::LxQt,
            DesktopEnvironment::I3,
            DesktopEnvironment::Sway,
            DesktopEnvironment::Hyprland,
            DesktopEnvironment::None,
        ]
    }
}

/// Handheld/gaming device type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HandheldDevice {
    SteamDeck,
    AyaNeo,
    GpdWin,
    LegionGo,
    RogAlly,
    Generic,
}

impl HandheldDevice {
    pub fn name(&self) -> &'static str {
        match self {
            HandheldDevice::SteamDeck => "Steam Deck",
            HandheldDevice::AyaNeo => "AYA NEO",
            HandheldDevice::GpdWin => "GPD Win",
            HandheldDevice::LegionGo => "Lenovo Legion Go",
            HandheldDevice::RogAlly => "ASUS ROG Ally",
            HandheldDevice::Generic => "Generic Handheld",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HandheldDevice::SteamDeck => "Valve's portable gaming PC with APU",
            HandheldDevice::AyaNeo => "AYA NEO handheld gaming devices",
            HandheldDevice::GpdWin => "GPD Win portable gaming devices",
            HandheldDevice::LegionGo => "Lenovo Legion Go gaming handheld",
            HandheldDevice::RogAlly => "ASUS ROG Ally gaming handheld",
            HandheldDevice::Generic => "Generic gaming handheld or console",
        }
    }

    pub fn all() -> Vec<HandheldDevice> {
        vec![
            HandheldDevice::SteamDeck,
            HandheldDevice::AyaNeo,
            HandheldDevice::GpdWin,
            HandheldDevice::LegionGo,
            HandheldDevice::RogAlly,
            HandheldDevice::Generic,
        ]
    }
}

/// Represents an installation profile/preset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallProfile {
    /// Minimal system with just base utilities
    Minimal,
    /// Desktop environment with common applications
    Desktop(DesktopEnvironment),
    /// Server configuration with services
    Server,
    /// Handheld gaming device configuration
    Handheld(HandheldDevice),
    /// Custom selection
    Custom,
}

impl InstallProfile {
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            InstallProfile::Minimal => "Base system with essential utilities only",
            InstallProfile::Desktop(_) => "Full desktop environment with common applications",
            InstallProfile::Server => "Server configuration with common services",
            InstallProfile::Handheld(_) => "Gaming handheld with Steam and gaming optimizations",
            InstallProfile::Custom => "Select packages manually",
        }
    }

    pub fn package_sets(&self) -> Vec<&'static str> {
        match self {
            InstallProfile::Minimal => vec!["@system"],
            InstallProfile::Desktop(de) => vec![
                "@system",
                "@desktop",
                "@audio",
                "@network",
                de.package_set(),
            ],
            InstallProfile::Server => vec!["@system", "@server", "@network"],
            InstallProfile::Handheld(_) => vec![
                "@system", "@desktop", "@audio", "@network", "@gaming", "@steam",
            ],
            InstallProfile::Custom => vec!["@system"],
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            InstallProfile::Minimal => "Minimal",
            InstallProfile::Desktop(_) => "Desktop",
            InstallProfile::Server => "Server",
            InstallProfile::Handheld(_) => "Handheld",
            InstallProfile::Custom => "Custom",
        }
    }
}

impl Default for InstallProfile {
    fn default() -> Self {
        InstallProfile::Desktop(DesktopEnvironment::Gnome)
    }
}

/// GPU vendor for driver selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    VirtualBox,
    VMware,
    Unknown,
}

impl GpuVendor {
    pub fn driver_packages(&self) -> Vec<&'static str> {
        match self {
            GpuVendor::Nvidia => vec!["nvidia-drivers", "nvidia-settings"],
            GpuVendor::Amd => vec!["mesa", "vulkan-radeon", "libva-mesa-driver"],
            GpuVendor::Intel => vec!["mesa", "vulkan-intel", "intel-media-driver"],
            GpuVendor::VirtualBox => vec!["virtualbox-guest-additions"],
            GpuVendor::VMware => vec!["open-vm-tools", "xf86-video-vmware"],
            GpuVendor::Unknown => vec!["mesa"],
        }
    }
}

/// Detected GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    pub pci_id: String,
}

/// Network interface type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkInterfaceType {
    Ethernet,
    Wifi,
    Bridge,
    Virtual,
    Unknown,
}

/// Detected network interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub interface_type: NetworkInterfaceType,
    pub mac_address: Option<String>,
    pub driver: Option<String>,
}

/// Kernel version channel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KernelChannel {
    LTS,      // Long-term support (e.g., 6.6 LTS)
    Stable,   // Latest stable (e.g., 6.17)
    Mainline, // Rolling/bleeding edge
}

impl KernelChannel {
    pub fn name(&self) -> &'static str {
        match self {
            KernelChannel::LTS => "LTS (Long-term Support)",
            KernelChannel::Stable => "Stable",
            KernelChannel::Mainline => "Mainline (Rolling)",
        }
    }

    pub fn description(&self, buckos_build_path: Option<&std::path::Path>) -> String {
        // Query version dynamically from Buck
        let version = Self::query_version(self, buckos_build_path)
            .unwrap_or_else(|_| "unknown".to_string());
        match self {
            KernelChannel::LTS => format!("LTS {} - Long-term support, maximum stability (recommended)", version),
            KernelChannel::Stable => {
                format!("Stable {} - Latest stable kernel, balance of new features and stability", version)
            }
            KernelChannel::Mainline => format!("Mainline {} - Cutting edge features, frequent updates", version),
        }
    }

    /// Query the actual kernel version from Buck build files
    pub fn query_version(&self, buckos_build_path: Option<&std::path::Path>) -> anyhow::Result<String> {
        use std::process::Command;
        use std::path::PathBuf;

        // Use provided path or find buckos-build directory
        let working_dir = if let Some(path) = buckos_build_path {
            path.to_path_buf()
        } else {
            std::env::var("BUCKOS_BUILD_PATH")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    // Try common locations
                    for path in &[
                        "/var/db/repos/buckos-build",
                        "../buckos-build",
                        "../../buckos-build",
                    ] {
                        let p = PathBuf::from(path);
                        if p.exists() && p.join(".buckconfig").exists() {
                            return Some(p);
                        }
                    }
                    None
                })
                .ok_or_else(|| anyhow::anyhow!("Could not find buckos-build directory"))?
        };

        let source_target = match self {
            KernelChannel::LTS => "//packages/linux/kernel/src:linux-lts-src",
            KernelChannel::Stable => "//packages/linux/kernel/src:linux-stable-src",
            KernelChannel::Mainline => "//packages/linux/kernel/src:linux-mainline-src",
        };

        // First, resolve the alias to get the actual source target
        let output = Command::new("buck2")
            .current_dir(&working_dir)
            .args(&["uquery", source_target, "--output-all-attributes"])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to query kernel source target from Buck");
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse JSON to get the actual source target from deps
        let actual_target = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
            json.as_object()
                .and_then(|o| o.values().next())
                .and_then(|v| v.get("buck.deps"))
                .and_then(|deps| deps.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let archive_target = if let Some(target) = actual_target {
            format!("{}-archive", target)
        } else {
            anyhow::bail!("Could not resolve kernel source target");
        };

        // Query the archive target for URLs
        let output = Command::new("buck2")
            .current_dir(&working_dir)
            .args(&["uquery", &archive_target, "--output-all-attributes"])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to query kernel archive target from Buck");
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse JSON to extract URL
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
            if let Some(url) = json.as_object()
                .and_then(|o| o.values().next())
                .and_then(|v| v.get("urls"))
                .and_then(|urls| urls.as_array())
                .and_then(|arr| arr.first())
                .and_then(|u| u.as_str())
            {
                // Extract version from URL like "linux-6.17.10.tar.xz"
                if let Some(version) = url
                    .split("linux-").nth(1)
                    .and_then(|s| s.split(".tar").next())
                {
                    return Ok(version.to_string());
                }
            }
        }

        anyhow::bail!("Could not extract kernel version from Buck query")
    }

    pub fn package_target(&self) -> &'static str {
        match self {
            // LTS: Server-optimized for stability
            KernelChannel::LTS => "\"//packages/linux/kernel/buckos-kernel:buckos-kernel-server\"",
            // Stable: Default balanced configuration
            KernelChannel::Stable => "\"//packages/linux/kernel/buckos-kernel:buckos-kernel\"",
            // Mainline: Minimal for latest features
            KernelChannel::Mainline => {
                "\"//packages/linux/kernel/buckos-kernel:buckos-kernel-minimal\""
            }
        }
    }

    pub fn all() -> Vec<KernelChannel> {
        vec![
            KernelChannel::LTS,
            KernelChannel::Stable,
            KernelChannel::Mainline,
        ]
    }
}

impl Default for KernelChannel {
    fn default() -> Self {
        KernelChannel::Stable
    }
}

/// Init system type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InitSystem {
    Systemd,
    OpenRC,
    Runit,
    S6,
    SysVinit,
    Dinit,
    BusyBoxInit,
}

impl InitSystem {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            InitSystem::Systemd => "systemd",
            InitSystem::OpenRC => "OpenRC",
            InitSystem::Runit => "runit",
            InitSystem::S6 => "s6",
            InitSystem::SysVinit => "SysVinit",
            InitSystem::Dinit => "dinit",
            InitSystem::BusyBoxInit => "BusyBox init",
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            InitSystem::Systemd => "Modern init system and service manager (recommended)",
            InitSystem::OpenRC => "Dependency-based init system",
            InitSystem::Runit => "Simple init with service supervision",
            InitSystem::S6 => "Small and secure init system",
            InitSystem::SysVinit => "Traditional SysV init system",
            InitSystem::Dinit => "Service manager and init system",
            InitSystem::BusyBoxInit => "Minimal init from BusyBox (included in @system)",
        }
    }

    #[allow(dead_code)]
    pub fn package_set(&self) -> &'static str {
        match self {
            InitSystem::Systemd => "@systemd",
            InitSystem::OpenRC => "@openrc",
            InitSystem::Runit => "@runit",
            InitSystem::S6 => "@s6",
            InitSystem::SysVinit => "@sysvinit",
            InitSystem::Dinit => "@dinit",
            InitSystem::BusyBoxInit => "@busybox-init",
        }
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<InitSystem> {
        vec![
            InitSystem::Systemd,
            InitSystem::OpenRC,
            InitSystem::Runit,
            InitSystem::S6,
            InitSystem::SysVinit,
            InitSystem::Dinit,
            InitSystem::BusyBoxInit,
        ]
    }
}

/// Audio subsystem type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioSubsystem {
    PipeWire,
    PulseAudio,
    Alsa,
}

impl AudioSubsystem {
    pub fn name(&self) -> &'static str {
        match self {
            AudioSubsystem::PipeWire => "PipeWire",
            AudioSubsystem::PulseAudio => "PulseAudio",
            AudioSubsystem::Alsa => "ALSA only",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AudioSubsystem::PipeWire => "Modern multimedia server (recommended)",
            AudioSubsystem::PulseAudio => "Traditional Linux audio server",
            AudioSubsystem::Alsa => "Basic kernel-level audio (minimal)",
        }
    }

    pub fn package_set(&self) -> &'static str {
        match self {
            AudioSubsystem::PipeWire => "@pipewire",
            AudioSubsystem::PulseAudio => "@pulseaudio",
            AudioSubsystem::Alsa => "@alsa",
        }
    }
}

/// Detected audio device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub card_id: String,
    pub is_hdmi: bool,
}

/// Storage controller type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageControllerType {
    Nvme,
    Ahci,
    Raid,
    Virtio,
    Usb,
    Unknown,
}

/// Power profile for laptops/handhelds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PowerProfile {
    Desktop,
    Laptop,
    Gaming,
    Server,
}

impl PowerProfile {
    pub fn packages(&self) -> Vec<&'static str> {
        match self {
            PowerProfile::Desktop => vec![],
            PowerProfile::Laptop => vec!["tlp", "thermald", "powertop"],
            PowerProfile::Gaming => vec!["gamemode", "cpupower"],
            PowerProfile::Server => vec!["tuned"],
        }
    }
}

/// Complete hardware detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub gpus: Vec<GpuInfo>,
    pub network_interfaces: Vec<NetworkInterfaceInfo>,
    pub audio_devices: Vec<AudioDeviceInfo>,
    pub storage_controller: StorageControllerType,
    pub power_profile: PowerProfile,
    pub has_bluetooth: bool,
    pub has_touchscreen: bool,
    pub is_laptop: bool,
    pub is_virtual_machine: bool,
    pub cpu_vendor: String,
    pub cpu_flags: Vec<String>,
}

impl Default for HardwareInfo {
    fn default() -> Self {
        Self {
            gpus: Vec::new(),
            network_interfaces: Vec::new(),
            audio_devices: Vec::new(),
            storage_controller: StorageControllerType::Unknown,
            power_profile: PowerProfile::Desktop,
            has_bluetooth: false,
            has_touchscreen: false,
            is_laptop: false,
            is_virtual_machine: false,
            cpu_vendor: String::new(),
            cpu_flags: Vec::new(),
        }
    }
}

/// Suggested packages based on hardware detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwarePackageSuggestion {
    pub category: String,
    pub reason: String,
    pub packages: Vec<String>,
    pub selected: bool,
}

/// Disk layout preset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiskLayoutPreset {
    /// Simple: single root partition
    Simple,
    /// Standard: boot/efi + swap + root
    Standard,
    /// Separate home: boot/efi + swap + root + home
    SeparateHome,
    /// Server: boot/efi + swap + root + var + home
    Server,
    /// Btrfs with subvolumes
    BtrfsSubvolumes,
    /// Custom manual layout
    Custom,
}

impl DiskLayoutPreset {
    pub fn name(&self) -> &'static str {
        match self {
            DiskLayoutPreset::Simple => "Simple",
            DiskLayoutPreset::Standard => "Standard",
            DiskLayoutPreset::SeparateHome => "Separate /home",
            DiskLayoutPreset::Server => "Server Layout",
            DiskLayoutPreset::BtrfsSubvolumes => "Btrfs Subvolumes",
            DiskLayoutPreset::Custom => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DiskLayoutPreset::Simple => "Single root partition (simplest setup)",
            DiskLayoutPreset::Standard => "Boot/EFI + Swap + Root (recommended)",
            DiskLayoutPreset::SeparateHome => "Standard + separate /home partition",
            DiskLayoutPreset::Server => "Root + /var + /home for servers",
            DiskLayoutPreset::BtrfsSubvolumes => "Btrfs with @, @home, @snapshots subvolumes",
            DiskLayoutPreset::Custom => "Configure partitions manually",
        }
    }

    pub fn all() -> Vec<DiskLayoutPreset> {
        vec![
            DiskLayoutPreset::Standard,
            DiskLayoutPreset::Simple,
            DiskLayoutPreset::SeparateHome,
            DiskLayoutPreset::Server,
            DiskLayoutPreset::BtrfsSubvolumes,
            DiskLayoutPreset::Custom,
        ]
    }
}

/// Encryption type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionType {
    None,
    LuksRoot,
    LuksFull,
    LuksHome,
}

impl EncryptionType {
    pub fn name(&self) -> &'static str {
        match self {
            EncryptionType::None => "No Encryption",
            EncryptionType::LuksRoot => "Encrypt Root Only",
            EncryptionType::LuksFull => "Full Disk Encryption",
            EncryptionType::LuksHome => "Encrypt /home Only",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            EncryptionType::None => "No disk encryption (fastest)",
            EncryptionType::LuksRoot => "Encrypt root partition with LUKS",
            EncryptionType::LuksFull => "Encrypt all partitions except boot (most secure)",
            EncryptionType::LuksHome => "Only encrypt the home partition",
        }
    }

    pub fn all() -> Vec<EncryptionType> {
        vec![
            EncryptionType::None,
            EncryptionType::LuksRoot,
            EncryptionType::LuksFull,
            EncryptionType::LuksHome,
        ]
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub encryption_type: EncryptionType,
    pub passphrase: String,
    /// Use TPM to unlock automatically on trusted boot
    pub use_tpm: bool,
    /// LUKS key derivation function iterations (higher = more secure but slower)
    pub luks_pbkdf_iterations: Option<u32>,
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

    #[allow(dead_code)]
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
    /// Is this a removable/USB drive
    pub removable: bool,
}

/// Bootloader type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BootloaderType {
    Grub,
    Systemdboot,
    Refind,
    Efistub,
    Limine,
    None,
}

impl BootloaderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BootloaderType::Grub => "GRUB",
            BootloaderType::Systemdboot => "systemd-boot",
            BootloaderType::Refind => "rEFInd",
            BootloaderType::Efistub => "EFISTUB",
            BootloaderType::Limine => "Limine",
            BootloaderType::None => "None (manual)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BootloaderType::Grub => "Most compatible, supports BIOS and UEFI, many features",
            BootloaderType::Systemdboot => "Simple UEFI bootloader, minimal and fast",
            BootloaderType::Refind => "Graphical UEFI boot manager with auto-detection",
            BootloaderType::Efistub => "Boot kernel directly from UEFI (advanced)",
            BootloaderType::Limine => "Modern bootloader with multiboot support",
            BootloaderType::None => "Manual bootloader setup required",
        }
    }

    pub fn requires_uefi(&self) -> bool {
        matches!(
            self,
            BootloaderType::Systemdboot | BootloaderType::Refind | BootloaderType::Efistub
        )
    }

    pub fn all() -> Vec<BootloaderType> {
        vec![
            BootloaderType::Grub,
            BootloaderType::Systemdboot,
            BootloaderType::Refind,
            BootloaderType::Limine,
            BootloaderType::Efistub,
            BootloaderType::None,
        ]
    }

    pub fn all_for_bios() -> Vec<BootloaderType> {
        vec![
            BootloaderType::Grub,
            BootloaderType::Limine,
            BootloaderType::None,
        ]
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
    /// Path to buckos-build repository
    pub buckos_build_path: PathBuf,
    /// Installation profile
    pub profile: InstallProfile,
    /// Disk configuration
    pub disk: Option<DiskConfig>,
    /// Disk layout preset used
    pub disk_layout: DiskLayoutPreset,
    /// Encryption configuration
    pub encryption: EncryptionConfig,
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
    /// Init system choice
    pub init_system: InitSystem,
    /// Audio subsystem choice
    pub audio_subsystem: AudioSubsystem,
    /// Kernel channel choice
    pub kernel_channel: KernelChannel,
    /// Include all firmware in initramfs (disables hostonly mode)
    pub include_all_firmware: bool,
    /// Detected hardware information
    pub hardware_info: HardwareInfo,
    /// Hardware-based package suggestions
    pub hardware_packages: Vec<HardwarePackageSuggestion>,
    /// Hardware-based kernel config fragment
    pub kernel_config_fragment: Option<String>,
    /// Additional packages to install
    pub extra_packages: Vec<String>,
    /// Dry run mode
    pub dry_run: bool,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            target_root: PathBuf::from("/mnt/buckos"),
            buckos_build_path: PathBuf::from("/var/db/repos/buckos-build"),
            profile: InstallProfile::default(),
            disk: None,
            disk_layout: DiskLayoutPreset::Standard,
            encryption: EncryptionConfig {
                encryption_type: EncryptionType::None,
                passphrase: String::new(),
                use_tpm: false,
                luks_pbkdf_iterations: None,
            },
            bootloader: BootloaderType::Grub,
            root_password: String::new(),
            users: Vec::new(),
            network: NetworkConfig {
                hostname: "BuckOS".to_string(),
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
            init_system: InitSystem::Systemd,
            audio_subsystem: AudioSubsystem::PipeWire,
            kernel_channel: KernelChannel::default(),
            include_all_firmware: true,
            hardware_info: HardwareInfo::default(),
            hardware_packages: Vec::new(),
            kernel_config_fragment: None,
            extra_packages: Vec::new(),
            dry_run: false,
        }
    }
}

/// Represents the current installation step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStep {
    Welcome,
    HardwareDetection,
    ProfileSelection,
    KernelSelection,
    DiskSetup,
    Bootloader,
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
            InstallStep::HardwareDetection => "Hardware Detection",
            InstallStep::ProfileSelection => "Profile Selection",
            InstallStep::KernelSelection => "Kernel Selection",
            InstallStep::DiskSetup => "Disk Setup",
            InstallStep::Bootloader => "Bootloader",
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
            InstallStep::Welcome => Some(InstallStep::HardwareDetection),
            InstallStep::HardwareDetection => Some(InstallStep::ProfileSelection),
            InstallStep::ProfileSelection => Some(InstallStep::KernelSelection),
            InstallStep::KernelSelection => Some(InstallStep::DiskSetup),
            InstallStep::DiskSetup => Some(InstallStep::Bootloader),
            InstallStep::Bootloader => Some(InstallStep::UserSetup),
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
            InstallStep::HardwareDetection => Some(InstallStep::Welcome),
            InstallStep::ProfileSelection => Some(InstallStep::HardwareDetection),
            InstallStep::KernelSelection => Some(InstallStep::ProfileSelection),
            InstallStep::DiskSetup => Some(InstallStep::KernelSelection),
            InstallStep::Bootloader => Some(InstallStep::DiskSetup),
            InstallStep::UserSetup => Some(InstallStep::Bootloader),
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
            InstallStep::HardwareDetection => 1,
            InstallStep::ProfileSelection => 2,
            InstallStep::KernelSelection => 3,
            InstallStep::DiskSetup => 4,
            InstallStep::Bootloader => 5,
            InstallStep::UserSetup => 6,
            InstallStep::NetworkSetup => 7,
            InstallStep::Timezone => 8,
            InstallStep::Summary => 9,
            InstallStep::Installing => 10,
            InstallStep::Complete => 11,
        }
    }

    pub fn total_steps() -> usize {
        12
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
    #[allow(dead_code)]
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

impl InstallProgress {
    /// Maximum number of log lines to keep in buffer
    const MAX_LOG_LINES: usize = 1000;

    /// Add a log message, maintaining the 1000-line buffer limit
    pub fn add_log(&mut self, message: impl Into<String>) {
        self.log.push(message.into());
        // Keep only the last 1000 lines
        if self.log.len() > Self::MAX_LOG_LINES {
            self.log.drain(0..(self.log.len() - Self::MAX_LOG_LINES));
        }
    }

    /// Add an error message
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    /// Update operation and progress
    /// Progress values are clamped to prevent going backwards
    pub fn update(&mut self, operation: impl Into<String>, overall: f32, step: f32) {
        let new_operation = operation.into();

        // Only allow overall progress to increase (never go backwards)
        if overall > self.overall_progress {
            self.overall_progress = overall;
        }

        // Step progress resets when operation changes, otherwise only increases
        if new_operation != self.operation {
            self.step_progress = step;
        } else if step > self.step_progress {
            self.step_progress = step;
        }

        self.operation = new_operation;
    }
}
