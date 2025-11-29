//! Installation wizard step UI components

use egui::{self, RichText, Ui};

use crate::system::{self, SystemInfo};
use crate::types::{
    AudioSubsystem, BootloaderType, DesktopEnvironment, DiskInfo, DiskLayoutPreset, EncryptionType,
    HandheldDevice, HardwareInfo, HardwarePackageSuggestion, InitSystem, InstallConfig,
    InstallProfile, InstallProgress, KernelChannel, NetworkInterfaceType, UserConfig,
};

/// Render the welcome step
pub fn render_welcome(ui: &mut Ui, system_info: &SystemInfo) {
    ui.label("Welcome to the BuckOS installer! This wizard will guide you through installing BuckOS on your system.");

    ui.add_space(16.0);

    ui.label(RichText::new("What is BuckOS?").strong());
    ui.label("Buckos is a source-based Linux distribution inspired by Gentoo, designed for users who want control over their system while maintaining ease of use.");

    ui.add_space(16.0);

    ui.label(RichText::new("System Information").strong());
    ui.indent("sysinfo", |ui| {
        egui::Grid::new("sysinfo_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("Memory:");
                ui.label(format!(
                    "{} / {}",
                    system::format_size(system_info.available_memory),
                    system::format_size(system_info.total_memory)
                ));
                ui.end_row();

                ui.label("CPU:");
                ui.label(format!(
                    "{} cores - {}",
                    system_info.cpu_count,
                    system_info.cpu_brand.as_deref().unwrap_or("Unknown")
                ));
                ui.end_row();

                if let Some(kernel) = &system_info.kernel_version {
                    ui.label("Kernel:");
                    ui.label(kernel);
                    ui.end_row();
                }

                ui.label("Boot mode:");
                ui.label(if system::is_efi_system() {
                    "UEFI"
                } else {
                    "BIOS/Legacy"
                });
                ui.end_row();
            });
    });

    ui.add_space(16.0);

    ui.label(RichText::new("Manual Installation").strong());
    ui.label("If you prefer to install manually (like Gentoo), you can run:");
    ui.add_space(4.0);
    ui.monospace("buckos-installer --text-mode");
    ui.add_space(4.0);
    ui.label("This will show you the commands to run manually.");

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Note:").strong());
        ui.label("This installer will make changes to your disk. Please back up important data.");
    });
}

/// Render the hardware detection step
pub fn render_hardware_detection(
    ui: &mut Ui,
    hardware: &HardwareInfo,
    _suggestions: &mut Vec<HardwarePackageSuggestion>,
) {
    ui.label("We've detected the following hardware in your system.");
    ui.add_space(4.0);
    ui.label(
        RichText::new("Hardware-specific drivers and packages will be automatically included based on your profile selection.")
            .small()
            .weak(),
    );

    ui.add_space(16.0);

    // GPU section
    if !hardware.gpus.is_empty() {
        ui.label(RichText::new("Graphics Cards").strong());
        ui.indent("gpus", |ui| {
            for gpu in &hardware.gpus {
                let vendor_name = match gpu.vendor {
                    crate::types::GpuVendor::Nvidia => "NVIDIA",
                    crate::types::GpuVendor::Amd => "AMD",
                    crate::types::GpuVendor::Intel => "Intel",
                    crate::types::GpuVendor::VirtualBox => "VirtualBox",
                    crate::types::GpuVendor::VMware => "VMware",
                    crate::types::GpuVendor::Unknown => "Unknown",
                };
                ui.label(format!("{}: {}", vendor_name, gpu.name));
            }
        });
        ui.add_space(8.0);
    }

    // Network interfaces
    if !hardware.network_interfaces.is_empty() {
        ui.label(RichText::new("Network Interfaces").strong());
        ui.indent("network", |ui| {
            for iface in &hardware.network_interfaces {
                let itype = match iface.interface_type {
                    NetworkInterfaceType::Ethernet => "Ethernet",
                    NetworkInterfaceType::Wifi => "WiFi",
                    NetworkInterfaceType::Bridge => "Bridge",
                    NetworkInterfaceType::Virtual => "Virtual",
                    NetworkInterfaceType::Unknown => "Unknown",
                };
                ui.label(format!("{} ({})", iface.name, itype));
            }
        });
        ui.add_space(8.0);
    }

    // System characteristics
    ui.label(RichText::new("System Type").strong());
    ui.indent("system_type", |ui| {
        if hardware.is_virtual_machine {
            ui.label("Running in virtual machine");
        } else if hardware.is_laptop {
            ui.label("Laptop/Portable device");
        } else {
            ui.label("Desktop system");
        }

        if hardware.has_bluetooth {
            ui.label("Bluetooth available");
        }
        if hardware.has_touchscreen {
            ui.label("Touchscreen detected");
        }
    });
}

