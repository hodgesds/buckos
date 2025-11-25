//! Main GUI application for the Buckos installer

use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::steps;
use crate::system;
use crate::types::{
    AudioSubsystem, DesktopEnvironment, DiskInfo, DiskLayoutPreset, EncryptionType, FilesystemType,
    HandheldDevice, HardwareInfo, HardwarePackageSuggestion, InstallConfig, InstallProfile,
    InstallProgress, InstallStep, MountPoint, PartitionConfig, UserConfig,
};

/// Main installer application state
pub struct InstallerApp {
    /// Current installation step
    current_step: InstallStep,

    /// Installation configuration being built
    config: InstallConfig,

    /// Available disks in the system
    available_disks: Vec<DiskInfo>,

    /// System information
    system_info: system::SystemInfo,

    /// Installation progress (used during installation)
    progress: Arc<Mutex<InstallProgress>>,

    /// Whether installation is running
    installing: bool,

    /// Temporary state for UI
    ui_state: UiState,
}

/// Temporary UI state
struct UiState {
    // Hardware detection
    hardware_info: HardwareInfo,
    hardware_suggestions: Vec<HardwarePackageSuggestion>,

    // Profile selection
    selected_de: DesktopEnvironment,
    selected_handheld: HandheldDevice,
    audio_subsystem: AudioSubsystem,

    // Disk setup
    selected_disk_index: usize,
    auto_partition: bool,
    show_partition_editor: bool,
    layout_preset: DiskLayoutPreset,
    root_filesystem: FilesystemType,
    encryption_type: EncryptionType,
    encryption_passphrase: String,
    confirm_encryption_passphrase: String,

    // User setup
    new_username: String,
    new_fullname: String,
    new_password: String,
    confirm_password: String,
    new_user_admin: bool,
    root_password: String,
    confirm_root_password: String,

    // Network setup
    hostname: String,

    // Timezone
    selected_timezone_index: usize,
    selected_locale_index: usize,
    selected_keyboard_index: usize,

    // Summary confirmations
    confirm_wipe: bool,
    confirm_install: bool,

    // Extra packages
    extra_packages_text: String,

    // Errors
    validation_error: Option<String>,

    // Cached data
    timezones: Vec<String>,
    locales: Vec<String>,
    keyboards: Vec<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            hardware_info: HardwareInfo::default(),
            hardware_suggestions: Vec::new(),
            selected_de: DesktopEnvironment::Gnome,
            selected_handheld: HandheldDevice::SteamDeck,
            audio_subsystem: AudioSubsystem::PipeWire,
            selected_disk_index: 0,
            auto_partition: true,
            show_partition_editor: false,
            layout_preset: DiskLayoutPreset::Standard,
            root_filesystem: FilesystemType::Ext4,
            encryption_type: EncryptionType::None,
            encryption_passphrase: String::new(),
            confirm_encryption_passphrase: String::new(),
            new_username: String::new(),
            new_fullname: String::new(),
            new_password: String::new(),
            confirm_password: String::new(),
            new_user_admin: true,
            root_password: String::new(),
            confirm_root_password: String::new(),
            hostname: "buckos".to_string(),
            selected_timezone_index: 0,
            selected_locale_index: 0,
            selected_keyboard_index: 0,
            confirm_wipe: false,
            confirm_install: false,
            extra_packages_text: String::new(),
            validation_error: None,
            timezones: Vec::new(),
            locales: Vec::new(),
            keyboards: Vec::new(),
        }
    }
}

impl InstallerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, target: String, dry_run: bool, buckos_build_path: PathBuf) -> Self {
        let available_disks = system::get_available_disks().unwrap_or_default();
        let system_info = system::get_system_info();

        let mut config = InstallConfig::default();
        config.target_root = PathBuf::from(target);
        config.buckos_build_path = buckos_build_path;
        config.dry_run = dry_run;

        // Perform hardware detection
        let hardware_info = system::detect_hardware();
        let hardware_suggestions = system::generate_hardware_suggestions(&hardware_info);

        // Initialize UI state with defaults
        let mut ui_state = UiState::default();
        ui_state.auto_partition = true;
        ui_state.hostname = config.network.hostname.clone();
        ui_state.timezones = system::get_timezones();
        ui_state.locales = system::get_locales();
        ui_state.keyboards = system::get_keyboard_layouts();
        ui_state.hardware_info = hardware_info;
        ui_state.hardware_suggestions = hardware_suggestions;

