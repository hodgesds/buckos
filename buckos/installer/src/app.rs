//! Main GUI application for the Buckos installer

use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::steps;
use crate::system;
use crate::types::{
    DiskInfo, FilesystemType, InstallConfig, InstallProfile, InstallProgress, InstallStep,
    MountPoint, PartitionConfig, UserConfig,
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
#[derive(Default)]
struct UiState {
    // Disk setup
    selected_disk_index: usize,
    auto_partition: bool,
    show_partition_editor: bool,

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

    // Extra packages
    extra_packages_text: String,

    // Errors
    validation_error: Option<String>,

    // Cached data
    timezones: Vec<String>,
    locales: Vec<String>,
    keyboards: Vec<String>,
}

impl InstallerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, target: String, dry_run: bool) -> Self {
        let available_disks = system::get_available_disks().unwrap_or_default();
        let system_info = system::get_system_info();

        let mut config = InstallConfig::default();
        config.target_root = PathBuf::from(target);
        config.dry_run = dry_run;

        // Initialize UI state with defaults
        let mut ui_state = UiState::default();
        ui_state.auto_partition = true;
        ui_state.hostname = config.network.hostname.clone();
        ui_state.timezones = system::get_timezones();
        ui_state.locales = system::get_locales();
        ui_state.keyboards = system::get_keyboard_layouts();

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
            InstallStep::DiskSetup => {
                // Need at least one disk available or manual setup
                !self.available_disks.is_empty() || !self.ui_state.auto_partition
            }
            InstallStep::ProfileSelection => true,
            InstallStep::UserSetup => {
                // Need root password
                !self.ui_state.root_password.is_empty()
                    && self.ui_state.root_password == self.ui_state.confirm_root_password
            }
            InstallStep::NetworkSetup => !self.ui_state.hostname.is_empty(),
            InstallStep::Timezone => true,
            InstallStep::Summary => true,
            InstallStep::Installing => false, // Can't proceed during installation
            InstallStep::Complete => false,
        }
    }

    fn validate_and_proceed(&mut self) -> bool {
        self.ui_state.validation_error = None;

        match self.current_step {
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
            InstallStep::DiskSetup => {
                if self.ui_state.auto_partition && !self.available_disks.is_empty() {
                    let disk = &self.available_disks[self.ui_state.selected_disk_index];
                    self.config.disk = Some(create_auto_partition_config(disk));
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
                    InstallStep::DiskSetup => steps::render_disk_setup(
                        ui,
                        &self.available_disks,
                        &mut self.ui_state.selected_disk_index,
                        &mut self.ui_state.auto_partition,
                    ),
                    InstallStep::ProfileSelection => {
                        steps::render_profile_selection(ui, &mut self.config.profile)
                    }
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
                    InstallStep::Summary => steps::render_summary(ui, &self.config, &self.available_disks, self.ui_state.selected_disk_index),
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
fn create_auto_partition_config(disk: &DiskInfo) -> crate::types::DiskConfig {
    let is_efi = system::is_efi_system();

    let mut partitions = Vec::new();

    if is_efi {
        // EFI System Partition
        partitions.push(PartitionConfig {
            device: format!("{}1", disk.device),
            size: 512 * 1024 * 1024, // 512 MB
            filesystem: FilesystemType::Fat32,
            mount_point: MountPoint::BootEfi,
            format: true,
            mount_options: String::new(),
        });
    } else {
        // BIOS boot partition
        partitions.push(PartitionConfig {
            device: format!("{}1", disk.device),
            size: 1024 * 1024, // 1 MB for BIOS boot
            filesystem: FilesystemType::None,
            mount_point: MountPoint::Boot,
            format: false,
            mount_options: String::new(),
        });
    }

    // Swap partition (size based on RAM, max 8GB)
    let swap_size = std::cmp::min(
        8 * 1024 * 1024 * 1024,
        sysinfo::System::new_all().total_memory() * 2,
    );
    partitions.push(PartitionConfig {
        device: format!("{}2", disk.device),
        size: swap_size,
        filesystem: FilesystemType::Swap,
        mount_point: MountPoint::Swap,
        format: true,
        mount_options: String::new(),
    });

    // Root partition (remaining space)
    partitions.push(PartitionConfig {
        device: format!("{}3", disk.device),
        size: 0, // Use remaining space
        filesystem: FilesystemType::Ext4,
        mount_point: MountPoint::Root,
        format: true,
        mount_options: String::new(),
    });

    crate::types::DiskConfig {
        device: disk.device.clone(),
        use_gpt: is_efi,
        partitions,
        wipe_disk: true,
    }
}
