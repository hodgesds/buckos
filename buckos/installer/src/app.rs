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
    pub fn new(_cc: &eframe::CreationContext<'_>, target: String, dry_run: bool) -> Self {
        let available_disks = system::get_available_disks().unwrap_or_default();
        let system_info = system::get_system_info();

        let mut config = InstallConfig::default();
        config.target_root = PathBuf::from(target);
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
                        && self.ui_state.encryption_passphrase == self.ui_state.confirm_encryption_passphrase);
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
                    InstallProfile::Desktop(_) => InstallProfile::Desktop(self.ui_state.selected_de.clone()),
                    InstallProfile::Handheld(_) => InstallProfile::Handheld(self.ui_state.selected_handheld.clone()),
                    other => other.clone(),
                };
                self.config.audio_subsystem = self.ui_state.audio_subsystem.clone();
            }
            InstallStep::DiskSetup => {
                // Validate encryption passphrase
                if self.ui_state.encryption_type != EncryptionType::None {
                    if self.ui_state.encryption_passphrase.is_empty() {
                        self.ui_state.validation_error = Some("Encryption passphrase is required".to_string());
                        return false;
                    }
                    if self.ui_state.encryption_passphrase != self.ui_state.confirm_encryption_passphrase {
                        self.ui_state.validation_error = Some("Encryption passphrases do not match".to_string());
                        return false;
                    }
                    if self.ui_state.encryption_passphrase.len() < 8 {
                        self.ui_state.validation_error =
                            Some("Encryption passphrase should be at least 8 characters".to_string());
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
                    self.ui_state.validation_error = Some("Root passwords do not match".to_string());
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
                if let Some(tz) = self.ui_state.timezones.get(self.ui_state.selected_timezone_index) {
                    self.config.timezone.timezone = tz.clone();
                }
                if let Some(locale) = self.ui_state.locales.get(self.ui_state.selected_locale_index) {
                    self.config.locale.locale = locale.clone();
                }
                if let Some(kb) = self.ui_state.keyboards.get(self.ui_state.selected_keyboard_index) {
                    self.config.locale.keyboard = kb.clone();
                }
            }
            InstallStep::Summary => {
                // Validate confirmation checkboxes
                if !self.ui_state.confirm_install {
                    self.ui_state.validation_error = Some("Please confirm installation".to_string());
                    return false;
                }
                if self.config.disk.is_some() && !self.config.dry_run && !self.ui_state.confirm_wipe {
                    self.ui_state.validation_error = Some("Please confirm data destruction".to_string());
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
            let progress = self.current_step.index() as f32 / (InstallStep::total_steps() - 1) as f32;
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
                                    // TODO: Implement actual installation
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.current_step {
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
                }
            });
        });
    }
}

/// Create automatic partition configuration for a disk
fn create_auto_partition_config(disk: &DiskInfo, layout: &DiskLayoutPreset) -> crate::types::DiskConfig {
    let is_efi = system::is_efi_system();
    let mut partitions = Vec::new();
    let mut part_num = 1;

    // Determine filesystem type based on layout
    let root_fs = match layout {
        DiskLayoutPreset::BtrfsSubvolumes => FilesystemType::Btrfs,
        _ => FilesystemType::Ext4,
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