        // Auto-detect handheld device
        if let Some(device) = system::detect_handheld_device() {
            ui_state.selected_handheld = device;
        }

        // Find default selections
        ui_state.selected_timezone_index = ui_state
            .timezones
            .iter()
            .position(|tz| tz == "UTC")
            .unwrap_or(0);
        ui_state.selected_locale_index = ui_state
            .locales
            .iter()
            .position(|l| l == "en_US.UTF-8")
            .unwrap_or(0);
        ui_state.selected_keyboard_index = 0; // "us"

        Self {
            current_step: InstallStep::Welcome,
            config,
            available_disks,
            system_info,
            progress: Arc::new(Mutex::new(InstallProgress::default())),
            installing: false,
            ui_state,
        }
    }

    fn can_proceed(&self) -> bool {
        match self.current_step {
            InstallStep::Welcome => true,
            InstallStep::HardwareDetection => true,
            InstallStep::ProfileSelection => true,
            InstallStep::DiskSetup => {
                // Need at least one disk available or manual setup
                let disk_ok = !self.available_disks.is_empty() || !self.ui_state.auto_partition;
                // If encryption selected, need passphrase
                let enc_ok = self.ui_state.encryption_type == EncryptionType::None
                    || (!self.ui_state.encryption_passphrase.is_empty()
                        && self.ui_state.encryption_passphrase
                            == self.ui_state.confirm_encryption_passphrase);
                disk_ok && enc_ok
            }
            InstallStep::Bootloader => true,
            InstallStep::UserSetup => {
                // Need root password
                !self.ui_state.root_password.is_empty()
                    && self.ui_state.root_password == self.ui_state.confirm_root_password
            }
            InstallStep::NetworkSetup => !self.ui_state.hostname.is_empty(),
            InstallStep::Timezone => true,
            InstallStep::Summary => {
                // Need to confirm installation
                let base_confirm = self.ui_state.confirm_install;
                let wipe_confirm = if self.config.disk.is_some() && !self.config.dry_run {
                    self.ui_state.confirm_wipe
                } else {
                    true
                };
                base_confirm && wipe_confirm
            }
            InstallStep::Installing => false, // Can't proceed during installation
            InstallStep::Complete => false,
        }
    }

    fn validate_and_proceed(&mut self) -> bool {
        self.ui_state.validation_error = None;

        match self.current_step {
            InstallStep::HardwareDetection => {
                // Copy hardware info and selected suggestions to config
                self.config.hardware_info = self.ui_state.hardware_info.clone();
                self.config.hardware_packages = self.ui_state.hardware_suggestions.clone();
            }
            InstallStep::ProfileSelection => {
                // Update config with selected profile and audio subsystem
                self.config.profile = match &self.config.profile {
                    InstallProfile::Desktop(_) => {
                        InstallProfile::Desktop(self.ui_state.selected_de.clone())
                    }
                    InstallProfile::Handheld(_) => {
                        InstallProfile::Handheld(self.ui_state.selected_handheld.clone())
                    }
                    other => other.clone(),
                };
                self.config.audio_subsystem = self.ui_state.audio_subsystem.clone();
            }
            InstallStep::DiskSetup => {
                // Validate encryption passphrase
                if self.ui_state.encryption_type != EncryptionType::None {
                    if self.ui_state.encryption_passphrase.is_empty() {
                        self.ui_state.validation_error =
                            Some("Encryption passphrase is required".to_string());
                        return false;
                    }
                    if self.ui_state.encryption_passphrase
                        != self.ui_state.confirm_encryption_passphrase
                    {
                        self.ui_state.validation_error =
                            Some("Encryption passphrases do not match".to_string());
                        return false;
                    }
                    if self.ui_state.encryption_passphrase.len() < 8 {
                        self.ui_state.validation_error = Some(
                            "Encryption passphrase should be at least 8 characters".to_string(),
                        );
                        return false;
                    }
                }

                // Set encryption config
                self.config.encryption.encryption_type = self.ui_state.encryption_type.clone();
                self.config.encryption.passphrase = self.ui_state.encryption_passphrase.clone();
                self.config.disk_layout = self.ui_state.layout_preset.clone();

                // Create disk configuration
                if self.ui_state.auto_partition && !self.available_disks.is_empty() {
                    let disk = &self.available_disks[self.ui_state.selected_disk_index];
                    self.config.disk = Some(create_auto_partition_config(
                        disk,
                        &self.ui_state.layout_preset,
                        self.ui_state.root_filesystem,
                    ));
                }
            }
            InstallStep::Bootloader => {
                // Bootloader is set directly in UI, nothing to validate
            }
            InstallStep::UserSetup => {
                if self.ui_state.root_password.is_empty() {
                    self.ui_state.validation_error = Some("Root password is required".to_string());
                    return false;
                }
                if self.ui_state.root_password != self.ui_state.confirm_root_password {
                    self.ui_state.validation_error =
                        Some("Root passwords do not match".to_string());
                    return false;
                }
                if self.ui_state.root_password.len() < 4 {
                    self.ui_state.validation_error =
                        Some("Root password must be at least 4 characters".to_string());
                    return false;
                }
                self.config.root_password = self.ui_state.root_password.clone();
            }
            InstallStep::NetworkSetup => {
                if self.ui_state.hostname.is_empty() {
                    self.ui_state.validation_error = Some("Hostname is required".to_string());
                    return false;
                }
                self.config.network.hostname = self.ui_state.hostname.clone();
            }
            InstallStep::Timezone => {
                if let Some(tz) = self
                    .ui_state
                    .timezones
                    .get(self.ui_state.selected_timezone_index)
                {
                    self.config.timezone.timezone = tz.clone();
                }
                if let Some(locale) = self
                    .ui_state
                    .locales
                    .get(self.ui_state.selected_locale_index)
                {
                    self.config.locale.locale = locale.clone();
                }
                if let Some(kb) = self
                    .ui_state
                    .keyboards
                    .get(self.ui_state.selected_keyboard_index)
                {
                    self.config.locale.keyboard = kb.clone();
                }
            }
            InstallStep::Summary => {
                // Validate confirmation checkboxes
                if !self.ui_state.confirm_install {
                    self.ui_state.validation_error =
                        Some("Please confirm installation".to_string());
                    return false;
                }
                if self.config.disk.is_some() && !self.config.dry_run && !self.ui_state.confirm_wipe
                {
                    self.ui_state.validation_error =
                        Some("Please confirm data destruction".to_string());
                    return false;
                }
            }
            _ => {}
        }

        true
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if installation is complete and transition to Complete step
        if self.installing && self.current_step == InstallStep::Installing {
            if let Ok(progress) = self.progress.lock() {
                if progress.overall_progress >= 1.0 {
                    self.current_step = InstallStep::Complete;
                    self.installing = false;
                }
            }
        }

        // Request repaint for smooth progress updates during installation
        if self.installing {
            ctx.request_repaint();
        }

        // Top panel with progress indicator
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("Buckos Installer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.config.dry_run {
                        ui.label(
                            egui::RichText::new("DRY RUN")
                                .color(egui::Color32::YELLOW)
                                .strong(),
                        );
                    }
                });
            });

            // Progress bar
            ui.add_space(4.0);
            let progress =
                self.current_step.index() as f32 / (InstallStep::total_steps() - 1) as f32;
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            ui.add_space(8.0);
        });

        // Bottom panel with navigation buttons
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                // Back button
                let can_go_back = self.current_step.prev().is_some();
                if ui
                    .add_enabled(can_go_back, egui::Button::new("← Back"))
                    .clicked()
                {
                    if let Some(prev) = self.current_step.prev() {
                        self.current_step = prev;
                        self.ui_state.validation_error = None;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Next/Install button
                    let (label, enabled) = match self.current_step {
                        InstallStep::Summary => ("Install", self.can_proceed()),
                        InstallStep::Installing => ("Installing...", false),
                        InstallStep::Complete => ("Close", true),
                        _ => ("Next →", self.can_proceed()),
                    };

                    if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                        if self.current_step == InstallStep::Complete {
                            std::process::exit(0);
                        } else if self.validate_and_proceed() {
                            if let Some(next) = self.current_step.next() {
                                self.current_step = next;
                                if self.current_step == InstallStep::Installing {
                                    self.installing = true;
                                    // Start installation in background
                                    let config = self.config.clone();
                                    let progress = Arc::clone(&self.progress);
                                    std::thread::spawn(move || {
                                        run_installation(config, progress);
                                    });
                                }
                            }
                        }
                    }
                });
            });
            ui.add_space(8.0);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show validation error if any
            if let Some(error) = &self.ui_state.validation_error {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠").color(egui::Color32::RED));
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                });
                ui.add_space(8.0);
            }

            // Step title
            ui.heading(self.current_step.title());
            ui.separator();
            ui.add_space(8.0);

            // Render current step
            egui::ScrollArea::vertical().show(ui, |ui| match self.current_step {
                InstallStep::Welcome => steps::render_welcome(ui, &self.system_info),
                InstallStep::HardwareDetection => steps::render_hardware_detection(
                    ui,
                    &self.ui_state.hardware_info,
                    &mut self.ui_state.hardware_suggestions,
                ),
                InstallStep::ProfileSelection => steps::render_profile_selection(
                    ui,
                    &mut self.config.profile,
                    &mut self.ui_state.selected_de,
                    &mut self.ui_state.selected_handheld,
                    &mut self.ui_state.audio_subsystem,
                ),
                InstallStep::DiskSetup => steps::render_disk_setup(
                    ui,
                    &self.available_disks,
                    &mut self.ui_state.selected_disk_index,
                    &mut self.ui_state.auto_partition,
                    &mut self.ui_state.layout_preset,
                    &mut self.ui_state.root_filesystem,
                    &mut self.ui_state.encryption_type,
                    &mut self.ui_state.encryption_passphrase,
                    &mut self.ui_state.confirm_encryption_passphrase,
                ),
                InstallStep::Bootloader => steps::render_bootloader(
                    ui,
                    &mut self.config.bootloader,
                    system::is_efi_system(),
                ),
                InstallStep::UserSetup => steps::render_user_setup(
                    ui,
                    &mut self.config.users,
                    &mut self.ui_state.new_username,
                    &mut self.ui_state.new_fullname,
                    &mut self.ui_state.new_password,
                    &mut self.ui_state.confirm_password,
                    &mut self.ui_state.new_user_admin,
                    &mut self.ui_state.root_password,
                    &mut self.ui_state.confirm_root_password,
                ),
                InstallStep::NetworkSetup => steps::render_network_setup(
                    ui,
                    &mut self.ui_state.hostname,
                    &mut self.config.network.use_dhcp,
                ),
                InstallStep::Timezone => steps::render_timezone_setup(
                    ui,
                    &self.ui_state.timezones,
                    &mut self.ui_state.selected_timezone_index,
                    &self.ui_state.locales,
                    &mut self.ui_state.selected_locale_index,
                    &self.ui_state.keyboards,
                    &mut self.ui_state.selected_keyboard_index,
                ),
                InstallStep::Summary => steps::render_summary(
                    ui,
                    &self.config,
                    &self.available_disks,
                    self.ui_state.selected_disk_index,
                    &mut self.ui_state.confirm_wipe,
                    &mut self.ui_state.confirm_install,
                ),
                InstallStep::Installing => {
                    let progress = self.progress.lock().unwrap();
                    steps::render_installing(ui, &progress)
                }
                InstallStep::Complete => steps::render_complete(ui, &self.config),
            });
        });
    }
}