/// Render the disk setup step
#[allow(clippy::too_many_arguments)]
pub fn render_disk_setup(
    ui: &mut Ui,
    disks: &[DiskInfo],
    selected_disk: &mut usize,
    auto_partition: &mut bool,
    layout_preset: &mut DiskLayoutPreset,
    root_filesystem: &mut crate::types::FilesystemType,
    encryption_type: &mut EncryptionType,
    encryption_passphrase: &mut String,
    confirm_passphrase: &mut String,
) {
    ui.label("Configure disk partitioning and encryption for your installation.");

    ui.add_space(16.0);

    if disks.is_empty() {
        ui.label(
            RichText::new("No disks detected!")
                .color(egui::Color32::RED)
                .strong(),
        );
        ui.label("Please ensure your disk is properly connected.");
        ui.add_space(8.0);
        ui.label(
            "For manual installation, you can partition the disk yourself and skip this step.",
        );
        *auto_partition = false;
        return;
    }

    // Disk selection
    ui.label(RichText::new("Select Disk:").strong());
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .max_height(120.0)
        .id_salt("disk_scroll")
        .show(ui, |ui| {
            for (i, disk) in disks.iter().enumerate() {
                let is_selected = *selected_disk == i;
                let response = ui.selectable_label(
                    is_selected,
                    format!(
                        "{} - {} ({}){}",
                        disk.device,
                        disk.model,
                        system::format_size(disk.size),
                        if disk.removable { " [Removable]" } else { "" }
                    ),
                );
                if response.clicked() {
                    *selected_disk = i;
                }

                // Show existing partitions
                if is_selected && !disk.partitions.is_empty() {
                    ui.indent("partitions", |ui| {
                        ui.label(RichText::new("Existing partitions:").small());
                        for part in &disk.partitions {
                            let fs = part.filesystem.as_deref().unwrap_or("unknown");
                            let mount = part
                                .mount_point
                                .as_deref()
                                .map(|m| format!(" on {}", m))
                                .unwrap_or_default();
                            ui.label(
                                RichText::new(format!(
                                    "  {} - {} ({}){}",
                                    part.device,
                                    system::format_size(part.size),
                                    fs,
                                    mount
                                ))
                                .small(),
                            );
                        }
                    });
                }
            }
        });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Partitioning options
    ui.checkbox(auto_partition, "Use automatic partitioning");

    if *auto_partition {
        ui.add_space(8.0);
        ui.label(RichText::new("Partition Layout:").strong());
        ui.add_space(4.0);

        for preset in DiskLayoutPreset::all() {
            if preset == DiskLayoutPreset::Custom {
                continue; // Skip custom in auto mode
            }
            let is_selected = layout_preset == &preset;
            let response = ui.selectable_label(is_selected, preset.name());
            if response.clicked() {
                *layout_preset = preset.clone();
            }
            ui.indent("layout_desc", |ui| {
                ui.label(RichText::new(preset.description()).small().weak());
            });
        }

        ui.add_space(16.0);
        ui.label(RichText::new("Root Filesystem:").strong());
        ui.add_space(4.0);

        // Filesystem options (only show filesystem types suitable for root)
        let filesystem_options = [
            (
                crate::types::FilesystemType::Ext4,
                "ext4",
                "Ext4 - Standard journaling filesystem (recommended)",
            ),
            (
                crate::types::FilesystemType::Btrfs,
                "btrfs",
                "Btrfs - Advanced copy-on-write filesystem with snapshots",
            ),
            (
                crate::types::FilesystemType::Xfs,
                "xfs",
                "XFS - High-performance journaling filesystem",
            ),
            (
                crate::types::FilesystemType::F2fs,
                "f2fs",
                "F2FS - Flash-Friendly File System (for SSDs)",
            ),
        ];

        for (fs_type, name, description) in filesystem_options {
            let is_selected = root_filesystem == &fs_type;
            let response = ui.selectable_label(is_selected, name);
            if response.clicked() {
                *root_filesystem = fs_type;
            }
            ui.indent("fs_desc", |ui| {
                ui.label(RichText::new(description).small().weak());
            });
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("Warning: This will erase all data on the selected disk!")
                .color(egui::Color32::RED),
        );
    } else {
        *layout_preset = DiskLayoutPreset::Custom;
        ui.indent("manual_part_info", |ui| {
            ui.label("You will need to partition the disk manually before proceeding.");
            ui.label("Mount your root partition to the target directory.");
        });
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Encryption options
    ui.label(RichText::new("Disk Encryption:").strong());
    ui.add_space(4.0);

    for enc in EncryptionType::all() {
        let is_selected = encryption_type == &enc;
        let response = ui.selectable_label(is_selected, enc.name());
        if response.clicked() {
            *encryption_type = enc.clone();
        }
        ui.indent("enc_desc", |ui| {
            ui.label(RichText::new(enc.description()).small().weak());
        });
    }

    // Encryption passphrase (if encryption selected)
    if *encryption_type != EncryptionType::None {
        ui.add_space(8.0);
        ui.label(RichText::new("Encryption Passphrase:").strong());
        ui.indent("enc_pass", |ui| {
            ui.horizontal(|ui| {
                ui.label("Passphrase:");
                ui.add(egui::TextEdit::singleline(encryption_passphrase).password(true));
            });
            ui.horizontal(|ui| {
                ui.label("Confirm:");
                ui.add(egui::TextEdit::singleline(confirm_passphrase).password(true));
            });

            if !encryption_passphrase.is_empty() && encryption_passphrase != confirm_passphrase {
                ui.label(
                    RichText::new("Passphrases do not match")
                        .color(egui::Color32::RED)
                        .small(),
                );
            }

            if !encryption_passphrase.is_empty() && encryption_passphrase.len() < 8 {
                ui.label(
                    RichText::new("Passphrase should be at least 8 characters")
                        .color(egui::Color32::YELLOW)
                        .small(),
                );
            }
        });
    }
}

/// Render the bootloader selection step
pub fn render_bootloader(ui: &mut Ui, bootloader: &mut BootloaderType, is_efi: bool) {
    ui.label("Select a bootloader to boot your system.");

    ui.add_space(16.0);

    // Show boot mode
    ui.horizontal(|ui| {
        ui.label(RichText::new("Boot Mode:").strong());
        ui.label(if is_efi { "UEFI" } else { "BIOS/Legacy" });
    });

    ui.add_space(8.0);

    if !is_efi {
        ui.label(
            RichText::new(
                "Note: Some bootloaders require UEFI and are not available in BIOS mode.",
            )
            .color(egui::Color32::YELLOW)
            .small(),
        );
        ui.add_space(8.0);
    }

    ui.label(RichText::new("Select Bootloader:").strong());
    ui.add_space(8.0);

    let available_bootloaders = if is_efi {
        BootloaderType::all()
    } else {
        BootloaderType::all_for_bios()
    };

    for bl in available_bootloaders {
        let is_selected = *bootloader == bl;
        let response = ui.selectable_label(is_selected, RichText::new(bl.as_str()).strong());
        if response.clicked() {
            *bootloader = bl;
        }

        ui.indent("bl_desc", |ui| {
            ui.label(RichText::new(bl.description()).small());

            if bl.requires_uefi() {
                ui.label(RichText::new("Requires UEFI").small().weak());
            }
        });
        ui.add_space(4.0);
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Bootloader-specific notes
    match bootloader {
        BootloaderType::Grub => {
            ui.label(RichText::new("GRUB Notes:").strong());
            ui.indent("grub_notes", |ui| {
                ui.label("• Most compatible option");
                ui.label("• Supports both BIOS and UEFI");
                ui.label("• Rich feature set (themes, encryption, etc.)");
                ui.label("• Can chainload other operating systems");
            });
        }
        BootloaderType::Systemdboot => {
            ui.label(RichText::new("systemd-boot Notes:").strong());
            ui.indent("sdb_notes", |ui| {
                ui.label("• Simple and minimal");
                ui.label("• Fast boot times");
                ui.label("• Easy to configure");
                ui.label("• Automatic kernel detection");
            });
        }
        BootloaderType::Refind => {
            ui.label(RichText::new("rEFInd Notes:").strong());
            ui.indent("refind_notes", |ui| {
                ui.label("• Graphical boot menu");
                ui.label("• Auto-detects operating systems");
                ui.label("• Highly customizable themes");
                ui.label("• Great for multi-boot setups");
            });
        }
        BootloaderType::Efistub => {
            ui.label(RichText::new("EFISTUB Notes:").strong());
            ui.indent("efistub_notes", |ui| {
                ui.label("• Boots kernel directly from UEFI");
                ui.label("• No bootloader overhead");
                ui.label("• Requires UEFI configuration");
                ui.label("• Advanced users only");
            });
        }
        BootloaderType::Limine => {
            ui.label(RichText::new("Limine Notes:").strong());
            ui.indent("limine_notes", |ui| {
                ui.label("• Modern bootloader");
                ui.label("• Supports both BIOS and UEFI");
                ui.label("• Multiboot and chainloading support");
                ui.label("• Active development");
            });
        }
        BootloaderType::None => {
            ui.label(RichText::new("Manual Bootloader:").strong());
            ui.indent("manual_notes", |ui| {
                ui.label("• No bootloader will be installed");
                ui.label("• You must configure a bootloader yourself");
                ui.label("• System will not boot without configuration");
            });
        }
    }
}

/// Render the profile selection step
#[allow(clippy::too_many_arguments)]
pub fn render_profile_selection(
    ui: &mut Ui,
    profile: &mut InstallProfile,
    selected_de: &mut DesktopEnvironment,
    selected_handheld: &mut HandheldDevice,
    audio_subsystem: &mut AudioSubsystem,
) {
    ui.label("Select an installation profile. This determines the default package set to install.");

    ui.add_space(16.0);

    // Profile category selection
    let categories = ["Desktop", "Server", "Handheld", "Minimal", "Custom"];
    let current_category = profile.category();

    ui.label(RichText::new("Profile Type").strong());
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        for cat in categories {
            let is_selected = current_category == cat;
            if ui.selectable_label(is_selected, cat).clicked() {
                *profile = match cat {
                    "Desktop" => InstallProfile::Desktop(selected_de.clone()),
                    "Server" => InstallProfile::Server,
                    "Handheld" => InstallProfile::Handheld(selected_handheld.clone()),
                    "Minimal" => InstallProfile::Minimal,
                    "Custom" => InstallProfile::Custom,
                    _ => profile.clone(),
                };
            }
        }
    });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Show options based on selected category
    match profile {
        InstallProfile::Desktop(_) => {
            ui.label(RichText::new("Desktop Environment").strong());
            ui.label(
                RichText::new("Choose your preferred desktop environment:")
                    .small()
                    .weak(),
            );
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for de in DesktopEnvironment::all() {
                        let is_selected = selected_de == &de;
                        let response =
                            ui.selectable_label(is_selected, RichText::new(de.name()).strong());
                        if response.clicked() {
                            *selected_de = de.clone();
                            *profile = InstallProfile::Desktop(de.clone());
                        }

                        ui.indent("de_desc", |ui| {
                            ui.label(RichText::new(de.description()).small());
                        });
                        ui.add_space(4.0);
                    }
                });
        }
        InstallProfile::Handheld(_) => {
            ui.label(RichText::new("Handheld Device").strong());
            ui.label(
                RichText::new("Select your gaming handheld device:")
                    .small()
                    .weak(),
            );
            ui.add_space(8.0);

            for device in HandheldDevice::all() {
                let is_selected = selected_handheld == &device;
                let response =
                    ui.selectable_label(is_selected, RichText::new(device.name()).strong());
                if response.clicked() {
                    *selected_handheld = device.clone();
                    *profile = InstallProfile::Handheld(device.clone());
                }

                ui.indent("handheld_desc", |ui| {
                    ui.label(RichText::new(device.description()).small());
                });
                ui.add_space(4.0);
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new("Includes: Steam, Gamescope, gaming optimizations")
                    .small()
                    .weak(),
            );
        }
        InstallProfile::Server => {
            ui.label(RichText::new("Server Profile").strong());
            ui.label("Minimal system with server tools and services.");
            ui.add_space(8.0);
            ui.label("Includes:");
            ui.indent("server_includes", |ui| {
                ui.label("• Core system utilities");
                ui.label("• SSH server");
                ui.label("• Network tools");
                ui.label("• System monitoring");
            });
        }
        InstallProfile::Minimal => {
            ui.label(RichText::new("Minimal Profile").strong());
            ui.label("Base system with only essential utilities.");
            ui.add_space(8.0);
            ui.label("Build your system from scratch - only @system package set installed.");
        }
        InstallProfile::Custom => {
            ui.label(RichText::new("Custom Profile").strong());
            ui.label("Select packages manually after installation.");
            ui.add_space(8.0);
            ui.label("Starts with @system, then customize as needed.");
        }
    }

    // Audio subsystem selection for desktop/handheld profiles
    if matches!(
        profile,
        InstallProfile::Desktop(_) | InstallProfile::Handheld(_)
    ) {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(RichText::new("Audio System").strong());
        ui.add_space(4.0);

        let audio_systems = [
            AudioSubsystem::PipeWire,
            AudioSubsystem::PulseAudio,
            AudioSubsystem::Alsa,
        ];

        for audio in audio_systems {
            let is_selected = *audio_subsystem == audio;
            let response = ui.selectable_label(is_selected, audio.name());
            if response.clicked() {
                *audio_subsystem = audio.clone();
            }
            let desc = audio.description();
            ui.indent("audio_desc", |ui| {
                ui.label(RichText::new(desc).small().weak());
            });
        }
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.label(RichText::new("Package Sets:").strong());
    ui.indent("packages", |ui| {
        for pkg_set in profile.package_sets() {
            ui.label(format!("• {}", pkg_set));
        }
    });
}

