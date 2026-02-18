//! Main TUI application logic

use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::types::{
    AudioSubsystem, BootloaderType, DesktopEnvironment, DiskInfo, DiskLayoutPreset, EncryptionType,
    FilesystemType, HandheldDevice, HardwareInfo, HardwarePackageSuggestion, InitSystem,
    InstallConfig, InstallProfile, InstallProgress, InstallStep, KernelChannel, SystemLimitsConfig,
    SystemTuningProfile, UserConfig,
};
use crate::{disk, install, system};

use super::widgets::{Checkbox, HelpBar, InfoBox, TextInput};

/// Focus state for UI elements
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum FocusField {
    // Generic
    List,
    // Disk setup
    DiskList,
    LayoutPreset,
    Filesystem,
    Encryption,
    EncryptionPassword,
    EncryptionConfirm,
    // User setup
    RootPassword,
    RootPasswordConfirm,
    Username,
    FullName,
    UserPassword,
    UserPasswordConfirm,
    UserAdmin,
    AddUserButton,
    // Network
    Hostname,
    UseDhcp,
    // Timezone
    Timezone,
    Locale,
    Keyboard,
    // Summary
    ConfirmWipe,
    ConfirmInstall,
}

/// Main TUI application state
pub struct TuiApp {
    /// Current installation step
    current_step: InstallStep,
    /// Installation configuration being built
    config: InstallConfig,
    /// Available disks in the system
    available_disks: Vec<DiskInfo>,
    /// System information
    system_info: system::SystemInfo,
    /// Installation progress
    progress: Arc<Mutex<InstallProgress>>,
    /// Whether installation is running
    installing: bool,
    /// UI state
    ui: UiState,
    /// Should quit
    should_quit: bool,
}

/// UI state for selections and inputs
#[allow(dead_code)]
struct UiState {
    // Hardware detection
    hardware_info: HardwareInfo,
    hardware_suggestions: Vec<HardwarePackageSuggestion>,
    hardware_list_state: ListState,

    // Profile selection
    profile_list_state: ListState,
    de_list_state: ListState,
    handheld_list_state: ListState,
    audio_list_state: ListState,
    selected_profile_category: usize, // 0=Desktop, 1=Server, 2=Handheld, 3=Minimal, 4=Custom

    // Kernel selection
    kernel_list_state: ListState,
    init_list_state: ListState,
    include_all_firmware: bool,

    // Disk setup
    disk_list_state: ListState,
    layout_list_state: ListState,
    fs_list_state: ListState,
    encryption_list_state: ListState,
    encryption_password: String,
    encryption_password_confirm: String,
    auto_partition: bool,

    // Bootloader
    bootloader_list_state: ListState,

    // System tuning
    tuning_list_state: ListState,
    system_limits: SystemLimitsConfig,

    // User setup
    root_password: String,
    root_password_confirm: String,
    new_username: String,
    new_fullname: String,
    new_password: String,
    new_password_confirm: String,
    new_user_admin: bool,
    users_list_state: ListState,

    // Network
    hostname: String,
    use_dhcp: bool,

    // Timezone
    timezones: Vec<String>,
    timezone_list_state: ListState,
    locales: Vec<String>,
    locale_list_state: ListState,
    keyboards: Vec<String>,
    keyboard_list_state: ListState,

    // Summary
    confirm_wipe: bool,
    confirm_install: bool,

    // Focus management
    focus: FocusField,
    validation_error: Option<String>,

    // Scroll offset for log view
    log_scroll: u16,
}

impl Default for UiState {
    fn default() -> Self {
        let mut profile_list_state = ListState::default();
        profile_list_state.select(Some(0));

        let mut de_list_state = ListState::default();
        de_list_state.select(Some(0));

        let mut kernel_list_state = ListState::default();
        kernel_list_state.select(Some(1)); // Stable by default

        let mut init_list_state = ListState::default();
        init_list_state.select(Some(0)); // Systemd by default

        let mut bootloader_list_state = ListState::default();
        bootloader_list_state.select(Some(0)); // GRUB by default

        let mut layout_list_state = ListState::default();
        layout_list_state.select(Some(0));

        let mut fs_list_state = ListState::default();
        fs_list_state.select(Some(0));

        let mut encryption_list_state = ListState::default();
        encryption_list_state.select(Some(0));

        let mut tuning_list_state = ListState::default();
        tuning_list_state.select(Some(0));

        Self {
            hardware_info: HardwareInfo::default(),
            hardware_suggestions: Vec::new(),
            hardware_list_state: ListState::default(),

            profile_list_state,
            de_list_state,
            handheld_list_state: ListState::default(),
            audio_list_state: ListState::default(),
            selected_profile_category: 0,

            kernel_list_state,
            init_list_state,
            include_all_firmware: true,

            disk_list_state: ListState::default(),
            layout_list_state,
            fs_list_state,
            encryption_list_state,
            encryption_password: String::new(),
            encryption_password_confirm: String::new(),
            auto_partition: true,

            bootloader_list_state,

            tuning_list_state,
            system_limits: SystemLimitsConfig::default(),

            root_password: String::new(),
            root_password_confirm: String::new(),
            new_username: String::new(),
            new_fullname: String::new(),
            new_password: String::new(),
            new_password_confirm: String::new(),
            new_user_admin: true,
            users_list_state: ListState::default(),

            hostname: "buckos".to_string(),
            use_dhcp: true,

            timezones: Vec::new(),
            timezone_list_state: ListState::default(),
            locales: Vec::new(),
            locale_list_state: ListState::default(),
            keyboards: Vec::new(),
            keyboard_list_state: ListState::default(),

            confirm_wipe: false,
            confirm_install: false,

            focus: FocusField::List,
            validation_error: None,
            log_scroll: 0,
        }
    }
}

impl TuiApp {
    pub fn new(target: String, dry_run: bool, buckos_build_path: PathBuf) -> Self {
        let available_disks = system::get_available_disks().unwrap_or_default();
        let system_info = system::get_system_info();

        let mut config = InstallConfig::default();
        config.target_root = PathBuf::from(target);
        config.buckos_build_path = buckos_build_path;
        config.dry_run = dry_run;

        // Perform hardware detection
        let hardware_info = system::detect_hardware();
        let hardware_suggestions = system::generate_hardware_suggestions(&hardware_info);

        let mut ui = UiState::default();
        ui.hostname = config.network.hostname.clone();
        ui.timezones = system::get_timezones();
        ui.locales = system::get_locales();
        ui.keyboards = system::get_keyboard_layouts();
        ui.hardware_info = hardware_info.clone();
        ui.hardware_suggestions = hardware_suggestions;

        // Initialize system limits
        ui.system_limits =
            system::detect_system_limits(&hardware_info, &config.profile, &config.audio_subsystem);

        // Set initial selections
        if let Some(pos) = ui.timezones.iter().position(|tz| tz == "UTC") {
            ui.timezone_list_state.select(Some(pos));
        }
        if let Some(pos) = ui.locales.iter().position(|l| l == "en_US.UTF-8") {
            ui.locale_list_state.select(Some(pos));
        }
        ui.keyboard_list_state.select(Some(0));

        // Prefer removable disks
        if let Some(pos) = available_disks.iter().position(|d| d.removable) {
            ui.disk_list_state.select(Some(pos));
        } else if !available_disks.is_empty() {
            ui.disk_list_state.select(Some(0));
        }

        Self {
            current_step: InstallStep::Welcome,
            config,
            available_disks,
            system_info,
            progress: Arc::new(Mutex::new(InstallProgress::default())),
            installing: false,
            ui,
            should_quit: false,
        }
    }

    fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Global quit
        if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        if key == KeyCode::Char('q') && self.current_step != InstallStep::Installing {
            self.should_quit = true;
            return;
        }

        match self.current_step {
            InstallStep::Welcome => self.handle_welcome_input(key),
            InstallStep::HardwareDetection => self.handle_hardware_input(key),
            InstallStep::ProfileSelection => self.handle_profile_input(key),
            InstallStep::KernelSelection => self.handle_kernel_input(key),
            InstallStep::DiskSetup => self.handle_disk_input(key),
            InstallStep::Bootloader => self.handle_bootloader_input(key),
            InstallStep::SystemTuning => self.handle_tuning_input(key),
            InstallStep::UserSetup => self.handle_user_input(key),
            InstallStep::NetworkSetup => self.handle_network_input(key),
            InstallStep::Timezone => self.handle_timezone_input(key),
            InstallStep::Summary => self.handle_summary_input(key),
            InstallStep::Installing => self.handle_installing_input(key),
            InstallStep::Complete => self.handle_complete_input(key),
        }
    }

    fn navigate_next(&mut self) {
        self.ui.validation_error = None;

        if self.validate_current_step() {
            self.apply_current_step();

            if let Some(next) = self.current_step.next() {
                self.current_step = next;

                // Start installation if we're moving to Installing step
                if self.current_step == InstallStep::Installing {
                    self.start_installation();
                }
            }
        }
    }

    fn navigate_back(&mut self) {
        if let Some(prev) = self.current_step.prev() {
            self.current_step = prev;
            self.ui.validation_error = None;
        }
    }

    fn validate_current_step(&mut self) -> bool {
        match self.current_step {
            InstallStep::DiskSetup => {
                if self.ui.auto_partition && self.available_disks.is_empty() {
                    self.ui.validation_error =
                        Some("No disks available for installation".to_string());
                    return false;
                }

                let encryption_idx = self.ui.encryption_list_state.selected().unwrap_or(0);
                if encryption_idx != 0 {
                    // Encryption selected
                    if self.ui.encryption_password.is_empty() {
                        self.ui.validation_error =
                            Some("Encryption passphrase is required".to_string());
                        return false;
                    }
                    if self.ui.encryption_password != self.ui.encryption_password_confirm {
                        self.ui.validation_error =
                            Some("Encryption passphrases do not match".to_string());
                        return false;
                    }
                    if self.ui.encryption_password.len() < 8 {
                        self.ui.validation_error =
                            Some("Passphrase must be at least 8 characters".to_string());
                        return false;
                    }
                }
                true
            }
            InstallStep::UserSetup => {
                if self.ui.root_password.is_empty() {
                    self.ui.validation_error = Some("Root password is required".to_string());
                    return false;
                }
                if self.ui.root_password != self.ui.root_password_confirm {
                    self.ui.validation_error = Some("Root passwords do not match".to_string());
                    return false;
                }
                if self.ui.root_password.len() < 4 {
                    self.ui.validation_error =
                        Some("Root password must be at least 4 characters".to_string());
                    return false;
                }
                true
            }
            InstallStep::NetworkSetup => {
                if self.ui.hostname.is_empty() {
                    self.ui.validation_error = Some("Hostname is required".to_string());
                    return false;
                }
                true
            }
            InstallStep::Summary => {
                if !self.ui.confirm_install {
                    self.ui.validation_error = Some("Please confirm installation".to_string());
                    return false;
                }
                if self.config.disk.is_some() && !self.config.dry_run && !self.ui.confirm_wipe {
                    self.ui.validation_error = Some("Please confirm data destruction".to_string());
                    return false;
                }
                true
            }
            _ => true,
        }
    }

    fn apply_current_step(&mut self) {
        match self.current_step {
            InstallStep::HardwareDetection => {
                self.config.hardware_info = self.ui.hardware_info.clone();
                self.config.hardware_packages = self.ui.hardware_suggestions.clone();

                // Generate kernel config fragments
                let mut fragments = crate::kernel_config::generate_hardware_config_fragments(
                    &self.config.hardware_info,
                );

                let is_removable = self
                    .config
                    .disk
                    .as_ref()
                    .map(|d| d.removable)
                    .unwrap_or(false);
                fragments.push(crate::kernel_config::generate_boot_critical_config(
                    is_removable,
                ));

                let config_content = crate::kernel_config::fragments_to_config_file(&fragments);
                self.config.kernel_config_fragment = Some(config_content);
            }
            InstallStep::ProfileSelection => {
                let de_idx = self.ui.de_list_state.selected().unwrap_or(0);
                let des = DesktopEnvironment::all();
                let selected_de = des
                    .get(de_idx)
                    .cloned()
                    .unwrap_or(DesktopEnvironment::Gnome);

                let handheld_idx = self.ui.handheld_list_state.selected().unwrap_or(0);
                let handhelds = HandheldDevice::all();
                let selected_handheld = handhelds
                    .get(handheld_idx)
                    .cloned()
                    .unwrap_or(HandheldDevice::Generic);

                self.config.profile = match self.ui.selected_profile_category {
                    0 => InstallProfile::Desktop(selected_de),
                    1 => InstallProfile::Server,
                    2 => InstallProfile::Handheld(selected_handheld),
                    3 => InstallProfile::Minimal,
                    _ => InstallProfile::Custom,
                };

                let audio_idx = self.ui.audio_list_state.selected().unwrap_or(0);
                self.config.audio_subsystem = match audio_idx {
                    0 => AudioSubsystem::PipeWire,
                    1 => AudioSubsystem::PulseAudio,
                    _ => AudioSubsystem::Alsa,
                };
            }
            InstallStep::KernelSelection => {
                let kernel_idx = self.ui.kernel_list_state.selected().unwrap_or(1);
                self.config.kernel_channel = match kernel_idx {
                    0 => KernelChannel::LTS,
                    1 => KernelChannel::Stable,
                    _ => KernelChannel::Mainline,
                };

                let init_idx = self.ui.init_list_state.selected().unwrap_or(0);
                self.config.init_system = match init_idx {
                    0 => InitSystem::Systemd,
                    1 => InitSystem::OpenRC,
                    2 => InitSystem::Runit,
                    3 => InitSystem::S6,
                    4 => InitSystem::Dinit,
                    _ => InitSystem::BusyBoxInit,
                };

                self.config.include_all_firmware = self.ui.include_all_firmware;
            }
            InstallStep::DiskSetup => {
                let encryption_idx = self.ui.encryption_list_state.selected().unwrap_or(0);
                self.config.encryption.encryption_type = match encryption_idx {
                    0 => EncryptionType::None,
                    1 => EncryptionType::LuksRoot,
                    2 => EncryptionType::LuksFull,
                    _ => EncryptionType::LuksHome,
                };
                self.config.encryption.passphrase = self.ui.encryption_password.clone();

                let layout_idx = self.ui.layout_list_state.selected().unwrap_or(0);
                let layouts = DiskLayoutPreset::all();
                self.config.disk_layout = layouts
                    .get(layout_idx)
                    .cloned()
                    .unwrap_or(DiskLayoutPreset::Standard);

                if self.ui.auto_partition && !self.available_disks.is_empty() {
                    let disk_idx = self.ui.disk_list_state.selected().unwrap_or(0);
                    if let Some(disk) = self.available_disks.get(disk_idx) {
                        let fs_idx = self.ui.fs_list_state.selected().unwrap_or(0);
                        let fs = match fs_idx {
                            0 => FilesystemType::Ext4,
                            1 => FilesystemType::Btrfs,
                            2 => FilesystemType::Xfs,
                            _ => FilesystemType::F2fs,
                        };
                        self.config.disk = Some(disk::create_auto_partition_config(
                            disk,
                            &self.config.disk_layout,
                            fs,
                        ));
                    }
                }
            }
            InstallStep::Bootloader => {
                let idx = self.ui.bootloader_list_state.selected().unwrap_or(0);
                let bootloaders = if system::is_efi_system() {
                    BootloaderType::all()
                } else {
                    BootloaderType::all_for_bios()
                };
                self.config.bootloader = bootloaders
                    .get(idx)
                    .copied()
                    .unwrap_or(BootloaderType::Grub);
            }
            InstallStep::SystemTuning => {
                let idx = self.ui.tuning_list_state.selected().unwrap_or(0);
                let profiles = SystemTuningProfile::all();
                if let Some(profile) = profiles.get(idx) {
                    self.ui.system_limits.profile = *profile;
                }
                self.config.system_limits = self.ui.system_limits.clone();
            }
            InstallStep::UserSetup => {
                self.config.root_password = self.ui.root_password.clone();
            }
            InstallStep::NetworkSetup => {
                self.config.network.hostname = self.ui.hostname.clone();
                self.config.network.use_dhcp = self.ui.use_dhcp;
            }
            InstallStep::Timezone => {
                if let Some(idx) = self.ui.timezone_list_state.selected() {
                    if let Some(tz) = self.ui.timezones.get(idx) {
                        self.config.timezone.timezone = tz.clone();
                    }
                }
                if let Some(idx) = self.ui.locale_list_state.selected() {
                    if let Some(locale) = self.ui.locales.get(idx) {
                        self.config.locale.locale = locale.clone();
                    }
                }
                if let Some(idx) = self.ui.keyboard_list_state.selected() {
                    if let Some(kb) = self.ui.keyboards.get(idx) {
                        self.config.locale.keyboard = kb.clone();
                    }
                }
            }
            _ => {}
        }
    }

    fn start_installation(&mut self) {
        self.installing = true;
        let config = self.config.clone();
        let progress = Arc::clone(&self.progress);

        std::thread::spawn(move || {
            install::run_installation(config, progress);
        });
    }

    // Input handlers for each step
    fn handle_welcome_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            _ => {}
        }
    }

    fn handle_hardware_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left | KeyCode::Backspace => self.navigate_back(),
            KeyCode::Up => {
                let len = self.ui.hardware_suggestions.len();
                if len > 0 {
                    let i = self.ui.hardware_list_state.selected().unwrap_or(0);
                    let new_i = if i == 0 { len - 1 } else { i - 1 };
                    self.ui.hardware_list_state.select(Some(new_i));
                }
            }
            KeyCode::Down => {
                let len = self.ui.hardware_suggestions.len();
                if len > 0 {
                    let i = self.ui.hardware_list_state.selected().unwrap_or(0);
                    let new_i = (i + 1) % len;
                    self.ui.hardware_list_state.select(Some(new_i));
                }
            }
            KeyCode::Char(' ') => {
                if let Some(i) = self.ui.hardware_list_state.selected() {
                    if let Some(suggestion) = self.ui.hardware_suggestions.get_mut(i) {
                        suggestion.selected = !suggestion.selected;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_profile_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left | KeyCode::Backspace => self.navigate_back(),
            KeyCode::Tab => {
                // Cycle through profile categories
                self.ui.selected_profile_category = (self.ui.selected_profile_category + 1) % 5;
            }
            KeyCode::Up => {
                if self.ui.focus == FocusField::List {
                    let len = 5; // Number of profile categories
                    let i = self.ui.selected_profile_category;
                    self.ui.selected_profile_category = if i == 0 { len - 1 } else { i - 1 };
                }
            }
            KeyCode::Down => {
                if self.ui.focus == FocusField::List {
                    let len = 5;
                    self.ui.selected_profile_category =
                        (self.ui.selected_profile_category + 1) % len;
                }
            }
            _ => {}
        }
    }

    fn handle_kernel_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left | KeyCode::Backspace => self.navigate_back(),
            KeyCode::Tab => {
                self.ui.focus = match self.ui.focus {
                    FocusField::List => FocusField::List, // Keep on kernel list for simplicity
                    _ => FocusField::List,
                };
            }
            KeyCode::Up => {
                let i = self.ui.kernel_list_state.selected().unwrap_or(0);
                let new_i = if i == 0 { 2 } else { i - 1 };
                self.ui.kernel_list_state.select(Some(new_i));
            }
            KeyCode::Down => {
                let i = self.ui.kernel_list_state.selected().unwrap_or(0);
                let new_i = (i + 1) % 3;
                self.ui.kernel_list_state.select(Some(new_i));
            }
            KeyCode::Char(' ') => {
                self.ui.include_all_firmware = !self.ui.include_all_firmware;
            }
            _ => {}
        }
    }

    fn handle_disk_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left => self.navigate_back(),
            KeyCode::Tab => {
                self.ui.focus = match self.ui.focus {
                    FocusField::DiskList => FocusField::LayoutPreset,
                    FocusField::LayoutPreset => FocusField::Filesystem,
                    FocusField::Filesystem => FocusField::Encryption,
                    FocusField::Encryption => {
                        let encryption_idx = self.ui.encryption_list_state.selected().unwrap_or(0);
                        if encryption_idx != 0 {
                            FocusField::EncryptionPassword
                        } else {
                            FocusField::DiskList
                        }
                    }
                    FocusField::EncryptionPassword => FocusField::EncryptionConfirm,
                    FocusField::EncryptionConfirm => FocusField::DiskList,
                    _ => FocusField::DiskList,
                };
            }
            KeyCode::Up => match self.ui.focus {
                FocusField::DiskList => {
                    let len = self.available_disks.len();
                    if len > 0 {
                        let i = self.ui.disk_list_state.selected().unwrap_or(0);
                        let new_i = if i == 0 { len - 1 } else { i - 1 };
                        self.ui.disk_list_state.select(Some(new_i));
                    }
                }
                FocusField::LayoutPreset => {
                    let len = DiskLayoutPreset::all().len();
                    let i = self.ui.layout_list_state.selected().unwrap_or(0);
                    let new_i = if i == 0 { len - 1 } else { i - 1 };
                    self.ui.layout_list_state.select(Some(new_i));
                }
                FocusField::Filesystem => {
                    let len = 4; // Ext4, Btrfs, XFS, F2FS
                    let i = self.ui.fs_list_state.selected().unwrap_or(0);
                    let new_i = if i == 0 { len - 1 } else { i - 1 };
                    self.ui.fs_list_state.select(Some(new_i));
                }
                FocusField::Encryption => {
                    let len = EncryptionType::all().len();
                    let i = self.ui.encryption_list_state.selected().unwrap_or(0);
                    let new_i = if i == 0 { len - 1 } else { i - 1 };
                    self.ui.encryption_list_state.select(Some(new_i));
                }
                _ => {}
            },
            KeyCode::Down => match self.ui.focus {
                FocusField::DiskList => {
                    let len = self.available_disks.len();
                    if len > 0 {
                        let i = self.ui.disk_list_state.selected().unwrap_or(0);
                        let new_i = (i + 1) % len;
                        self.ui.disk_list_state.select(Some(new_i));
                    }
                }
                FocusField::LayoutPreset => {
                    let len = DiskLayoutPreset::all().len();
                    let i = self.ui.layout_list_state.selected().unwrap_or(0);
                    let new_i = (i + 1) % len;
                    self.ui.layout_list_state.select(Some(new_i));
                }
                FocusField::Filesystem => {
                    let len = 4;
                    let i = self.ui.fs_list_state.selected().unwrap_or(0);
                    let new_i = (i + 1) % len;
                    self.ui.fs_list_state.select(Some(new_i));
                }
                FocusField::Encryption => {
                    let len = EncryptionType::all().len();
                    let i = self.ui.encryption_list_state.selected().unwrap_or(0);
                    let new_i = (i + 1) % len;
                    self.ui.encryption_list_state.select(Some(new_i));
                }
                _ => {}
            },
            KeyCode::Char(c) => match self.ui.focus {
                FocusField::EncryptionPassword => {
                    self.ui.encryption_password.push(c);
                }
                FocusField::EncryptionConfirm => {
                    self.ui.encryption_password_confirm.push(c);
                }
                _ => {}
            },
            KeyCode::Backspace => match self.ui.focus {
                FocusField::EncryptionPassword => {
                    self.ui.encryption_password.pop();
                }
                FocusField::EncryptionConfirm => {
                    self.ui.encryption_password_confirm.pop();
                }
                _ => self.navigate_back(),
            },
            _ => {}
        }
    }

    fn handle_bootloader_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left | KeyCode::Backspace => self.navigate_back(),
            KeyCode::Up => {
                let bootloaders = if system::is_efi_system() {
                    BootloaderType::all()
                } else {
                    BootloaderType::all_for_bios()
                };
                let len = bootloaders.len();
                let i = self.ui.bootloader_list_state.selected().unwrap_or(0);
                let new_i = if i == 0 { len - 1 } else { i - 1 };
                self.ui.bootloader_list_state.select(Some(new_i));
            }
            KeyCode::Down => {
                let bootloaders = if system::is_efi_system() {
                    BootloaderType::all()
                } else {
                    BootloaderType::all_for_bios()
                };
                let len = bootloaders.len();
                let i = self.ui.bootloader_list_state.selected().unwrap_or(0);
                let new_i = (i + 1) % len;
                self.ui.bootloader_list_state.select(Some(new_i));
            }
            _ => {}
        }
    }

    fn handle_tuning_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left | KeyCode::Backspace => self.navigate_back(),
            KeyCode::Up => {
                let len = SystemTuningProfile::all().len();
                let i = self.ui.tuning_list_state.selected().unwrap_or(0);
                let new_i = if i == 0 { len - 1 } else { i - 1 };
                self.ui.tuning_list_state.select(Some(new_i));
            }
            KeyCode::Down => {
                let len = SystemTuningProfile::all().len();
                let i = self.ui.tuning_list_state.selected().unwrap_or(0);
                let new_i = (i + 1) % len;
                self.ui.tuning_list_state.select(Some(new_i));
            }
            _ => {}
        }
    }

    fn handle_user_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if self.ui.focus == FocusField::AddUserButton {
                    self.add_user();
                } else {
                    self.navigate_next();
                }
            }
            KeyCode::Right => {
                if !matches!(
                    self.ui.focus,
                    FocusField::RootPassword
                        | FocusField::RootPasswordConfirm
                        | FocusField::Username
                        | FocusField::FullName
                        | FocusField::UserPassword
                        | FocusField::UserPasswordConfirm
                ) {
                    self.navigate_next();
                }
            }
            KeyCode::Left => {
                if !matches!(
                    self.ui.focus,
                    FocusField::RootPassword
                        | FocusField::RootPasswordConfirm
                        | FocusField::Username
                        | FocusField::FullName
                        | FocusField::UserPassword
                        | FocusField::UserPasswordConfirm
                ) {
                    self.navigate_back();
                }
            }
            KeyCode::Tab => {
                self.ui.focus = match self.ui.focus {
                    FocusField::RootPassword => FocusField::RootPasswordConfirm,
                    FocusField::RootPasswordConfirm => FocusField::Username,
                    FocusField::Username => FocusField::FullName,
                    FocusField::FullName => FocusField::UserPassword,
                    FocusField::UserPassword => FocusField::UserPasswordConfirm,
                    FocusField::UserPasswordConfirm => FocusField::UserAdmin,
                    FocusField::UserAdmin => FocusField::AddUserButton,
                    FocusField::AddUserButton => FocusField::RootPassword,
                    _ => FocusField::RootPassword,
                };
            }
            KeyCode::Char(' ') => {
                if self.ui.focus == FocusField::UserAdmin {
                    self.ui.new_user_admin = !self.ui.new_user_admin;
                }
            }
            KeyCode::Char(c) => match self.ui.focus {
                FocusField::RootPassword => self.ui.root_password.push(c),
                FocusField::RootPasswordConfirm => self.ui.root_password_confirm.push(c),
                FocusField::Username => self.ui.new_username.push(c),
                FocusField::FullName => self.ui.new_fullname.push(c),
                FocusField::UserPassword => self.ui.new_password.push(c),
                FocusField::UserPasswordConfirm => self.ui.new_password_confirm.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.ui.focus {
                FocusField::RootPassword => {
                    self.ui.root_password.pop();
                }
                FocusField::RootPasswordConfirm => {
                    self.ui.root_password_confirm.pop();
                }
                FocusField::Username => {
                    self.ui.new_username.pop();
                }
                FocusField::FullName => {
                    self.ui.new_fullname.pop();
                }
                FocusField::UserPassword => {
                    self.ui.new_password.pop();
                }
                FocusField::UserPasswordConfirm => {
                    self.ui.new_password_confirm.pop();
                }
                _ => self.navigate_back(),
            },
            _ => {}
        }
    }

    fn add_user(&mut self) {
        if !self.ui.new_username.is_empty()
            && !self.ui.new_password.is_empty()
            && self.ui.new_password == self.ui.new_password_confirm
        {
            let user = UserConfig {
                username: self.ui.new_username.clone(),
                full_name: self.ui.new_fullname.clone(),
                password: self.ui.new_password.clone(),
                is_admin: self.ui.new_user_admin,
                shell: "/bin/bash".to_string(),
            };
            self.config.users.push(user);

            // Clear fields
            self.ui.new_username.clear();
            self.ui.new_fullname.clear();
            self.ui.new_password.clear();
            self.ui.new_password_confirm.clear();
            self.ui.new_user_admin = true;
        }
    }

    fn handle_network_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => {
                if self.ui.focus != FocusField::Hostname {
                    self.navigate_next();
                }
            }
            KeyCode::Left => {
                if self.ui.focus != FocusField::Hostname {
                    self.navigate_back();
                }
            }
            KeyCode::Tab => {
                self.ui.focus = match self.ui.focus {
                    FocusField::Hostname => FocusField::UseDhcp,
                    FocusField::UseDhcp => FocusField::Hostname,
                    _ => FocusField::Hostname,
                };
            }
            KeyCode::Char(' ') => {
                if self.ui.focus == FocusField::UseDhcp {
                    self.ui.use_dhcp = !self.ui.use_dhcp;
                }
            }
            KeyCode::Char(c) => {
                if self.ui.focus == FocusField::Hostname {
                    self.ui.hostname.push(c);
                }
            }
            KeyCode::Backspace => {
                if self.ui.focus == FocusField::Hostname {
                    self.ui.hostname.pop();
                } else {
                    self.navigate_back();
                }
            }
            _ => {}
        }
    }

    fn handle_timezone_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left | KeyCode::Backspace => self.navigate_back(),
            KeyCode::Tab => {
                self.ui.focus = match self.ui.focus {
                    FocusField::Timezone => FocusField::Locale,
                    FocusField::Locale => FocusField::Keyboard,
                    FocusField::Keyboard => FocusField::Timezone,
                    _ => FocusField::Timezone,
                };
            }
            KeyCode::Up => match self.ui.focus {
                FocusField::Timezone => {
                    let len = self.ui.timezones.len();
                    if len > 0 {
                        let i = self.ui.timezone_list_state.selected().unwrap_or(0);
                        let new_i = if i == 0 { len - 1 } else { i - 1 };
                        self.ui.timezone_list_state.select(Some(new_i));
                    }
                }
                FocusField::Locale => {
                    let len = self.ui.locales.len();
                    if len > 0 {
                        let i = self.ui.locale_list_state.selected().unwrap_or(0);
                        let new_i = if i == 0 { len - 1 } else { i - 1 };
                        self.ui.locale_list_state.select(Some(new_i));
                    }
                }
                FocusField::Keyboard => {
                    let len = self.ui.keyboards.len();
                    if len > 0 {
                        let i = self.ui.keyboard_list_state.selected().unwrap_or(0);
                        let new_i = if i == 0 { len - 1 } else { i - 1 };
                        self.ui.keyboard_list_state.select(Some(new_i));
                    }
                }
                _ => {}
            },
            KeyCode::Down => match self.ui.focus {
                FocusField::Timezone => {
                    let len = self.ui.timezones.len();
                    if len > 0 {
                        let i = self.ui.timezone_list_state.selected().unwrap_or(0);
                        let new_i = (i + 1) % len;
                        self.ui.timezone_list_state.select(Some(new_i));
                    }
                }
                FocusField::Locale => {
                    let len = self.ui.locales.len();
                    if len > 0 {
                        let i = self.ui.locale_list_state.selected().unwrap_or(0);
                        let new_i = (i + 1) % len;
                        self.ui.locale_list_state.select(Some(new_i));
                    }
                }
                FocusField::Keyboard => {
                    let len = self.ui.keyboards.len();
                    if len > 0 {
                        let i = self.ui.keyboard_list_state.selected().unwrap_or(0);
                        let new_i = (i + 1) % len;
                        self.ui.keyboard_list_state.select(Some(new_i));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_summary_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Right => self.navigate_next(),
            KeyCode::Left | KeyCode::Backspace => self.navigate_back(),
            KeyCode::Tab => {
                self.ui.focus = match self.ui.focus {
                    FocusField::ConfirmWipe => FocusField::ConfirmInstall,
                    FocusField::ConfirmInstall => FocusField::ConfirmWipe,
                    _ => FocusField::ConfirmWipe,
                };
            }
            KeyCode::Char(' ') => match self.ui.focus {
                FocusField::ConfirmWipe => {
                    self.ui.confirm_wipe = !self.ui.confirm_wipe;
                }
                FocusField::ConfirmInstall => {
                    self.ui.confirm_install = !self.ui.confirm_install;
                }
                _ => {}
            },
            KeyCode::Char('1') => {
                self.ui.confirm_wipe = !self.ui.confirm_wipe;
            }
            KeyCode::Char('2') => {
                self.ui.confirm_install = !self.ui.confirm_install;
            }
            _ => {}
        }
    }

    fn handle_installing_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if self.ui.log_scroll > 0 {
                    self.ui.log_scroll -= 1;
                }
            }
            KeyCode::Down => {
                self.ui.log_scroll += 1;
            }
            KeyCode::PageUp => {
                self.ui.log_scroll = self.ui.log_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.ui.log_scroll += 10;
            }
            _ => {}
        }
    }

    fn handle_complete_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    // Render methods
    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(2), // Progress
                Constraint::Min(10),   // Content
                Constraint::Length(1), // Help bar
            ])
            .split(frame.area());

        self.render_header(frame, chunks[0]);
        self.render_progress_bar(frame, chunks[1]);
        self.render_content(frame, chunks[2]);
        self.render_help_bar(frame, chunks[3]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = if self.config.dry_run {
            " BuckOS Installer [DRY RUN] "
        } else {
            " BuckOS Installer "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(block, area);
    }

    fn render_progress_bar(&self, frame: &mut Frame, area: Rect) {
        let progress =
            (self.current_step.index() as f64) / ((InstallStep::total_steps() - 1) as f64);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .percent((progress * 100.0) as u16)
            .label(format!(
                "Step {}/{}: {}",
                self.current_step.index() + 1,
                InstallStep::total_steps(),
                self.current_step.title()
            ));

        frame.render_widget(gauge, area);
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        // Create content area with border
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.current_step.title()))
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Show validation error if any
        if let Some(ref error) = self.ui.validation_error {
            let error_area = Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 1,
            };
            let error_text = Paragraph::new(format!("Error: {}", error))
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
            frame.render_widget(error_text, error_area);

            let content_area = Rect {
                x: inner.x,
                y: inner.y + 2,
                width: inner.width,
                height: inner.height.saturating_sub(2),
            };
            self.render_step_content(frame, content_area);
        } else {
            self.render_step_content(frame, inner);
        }
    }

    fn render_step_content(&mut self, frame: &mut Frame, area: Rect) {
        match self.current_step {
            InstallStep::Welcome => self.render_welcome(frame, area),
            InstallStep::HardwareDetection => self.render_hardware(frame, area),
            InstallStep::ProfileSelection => self.render_profile(frame, area),
            InstallStep::KernelSelection => self.render_kernel(frame, area),
            InstallStep::DiskSetup => self.render_disk(frame, area),
            InstallStep::Bootloader => self.render_bootloader(frame, area),
            InstallStep::SystemTuning => self.render_tuning(frame, area),
            InstallStep::UserSetup => self.render_user(frame, area),
            InstallStep::NetworkSetup => self.render_network(frame, area),
            InstallStep::Timezone => self.render_timezone(frame, area),
            InstallStep::Summary => self.render_summary(frame, area),
            InstallStep::Installing => self.render_installing(frame, area),
            InstallStep::Complete => self.render_complete(frame, area),
        }
    }

    fn render_welcome(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Welcome text
                Constraint::Min(8),    // System info
            ])
            .split(area);

        let welcome_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Welcome to the BuckOS Installer!",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("This installer will guide you through setting up BuckOS on your system."),
            Line::from(
                "Press Enter or Right Arrow to continue, Left Arrow or Backspace to go back.",
            ),
        ];

        let welcome = Paragraph::new(welcome_text).wrap(Wrap { trim: true });
        frame.render_widget(welcome, chunks[0]);

        // System info
        let info_items = vec![
            ("Memory", system::format_size(self.system_info.total_memory)),
            ("CPUs", format!("{} cores", self.system_info.cpu_count)),
            (
                "CPU",
                self.system_info
                    .cpu_brand
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
            ),
            (
                "Kernel",
                self.system_info
                    .kernel_version
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
            ),
            (
                "Boot Mode",
                if system::is_efi_system() {
                    "UEFI".to_string()
                } else {
                    "BIOS".to_string()
                },
            ),
        ];

        let info_widget = InfoBox::new("System Information", info_items);
        frame.render_widget(info_widget, chunks[1]);
    }

    fn render_hardware(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Hardware summary
                Constraint::Min(5),    // Package suggestions
            ])
            .split(area);

        // Hardware summary
        let hw = &self.ui.hardware_info;
        let gpu_info = hw
            .gpus
            .iter()
            .map(|g| g.name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let hw_items = vec![
            (
                "GPUs",
                if gpu_info.is_empty() {
                    "None detected".to_string()
                } else {
                    gpu_info
                },
            ),
            (
                "Network",
                format!("{} interfaces", hw.network_interfaces.len()),
            ),
            ("Audio", format!("{} devices", hw.audio_devices.len())),
            (
                "Bluetooth",
                if hw.has_bluetooth { "Yes" } else { "No" }.to_string(),
            ),
            (
                "Laptop",
                if hw.is_laptop { "Yes" } else { "No" }.to_string(),
            ),
            (
                "Virtual Machine",
                if hw.is_virtual_machine { "Yes" } else { "No" }.to_string(),
            ),
        ];

        let hw_widget = InfoBox::new("Detected Hardware", hw_items);
        frame.render_widget(hw_widget, chunks[0]);

        // Package suggestions
        let items: Vec<ListItem> = self
            .ui
            .hardware_suggestions
            .iter()
            .map(|s| {
                let checkbox = if s.selected { "[x]" } else { "[ ]" };
                let text = format!("{} {} - {}", checkbox, s.category, s.packages.join(", "));
                ListItem::new(Line::from(text))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Suggested Packages (Space to toggle)")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.ui.hardware_list_state);
    }

    fn render_profile(&mut self, frame: &mut Frame, area: Rect) {
        let categories = [
            ("Desktop", "Full desktop environment with GUI"),
            ("Server", "Headless server configuration"),
            ("Handheld", "Gaming handheld device"),
            ("Minimal", "Base system only"),
            ("Custom", "Select packages manually"),
        ];

        let items: Vec<ListItem> = categories
            .iter()
            .enumerate()
            .map(|(i, (name, desc))| {
                let marker = if i == self.ui.selected_profile_category {
                    "(*)"
                } else {
                    "( )"
                };
                let lines = vec![
                    Line::from(Span::styled(
                        format!("{} {}", marker, name),
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("    {}", desc),
                        Style::default().fg(Color::DarkGray),
                    )),
                ];
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Installation Profile")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(self.ui.selected_profile_category));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_kernel(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Kernel selection
                Constraint::Length(3),  // Firmware checkbox
                Constraint::Min(5),     // Init system
            ])
            .split(area);

        // Kernel selection
        let kernels = [
            ("LTS", "Long-term support - maximum stability"),
            ("Stable", "Latest stable - balanced (recommended)"),
            ("Mainline", "Bleeding edge - latest features"),
        ];

        let kernel_items: Vec<ListItem> = kernels
            .iter()
            .map(|(name, desc)| {
                let lines = vec![
                    Line::from(Span::styled(
                        *name,
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("  {}", desc),
                        Style::default().fg(Color::DarkGray),
                    )),
                ];
                ListItem::new(lines)
            })
            .collect();

        let kernel_list = List::new(kernel_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Kernel Channel")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(kernel_list, chunks[0], &mut self.ui.kernel_list_state);

        // Firmware checkbox
        let checkbox = Checkbox::new(
            "Include all firmware in initramfs (recommended for portability)",
            self.ui.include_all_firmware,
        );
        frame.render_widget(checkbox, chunks[1]);

        // Init system selection
        let init_systems = [
            "systemd (recommended)",
            "OpenRC",
            "runit",
            "s6",
            "dinit",
            "BusyBox init",
        ];

        let init_items: Vec<ListItem> = init_systems
            .iter()
            .map(|s| ListItem::new(Line::from(*s)))
            .collect();

        let init_list = List::new(init_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Init System")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(init_list, chunks[2], &mut self.ui.init_list_state);
    }

    fn render_disk(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Filesystem
                Constraint::Length(8), // Encryption
                Constraint::Min(5),    // Password fields
            ])
            .split(chunks[1]);

        // Disk list
        let disk_items: Vec<ListItem> = self
            .available_disks
            .iter()
            .map(|d| {
                let removable = if d.removable { " [USB]" } else { "" };
                let text = format!(
                    "{}{} - {} ({})",
                    d.device,
                    removable,
                    d.model,
                    system::format_size(d.size)
                );
                ListItem::new(text)
            })
            .collect();

        let disk_list = List::new(disk_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Target Disk")
                    .border_style(if self.ui.focus == FocusField::DiskList {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Cyan)
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(disk_list, left_chunks[0], &mut self.ui.disk_list_state);

        // Layout presets
        let layouts = DiskLayoutPreset::all();
        let layout_items: Vec<ListItem> = layouts
            .iter()
            .map(|l| ListItem::new(format!("{} - {}", l.name(), l.description())))
            .collect();

        let layout_list = List::new(layout_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Partition Layout")
                    .border_style(if self.ui.focus == FocusField::LayoutPreset {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Cyan)
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(layout_list, left_chunks[1], &mut self.ui.layout_list_state);

        // Filesystem selection
        let filesystems = ["ext4 (recommended)", "btrfs", "xfs", "f2fs"];
        let fs_items: Vec<ListItem> = filesystems.iter().map(|s| ListItem::new(*s)).collect();

        let fs_list = List::new(fs_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Filesystem")
                    .border_style(if self.ui.focus == FocusField::Filesystem {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Cyan)
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(fs_list, right_chunks[0], &mut self.ui.fs_list_state);

        // Encryption selection
        let encryption_types = EncryptionType::all();
        let enc_items: Vec<ListItem> = encryption_types
            .iter()
            .map(|e| ListItem::new(format!("{} - {}", e.name(), e.description())))
            .collect();

        let enc_list = List::new(enc_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Encryption")
                    .border_style(if self.ui.focus == FocusField::Encryption {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Cyan)
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(
            enc_list,
            right_chunks[1],
            &mut self.ui.encryption_list_state,
        );

        // Password fields (only if encryption selected)
        let encryption_idx = self.ui.encryption_list_state.selected().unwrap_or(0);
        if encryption_idx != 0 {
            let pw_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)])
                .split(right_chunks[2]);

            let pw_input = TextInput::new(&self.ui.encryption_password, "Passphrase")
                .focused(self.ui.focus == FocusField::EncryptionPassword)
                .password(true);
            frame.render_widget(pw_input, pw_chunks[0]);

            let pw_confirm =
                TextInput::new(&self.ui.encryption_password_confirm, "Confirm Passphrase")
                    .focused(self.ui.focus == FocusField::EncryptionConfirm)
                    .password(true);
            frame.render_widget(pw_confirm, pw_chunks[1]);
        }
    }

    fn render_bootloader(&mut self, frame: &mut Frame, area: Rect) {
        let bootloaders = if system::is_efi_system() {
            BootloaderType::all()
        } else {
            BootloaderType::all_for_bios()
        };

        let items: Vec<ListItem> = bootloaders
            .iter()
            .map(|b| {
                let lines = vec![
                    Line::from(Span::styled(
                        b.as_str(),
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("  {}", b.description()),
                        Style::default().fg(Color::DarkGray),
                    )),
                ];
                ListItem::new(lines)
            })
            .collect();

        let efi_status = if system::is_efi_system() {
            "UEFI mode detected"
        } else {
            "BIOS mode detected"
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Bootloader ({})", efi_status))
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.ui.bootloader_list_state);
    }

    fn render_tuning(&mut self, frame: &mut Frame, area: Rect) {
        let profiles = SystemTuningProfile::all();

        let items: Vec<ListItem> = profiles
            .iter()
            .map(|p| {
                let lines = vec![
                    Line::from(Span::styled(
                        p.name(),
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("  {}", p.description()),
                        Style::default().fg(Color::DarkGray),
                    )),
                ];
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("System Tuning Profile")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.ui.tuning_list_state);
    }

    fn render_user(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Root password section
                Constraint::Min(10),   // New user section
                Constraint::Length(5), // Users list
            ])
            .split(area);

        // Root password section
        let root_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(chunks[0]);

        let root_pw = TextInput::new(&self.ui.root_password, "Root Password")
            .focused(self.ui.focus == FocusField::RootPassword)
            .password(true);
        frame.render_widget(root_pw, root_chunks[0]);

        let root_pw_confirm =
            TextInput::new(&self.ui.root_password_confirm, "Confirm Root Password")
                .focused(self.ui.focus == FocusField::RootPasswordConfirm)
                .password(true);
        frame.render_widget(root_pw_confirm, root_chunks[1]);

        // New user section
        let user_block = Block::default()
            .borders(Borders::ALL)
            .title("Add User (optional)")
            .border_style(Style::default().fg(Color::Cyan));

        let user_inner = user_block.inner(chunks[1]);
        frame.render_widget(user_block, chunks[1]);

        let user_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Username
                Constraint::Length(3), // Full name
                Constraint::Length(3), // Password
                Constraint::Length(3), // Confirm password
                Constraint::Length(2), // Admin checkbox + Add button
            ])
            .split(user_inner);

        let username = TextInput::new(&self.ui.new_username, "Username")
            .focused(self.ui.focus == FocusField::Username);
        frame.render_widget(username, user_chunks[0]);

        let fullname = TextInput::new(&self.ui.new_fullname, "Full Name")
            .focused(self.ui.focus == FocusField::FullName);
        frame.render_widget(fullname, user_chunks[1]);

        let password = TextInput::new(&self.ui.new_password, "Password")
            .focused(self.ui.focus == FocusField::UserPassword)
            .password(true);
        frame.render_widget(password, user_chunks[2]);

        let password_confirm = TextInput::new(&self.ui.new_password_confirm, "Confirm Password")
            .focused(self.ui.focus == FocusField::UserPasswordConfirm)
            .password(true);
        frame.render_widget(password_confirm, user_chunks[3]);

        let admin_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(user_chunks[4]);

        let admin_checkbox = Checkbox::new("Administrator (wheel)", self.ui.new_user_admin)
            .focused(self.ui.focus == FocusField::UserAdmin);
        frame.render_widget(admin_checkbox, admin_area[0]);

        let add_button_style = if self.ui.focus == FocusField::AddUserButton {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let add_button = Paragraph::new("[Add User]").style(add_button_style);
        frame.render_widget(add_button, admin_area[1]);

        // Existing users list
        if !self.config.users.is_empty() {
            let user_items: Vec<ListItem> = self
                .config
                .users
                .iter()
                .map(|u| {
                    let admin = if u.is_admin { " (admin)" } else { "" };
                    ListItem::new(format!("{}{}", u.username, admin))
                })
                .collect();

            let users_list = List::new(user_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Added Users")
                    .border_style(Style::default().fg(Color::Cyan)),
            );

            frame.render_widget(users_list, chunks[2]);
        }
    }

    fn render_network(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Hostname
                Constraint::Length(2), // DHCP checkbox
                Constraint::Min(5),    // Info
            ])
            .split(area);

        let hostname = TextInput::new(&self.ui.hostname, "Hostname")
            .focused(self.ui.focus == FocusField::Hostname);
        frame.render_widget(hostname, chunks[0]);

        let dhcp_checkbox = Checkbox::new("Use DHCP for network configuration", self.ui.use_dhcp)
            .focused(self.ui.focus == FocusField::UseDhcp);
        frame.render_widget(dhcp_checkbox, chunks[1]);

        let info_text = vec![
            Line::from(""),
            Line::from("Network configuration will be applied after installation."),
            Line::from("You can modify /etc/network/interfaces or use NetworkManager later."),
        ];
        let info = Paragraph::new(info_text)
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true });
        frame.render_widget(info, chunks[2]);
    }

    fn render_timezone(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(area);

        // Timezone list
        let tz_items: Vec<ListItem> = self
            .ui
            .timezones
            .iter()
            .map(|tz| ListItem::new(tz.as_str()))
            .collect();

        let tz_list = List::new(tz_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Timezone")
                    .border_style(if self.ui.focus == FocusField::Timezone {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Cyan)
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(tz_list, chunks[0], &mut self.ui.timezone_list_state);

        // Locale list
        let locale_items: Vec<ListItem> = self
            .ui
            .locales
            .iter()
            .map(|l| ListItem::new(l.as_str()))
            .collect();

        let locale_list = List::new(locale_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Locale")
                    .border_style(if self.ui.focus == FocusField::Locale {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Cyan)
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(locale_list, chunks[1], &mut self.ui.locale_list_state);

        // Keyboard list
        let kb_items: Vec<ListItem> = self
            .ui
            .keyboards
            .iter()
            .map(|k| ListItem::new(k.as_str()))
            .collect();

        let kb_list = List::new(kb_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Keyboard")
                    .border_style(if self.ui.focus == FocusField::Keyboard {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Cyan)
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(kb_list, chunks[2], &mut self.ui.keyboard_list_state);
    }

    fn render_summary(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(15),   // Configuration summary
                Constraint::Length(5), // Confirmations
            ])
            .split(area);

        // Configuration summary
        let disk_info = self
            .config
            .disk
            .as_ref()
            .map(|d| format!("{} ({})", d.device, self.config.disk_layout.name()))
            .unwrap_or_else(|| "Manual configuration".to_string());

        let summary_items = vec![
            ("Profile", self.config.profile.category().to_string()),
            ("Kernel", self.config.kernel_channel.name().to_string()),
            ("Target Disk", disk_info),
            ("Bootloader", self.config.bootloader.as_str().to_string()),
            (
                "Encryption",
                self.config.encryption.encryption_type.name().to_string(),
            ),
            ("Root Password", "********".to_string()),
            ("Users", format!("{} user(s)", self.config.users.len())),
            ("Hostname", self.config.network.hostname.clone()),
            ("Timezone", self.config.timezone.timezone.clone()),
            ("Locale", self.config.locale.locale.clone()),
        ];

        let summary_widget = InfoBox::new("Installation Summary", summary_items);
        frame.render_widget(summary_widget, chunks[0]);

        // Confirmation checkboxes
        let confirm_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(2)])
            .split(chunks[1]);

        if self.config.disk.is_some() && !self.config.dry_run {
            let wipe_checkbox = Checkbox::new(
                "[1] I understand this will DESTROY ALL DATA on the selected disk",
                self.ui.confirm_wipe,
            )
            .focused(self.ui.focus == FocusField::ConfirmWipe);
            frame.render_widget(wipe_checkbox, confirm_chunks[0]);
        }

        let install_checkbox =
            Checkbox::new("[2] I am ready to install BuckOS", self.ui.confirm_install)
                .focused(self.ui.focus == FocusField::ConfirmInstall);
        frame.render_widget(install_checkbox, confirm_chunks[1]);
    }

    fn render_installing(&mut self, frame: &mut Frame, area: Rect) {
        let progress = self.progress.lock().unwrap();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Overall progress
                Constraint::Length(3), // Step progress
                Constraint::Length(2), // Current operation
                Constraint::Min(10),   // Log output
            ])
            .split(area);

        // Overall progress
        let overall_percent = (progress.overall_progress * 100.0) as u16;
        let overall_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Overall Progress"),
            )
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .percent(overall_percent)
            .label(format!("{}%", overall_percent));
        frame.render_widget(overall_gauge, chunks[0]);

        // Step progress
        let step_percent = (progress.step_progress * 100.0) as u16;
        let step_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Current Step"))
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .percent(step_percent);
        frame.render_widget(step_gauge, chunks[1]);

        // Current operation
        let operation = Paragraph::new(format!("  {}", progress.operation))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(operation, chunks[2]);

        // Log output
        let log_lines: Vec<Line> = progress
            .log
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect();

        let log = Paragraph::new(log_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Installation Log"),
            )
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false })
            .scroll((self.ui.log_scroll, 0));

        frame.render_widget(log, chunks[3]);

        // Auto-scroll to bottom
        let visible_lines = chunks[3].height.saturating_sub(2) as usize;
        if progress.log.len() > visible_lines {
            self.ui.log_scroll = (progress.log.len() - visible_lines) as u16;
        }

        // Check if installation is complete
        if progress.overall_progress >= 1.0 {
            self.current_step = InstallStep::Complete;
            self.installing = false;
        }
    }

    fn render_complete(&self, frame: &mut Frame, area: Rect) {
        let progress = self.progress.lock().unwrap();

        let has_errors = !progress.errors.is_empty();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Status message
                Constraint::Min(5),    // Instructions or errors
            ])
            .split(area);

        if has_errors {
            let error_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Installation completed with errors!",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Please review the errors below and try again."),
            ];

            let status = Paragraph::new(error_text).wrap(Wrap { trim: true });
            frame.render_widget(status, chunks[0]);

            let error_lines: Vec<Line> = progress
                .errors
                .iter()
                .map(|e| Line::from(Span::styled(e.as_str(), Style::default().fg(Color::Red))))
                .collect();

            let errors = Paragraph::new(error_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Errors")
                        .border_style(Style::default().fg(Color::Red)),
                )
                .wrap(Wrap { trim: true });

            frame.render_widget(errors, chunks[1]);
        } else {
            let success_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Installation Complete!",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("BuckOS has been successfully installed."),
                Line::from(""),
                Line::from("Next steps:"),
            ];

            let status = Paragraph::new(success_text).wrap(Wrap { trim: true });
            frame.render_widget(status, chunks[0]);

            let instructions = vec![
                Line::from("1. Remove the installation media"),
                Line::from("2. Reboot your system"),
                Line::from("3. Boot into your new BuckOS installation"),
                Line::from(""),
                Line::from(format!(
                    "Root directory: {}",
                    self.config.target_root.display()
                )),
                Line::from(""),
                Line::from("Press Enter or 'q' to exit."),
            ];

            let instructions_para = Paragraph::new(instructions)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Instructions")
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(Wrap { trim: true });

            frame.render_widget(instructions_para, chunks[1]);
        }
    }

    fn render_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help_items = match self.current_step {
            InstallStep::Welcome => vec![("Enter", "Next"), ("q", "Quit")],
            InstallStep::Installing => vec![("Up/Down", "Scroll"), ("PgUp/PgDn", "Page")],
            InstallStep::Complete => vec![("Enter/q", "Exit")],
            _ => vec![
                ("Tab", "Switch"),
                ("Enter", "Next"),
                ("Bksp", "Back"),
                ("Space", "Select"),
                ("q", "Quit"),
            ],
        };

        let help = HelpBar::new(help_items);
        frame.render_widget(help, area);
    }
}

/// Run the TUI installer
pub fn run_tui_installer(target: String, dry_run: bool, buckos_build_path: PathBuf) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = TuiApp::new(target, dry_run, buckos_build_path);

    // Initialize focus for first step
    app.ui.focus = FocusField::List;

    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut TuiApp) -> Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        // Poll for events with timeout for installation progress updates
        let timeout = if app.installing {
            Duration::from_millis(100)
        } else {
            Duration::from_millis(250)
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.handle_input(key.code, key.modifiers);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