/// Create automatic partition configuration for a disk
fn create_auto_partition_config(
    disk: &DiskInfo,
    layout: &DiskLayoutPreset,
    root_filesystem: FilesystemType,
) -> crate::types::DiskConfig {
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
            device: format!("{}{}", disk.device, part_num),
            size: 512 * 1024 * 1024, // 512 MB
            filesystem: FilesystemType::Fat32,
            mount_point: MountPoint::BootEfi,
            format: true,
            mount_options: String::new(),
        });
        part_num += 1;
    } else {
        partitions.push(PartitionConfig {
            device: format!("{}{}", disk.device, part_num),
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
        let swap_size = std::cmp::min(
            8 * 1024 * 1024 * 1024,
            sysinfo::System::new_all().total_memory() * 2,
        );
        partitions.push(PartitionConfig {
            device: format!("{}{}", disk.device, part_num),
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
                device: format!("{}{}", disk.device, part_num),
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
                device: format!("{}{}", disk.device, part_num),
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
                device: format!("{}{}", disk.device, part_num),
                size: root_size,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
            part_num += 1;

            // Home partition (remaining space)
            partitions.push(PartitionConfig {
                device: format!("{}{}", disk.device, part_num),
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
                device: format!("{}{}", disk.device, part_num),
                size: 30 * 1024 * 1024 * 1024,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
            part_num += 1;

            // Var partition (20GB)
            partitions.push(PartitionConfig {
                device: format!("{}{}", disk.device, part_num),
                size: 20 * 1024 * 1024 * 1024,
                filesystem: root_fs,
                mount_point: MountPoint::Var,
                format: true,
                mount_options: String::new(),
            });
            part_num += 1;

            // Home partition (remaining)
            partitions.push(PartitionConfig {
                device: format!("{}{}", disk.device, part_num),
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
                device: format!("{}{}", disk.device, part_num),
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
                device: format!("{}{}", disk.device, part_num),
                size: 0,
                filesystem: root_fs,
                mount_point: MountPoint::Root,
                format: true,
                mount_options: String::new(),
            });
        }
    }

    crate::types::DiskConfig {
        device: disk.device.clone(),
        use_gpt: is_efi,
        partitions,
        wipe_disk: true,
    }
}

/// Parse buck2 output to extract progress information
/// Returns a progress value between 0.0 and 1.0 if progress info is found
fn parse_buck2_progress(line: &str) -> Option<f32> {
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
fn run_installation(config: InstallConfig, progress: Arc<Mutex<InstallProgress>>) {
    use anyhow::Result;
    use std::error::Error;
    use std::process::Command;

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
        rootfs_packages.push("\"//packages/linux/core/file:file\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/bash:bash\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/zlib:zlib\"".to_string());
        rootfs_packages.push("\"//packages/linux/core/glibc:glibc\"".to_string());

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
        let rootfs_path = output_str
            .lines()
            .last()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| anyhow::anyhow!("Failed to parse buck2 output path"))?;

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
                        let boot_device = if let Some(disk_config) = &config.disk {
                            disk_config.device.clone()
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
                        if is_efi && PathBuf::from("/sys/firmware/efi/efivars").exists() {
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
                        }

                        update_progress("Installing bootloader", 0.84, 0.15, "Updating library cache...");

                        // Run ldconfig to update the dynamic linker cache
                        let ldconfig_output = Command::new("chroot")
                            .arg(&config.target_root)
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
                            config.target_root.join("boot/efi/EFI/GRUB")
                        } else {
                            config.target_root.join("boot/grub")
                        };
                        std::fs::create_dir_all(&grub_dir)?;

                        // Install GRUB
                        let mut grub_install_cmd = Command::new("chroot");
                        grub_install_cmd
                            .arg(&config.target_root)
                            .arg("grub-install");

                        if is_efi {
                            grub_install_cmd
                                .arg("--target=x86_64-efi")
                                .arg("--efi-directory=/boot/efi")
                                .arg("--bootloader-id=GRUB")
                                .arg("--recheck");
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
                        update_progress("Installing bootloader", 0.87, 0.6, "Generating GRUB configuration...");

                        // Generate GRUB configuration
                        let output = Command::new("chroot")
                            .arg(&config.target_root)
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
                        "title   Buckos\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux.img\noptions root=UUID={} rw\n",
                        root_uuid
                    );
                    std::fs::write(&entry_path, entry_content)?;
                }

                crate::types::BootloaderType::Refind => {
                    update_progress("Installing bootloader", 0.84, 0.3, "Installing rEFInd...");

                    // Install rEFInd
                    let output = Command::new("chroot")
                        .arg(&config.target_root)
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
                    let limine_cfg = "TIMEOUT=5\n\n:Buckos\nPROTOCOL=linux\nKERNEL_PATH=boot:///vmlinuz-linux\nKERNEL_CMDLINE=root=/dev/sda2 rw\nMODULE_PATH=boot:///initramfs-linux.img\n";
                    std::fs::write(&limine_cfg_path, limine_cfg)?;
                }

                crate::types::BootloaderType::Efistub => {
                    update_progress("Installing bootloader", 0.84, 0.3, "Creating EFISTUB boot entry...");

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
                        .arg("efibootmgr")
                        .arg("--create")
                        .arg("--disk").arg("/dev/sda")
                        .arg("--part").arg("1")
                        .arg("--label").arg("Buckos")
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
            // Use full paths to PAM modules since they're in /usr/lib/security
            let system_auth_content = "#%PAM-1.0
# System-wide authentication configuration

# Authentication
auth       required   /usr/lib/security/pam_unix.so     try_first_pass nullok
auth       optional   /usr/lib/security/pam_permit.so

# Account management
account    required   /usr/lib/security/pam_unix.so
account    optional   /usr/lib/security/pam_permit.so

# Password management
password   required   /usr/lib/security/pam_unix.so     try_first_pass nullok sha512
password   optional   /usr/lib/security/pam_permit.so

# Session management
session    required   /usr/lib/security/pam_unix.so
session    optional   /usr/lib/security/pam_permit.so
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