/// Render the kernel selection step
pub fn render_kernel_selection(
    ui: &mut Ui,
    kernel_channel: &mut KernelChannel,
    init_system: &mut InitSystem,
    include_all_firmware: &mut bool,
) {
    ui.label("Select the Linux kernel version and init system for your installation.");

    ui.add_space(16.0);

    // Kernel version selection
    ui.label(RichText::new("Linux Kernel").strong());
    ui.label(
        RichText::new("Choose the kernel version that best fits your needs:")
            .small()
            .weak(),
    );
    ui.add_space(8.0);

    for kernel in KernelChannel::all() {
        let is_selected = *kernel_channel == kernel;
        let response = ui.selectable_label(is_selected, RichText::new(kernel.name()).strong());
        if response.clicked() {
            *kernel_channel = kernel.clone();
        }
        ui.indent("kernel_desc", |ui| {
            ui.label(RichText::new(kernel.description()).small());
        });
        ui.add_space(4.0);
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Kernel-specific notes
    match kernel_channel {
        KernelChannel::LTS => {
            ui.label(RichText::new("LTS Kernel Notes:").strong());
            ui.indent("lts_notes", |ui| {
                ui.label("• Recommended for servers and production systems");
                ui.label("• Receives security updates for several years");
                ui.label("• Maximum stability and reliability");
                ui.label("• May lack support for newest hardware");
            });
        }
        KernelChannel::Stable => {
            ui.label(RichText::new("Stable Kernel Notes:").strong());
            ui.indent("stable_notes", |ui| {
                ui.label("• Good balance of features and stability");
                ui.label("• Recommended for most desktop users");
                ui.label("• Regular updates with new features");
                ui.label("• Better support for recent hardware");
            });
        }
        KernelChannel::Mainline => {
            ui.label(RichText::new("Mainline Kernel Notes:").strong());
            ui.indent("mainline_notes", |ui| {
                ui.label("• Cutting-edge features and drivers");
                ui.label("• Best for newest hardware support");
                ui.label("• May have occasional regressions");
                ui.label("• Recommended for developers and enthusiasts");
            });
        }
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Init system selection
    ui.label(RichText::new("Init System").strong());
    ui.label(
        RichText::new("Choose the init system and service manager:")
            .small()
            .weak(),
    );
    ui.add_space(8.0);

    for init in InitSystem::all() {
        let is_selected = *init_system == init;
        let response = ui.selectable_label(is_selected, RichText::new(init.name()).strong());
        if response.clicked() {
            *init_system = init.clone();
        }
        ui.indent("init_desc", |ui| {
            ui.label(RichText::new(init.description()).small());
        });
        ui.add_space(4.0);
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Init system-specific notes
    match init_system {
        InitSystem::Systemd => {
            ui.label(RichText::new("systemd Notes:").strong());
            ui.indent("systemd_notes", |ui| {
                ui.label("• Most widely used init system");
                ui.label("• Comprehensive service management");
                ui.label("• Best desktop integration");
                ui.label("• Required by some desktop environments");
            });
        }
        InitSystem::OpenRC => {
            ui.label(RichText::new("OpenRC Notes:").strong());
            ui.indent("openrc_notes", |ui| {
                ui.label("• Lightweight and fast boot times");
                ui.label("• Simple shell-based init scripts");
                ui.label("• Popular on Gentoo and Alpine");
                ui.label("• Good for servers and minimal systems");
            });
        }
        InitSystem::Runit => {
            ui.label(RichText::new("runit Notes:").strong());
            ui.indent("runit_notes", |ui| {
                ui.label("• Very simple and reliable");
                ui.label("• Process supervision built-in");
                ui.label("• Minimal resource usage");
                ui.label("• Used by Void Linux");
            });
        }
        _ => {
            ui.label(
                RichText::new("Advanced init system selected")
                    .small()
                    .weak(),
            );
        }
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Firmware inclusion option
    ui.label(RichText::new("Initramfs Options").strong());
    ui.add_space(8.0);

    ui.checkbox(include_all_firmware, "Include all firmware in initramfs");
    ui.indent("firmware_desc", |ui| {
        ui.label(
            RichText::new(
                "When enabled, includes all available firmware for maximum hardware compatibility. \
                 Recommended for portable installations (USB drives) that may boot on different machines.",
            )
            .small(),
        );
        ui.add_space(4.0);
        if *include_all_firmware {
            ui.label(
                RichText::new("✓ Larger initramfs, but works on any hardware")
                    .small()
                    .weak(),
            );
        } else {
            ui.label(
                RichText::new("✓ Smaller initramfs, optimized for this machine only")
                    .small()
                    .weak(),
            );
        }
    });
}

/// Render the user setup step
#[allow(clippy::too_many_arguments)]
pub fn render_user_setup(
    ui: &mut Ui,
    users: &mut Vec<UserConfig>,
    new_username: &mut String,
    new_fullname: &mut String,
    new_password: &mut String,
    confirm_password: &mut String,
    new_user_admin: &mut bool,
    root_password: &mut String,
    confirm_root_password: &mut String,
) {
    ui.label("Set up the root password and create user accounts.");

    ui.add_space(16.0);

    // Root password section
    ui.label(RichText::new("Root Password").strong());
    ui.indent("root_pw", |ui| {
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(root_password).password(true));
        });
        ui.horizontal(|ui| {
            ui.label("Confirm:");
            ui.add(egui::TextEdit::singleline(confirm_root_password).password(true));
        });

        if !root_password.is_empty() && root_password != confirm_root_password {
            ui.label(
                RichText::new("Passwords do not match")
                    .color(egui::Color32::RED)
                    .small(),
            );
        }
    });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // User accounts section
    ui.label(RichText::new("User Accounts").strong());

    // List existing users
    if !users.is_empty() {
        ui.indent("user_list", |ui| {
            let mut to_remove = None;
            for (i, user) in users.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} ({}){}",
                        user.username,
                        user.full_name,
                        if user.is_admin { " [admin]" } else { "" }
                    ));
                    if ui.small_button("Remove").clicked() {
                        to_remove = Some(i);
                    }
                });
            }
            if let Some(i) = to_remove {
                users.remove(i);
            }
        });
        ui.add_space(8.0);
    }

    // Add new user
    ui.collapsing("Add User", |ui| {
        egui::Grid::new("new_user_grid")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.label("Username:");
                ui.text_edit_singleline(new_username);
                ui.end_row();

                ui.label("Full Name:");
                ui.text_edit_singleline(new_fullname);
                ui.end_row();

                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(new_password).password(true));
                ui.end_row();

                ui.label("Confirm:");
                ui.add(egui::TextEdit::singleline(confirm_password).password(true));
                ui.end_row();

                ui.label("Administrator:");
                ui.checkbox(new_user_admin, "Add to wheel group");
                ui.end_row();
            });

        if !new_password.is_empty() && new_password != confirm_password {
            ui.label(
                RichText::new("Passwords do not match")
                    .color(egui::Color32::RED)
                    .small(),
            );
        }

        ui.add_space(8.0);

        let can_add = !new_username.is_empty()
            && !new_password.is_empty()
            && new_password == confirm_password;

        if ui
            .add_enabled(can_add, egui::Button::new("Add User"))
            .clicked()
        {
            users.push(UserConfig {
                username: new_username.clone(),
                full_name: new_fullname.clone(),
                password: new_password.clone(),
                is_admin: *new_user_admin,
                shell: "/bin/bash".to_string(),
            });

            // Clear fields
            new_username.clear();
            new_fullname.clear();
            new_password.clear();
            confirm_password.clear();
            *new_user_admin = true;
        }
    });

    ui.add_space(16.0);

    ui.label(
        RichText::new("Tip: Create at least one admin user for daily use instead of using root.")
            .small()
            .weak(),
    );
}

/// Render the network setup step
pub fn render_network_setup(ui: &mut Ui, hostname: &mut String, use_dhcp: &mut bool) {
    ui.label("Configure network settings for the installed system.");

    ui.add_space(16.0);

    ui.label(RichText::new("Hostname").strong());
    ui.indent("hostname", |ui| {
        ui.horizontal(|ui| {
            ui.label("Hostname:");
            ui.text_edit_singleline(hostname);
        });
        ui.label(
            RichText::new("The name used to identify this computer on the network")
                .small()
                .weak(),
        );
    });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.label(RichText::new("Network Configuration").strong());
    ui.indent("netconf", |ui| {
        ui.checkbox(use_dhcp, "Use DHCP for automatic configuration");

        if *use_dhcp {
            ui.label(
                RichText::new("Network will be configured automatically via DHCP")
                    .small()
                    .weak(),
            );
        } else {
            ui.add_space(8.0);
            ui.label("Manual network configuration can be done after installation.");
            ui.label(
                RichText::new("Edit /etc/conf.d/net or use NetworkManager")
                    .small()
                    .weak(),
            );
        }
    });
}

/// Render the timezone setup step
pub fn render_timezone_setup(
    ui: &mut Ui,
    timezones: &[String],
    selected_timezone: &mut usize,
    locales: &[String],
    selected_locale: &mut usize,
    keyboards: &[String],
    selected_keyboard: &mut usize,
) {
    ui.label("Configure timezone, locale, and keyboard layout.");

    ui.add_space(16.0);

    // Timezone
    ui.label(RichText::new("Timezone").strong());
    ui.indent("timezone", |ui| {
        egui::ComboBox::from_id_salt("timezone_combo")
            .selected_text(
                timezones
                    .get(*selected_timezone)
                    .map(|s| s.as_str())
                    .unwrap_or("Select timezone"),
            )
            .show_ui(ui, |ui| {
                for (i, tz) in timezones.iter().enumerate() {
                    ui.selectable_value(selected_timezone, i, tz);
                }
            });
    });

    ui.add_space(16.0);

    // Locale
    ui.label(RichText::new("System Locale").strong());
    ui.indent("locale", |ui| {
        egui::ComboBox::from_id_salt("locale_combo")
            .selected_text(
                locales
                    .get(*selected_locale)
                    .map(|s| s.as_str())
                    .unwrap_or("Select locale"),
            )
            .show_ui(ui, |ui| {
                for (i, locale) in locales.iter().enumerate() {
                    ui.selectable_value(selected_locale, i, locale);
                }
            });
    });

    ui.add_space(16.0);

    // Keyboard
    ui.label(RichText::new("Keyboard Layout").strong());
    ui.indent("keyboard", |ui| {
        egui::ComboBox::from_id_salt("keyboard_combo")
            .selected_text(
                keyboards
                    .get(*selected_keyboard)
                    .map(|s| s.as_str())
                    .unwrap_or("Select layout"),
            )
            .show_ui(ui, |ui| {
                for (i, kb) in keyboards.iter().enumerate() {
                    ui.selectable_value(selected_keyboard, i, kb);
                }
            });
    });
}

/// Render the summary step
pub fn render_summary(
    ui: &mut Ui,
    config: &InstallConfig,
    disks: &[DiskInfo],
    selected_disk: usize,
    confirm_wipe: &mut bool,
    confirm_install: &mut bool,
) {
    ui.label("Review your installation settings before proceeding.");

    ui.add_space(16.0);

    if config.dry_run {
        ui.label(
            RichText::new("DRY RUN MODE - No changes will be made")
                .color(egui::Color32::YELLOW)
                .strong(),
        );
        ui.add_space(8.0);
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("summary_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Target:").strong());
                ui.label(config.target_root.display().to_string());
                ui.end_row();

                // Profile details
                ui.label(RichText::new("Profile:").strong());
                let profile_str = match &config.profile {
                    InstallProfile::Desktop(de) => format!("Desktop ({})", de.name()),
                    InstallProfile::Handheld(device) => format!("Handheld ({})", device.name()),
                    InstallProfile::Server => "Server".to_string(),
                    InstallProfile::Minimal => "Minimal".to_string(),
                    InstallProfile::Custom => "Custom".to_string(),
                };
                ui.label(profile_str);
                ui.end_row();

                // Audio subsystem for desktop/handheld
                if matches!(
                    config.profile,
                    InstallProfile::Desktop(_) | InstallProfile::Handheld(_)
                ) {
                    ui.label(RichText::new("Audio:").strong());
                    ui.label(config.audio_subsystem.name());
                    ui.end_row();
                }

                // Disk configuration
                if let Some(disk_config) = &config.disk {
                    ui.label(RichText::new("Disk:").strong());
                    if let Some(disk) = disks.get(selected_disk) {
                        ui.label(format!(
                            "{} ({})",
                            disk.device,
                            system::format_size(disk.size)
                        ));
                    } else {
                        ui.label(&disk_config.device);
                    }
                    ui.end_row();

                    ui.label(RichText::new("Layout:").strong());
                    ui.label(config.disk_layout.name());
                    ui.end_row();

                    ui.label(RichText::new("Partitions:").strong());
                    ui.label(format!("{} partitions", disk_config.partitions.len()));
                    ui.end_row();
                }

                // Encryption
                ui.label(RichText::new("Encryption:").strong());
                ui.label(config.encryption.encryption_type.name());
                ui.end_row();

                // Bootloader
                ui.label(RichText::new("Bootloader:").strong());
                ui.label(config.bootloader.as_str());
                ui.end_row();

                ui.label(RichText::new("Hostname:").strong());
                ui.label(&config.network.hostname);
                ui.end_row();

                ui.label(RichText::new("Timezone:").strong());
                ui.label(&config.timezone.timezone);
                ui.end_row();

                ui.label(RichText::new("Locale:").strong());
                ui.label(&config.locale.locale);
                ui.end_row();

                ui.label(RichText::new("Keyboard:").strong());
                ui.label(&config.locale.keyboard);
                ui.end_row();

                ui.label(RichText::new("Users:").strong());
                if config.users.is_empty() {
                    ui.label("None (root only)");
                } else {
                    ui.label(
                        config
                            .users
                            .iter()
                            .map(|u| u.username.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                }
                ui.end_row();
            });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(RichText::new("Package Sets to Install:").strong());
        ui.indent("packages", |ui| {
            for pkg_set in config.profile.package_sets() {
                ui.label(format!("• {}", pkg_set));
            }
            // Add audio subsystem package set
            if matches!(
                config.profile,
                InstallProfile::Desktop(_) | InstallProfile::Handheld(_)
            ) {
                ui.label(format!("• {}", config.audio_subsystem.package_set()));
            }
        });

        ui.add_space(4.0);
        ui.label(
            RichText::new("Hardware-specific drivers will be automatically included")
                .small()
                .weak(),
        );

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Warning section for destructive operations
        if config.disk.is_some() && !config.dry_run {
            ui.label(
                RichText::new("WARNING: Destructive Operations")
                    .color(egui::Color32::RED)
                    .strong()
                    .heading(),
            );
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(50, 20, 20))
                .inner_margin(12.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("The following operations will PERMANENTLY DESTROY DATA:")
                            .color(egui::Color32::from_rgb(255, 150, 150)),
                    );
                    ui.add_space(4.0);

                    if let Some(disk) = disks.get(selected_disk) {
                        ui.label(
                            RichText::new(format!("• Disk {} will be WIPED", disk.device))
                                .color(egui::Color32::from_rgb(255, 200, 200)),
                        );
                    }

                    if config.encryption.encryption_type != crate::types::EncryptionType::None {
                        ui.label(
                            RichText::new(
                                "• Encryption will be applied (requires passphrase on every boot)",
                            )
                            .color(egui::Color32::from_rgb(255, 200, 200)),
                        );
                    }

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Make sure you have backed up any important data!")
                            .color(egui::Color32::YELLOW)
                            .strong(),
                    );
                });

            ui.add_space(12.0);

            ui.checkbox(
                confirm_wipe,
                RichText::new("I understand that all data on the selected disk will be destroyed")
                    .strong(),
            );
        }

        ui.add_space(8.0);

        ui.checkbox(
            confirm_install,
            RichText::new("I have reviewed the settings and want to proceed with installation")
                .strong(),
        );

        ui.add_space(16.0);

        let can_install = if config.disk.is_some() && !config.dry_run {
            *confirm_wipe && *confirm_install
        } else {
            *confirm_install
        };

        if !can_install {
            ui.label(
                RichText::new("Please check the confirmation boxes above to enable installation")
                    .small()
                    .weak(),
            );
        } else {
            ui.label(RichText::new("Click 'Install' to begin the installation process.").strong());
        }
    });
}

/// Render the installing step
pub fn render_installing(ui: &mut Ui, progress: &InstallProgress) {
    ui.label("Installing BuckOS to your system...");

    ui.add_space(16.0);

    // Current operation
    ui.label(RichText::new(&progress.operation).strong());

    ui.add_space(8.0);

    // Overall progress
    ui.horizontal(|ui| {
        ui.label("Overall:");
        ui.add(
            egui::ProgressBar::new(progress.overall_progress)
                .show_percentage()
                .animate(true),
        );
    });

    // Step progress
    ui.horizontal(|ui| {
        ui.label("Current:");
        ui.add(egui::ProgressBar::new(progress.step_progress).animate(true));
    });

    ui.add_space(16.0);

    // Log output
    ui.label(RichText::new("Log:").strong());
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for msg in &progress.log {
                ui.label(RichText::new(msg).monospace().small());
            }
        });

    // Show errors if any
    if !progress.errors.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new("Errors:").strong().color(egui::Color32::RED));
        for err in &progress.errors {
            ui.label(RichText::new(err).color(egui::Color32::RED).small());
        }
    }
}

/// Render the complete step
pub fn render_complete(ui: &mut Ui, config: &InstallConfig) {
    if config.dry_run {
        ui.label(
            RichText::new("Dry run completed!")
                .heading()
                .color(egui::Color32::GREEN),
        );
        ui.add_space(8.0);
        ui.label("No changes were made to your system.");
    } else {
        ui.label(
            RichText::new("Installation Complete!")
                .heading()
                .color(egui::Color32::GREEN),
        );
    }

    ui.add_space(16.0);

    ui.label("BuckOS has been successfully installed to your system.");

    ui.add_space(16.0);

    ui.label(RichText::new("Next Steps:").strong());
    ui.indent("next_steps", |ui| {
        ui.label("1. Remove the installation media");
        ui.label("2. Reboot your computer");
        ui.label("3. Log in with your user account");
        ui.label("4. Run 'buckos sync' to update package information");
        ui.label("5. Run 'buckos update @world' to update all packages");
    });

    ui.add_space(16.0);

    ui.label(RichText::new("Useful Commands:").strong());
    ui.indent("commands", |ui| {
        ui.monospace("buckos search <package>  # Search for packages");
        ui.monospace("buckos install <package> # Install a package");
        ui.monospace("buckos info <package>    # Show package info");
        ui.monospace("buckos --help            # Show all commands");
    });

    ui.add_space(16.0);

    ui.label(RichText::new("Thank you for choosing BuckOS!").strong());
}
