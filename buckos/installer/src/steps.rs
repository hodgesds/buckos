//! Installation wizard step UI components

use egui::{self, RichText, Ui};

use crate::system::{self, SystemInfo};
use crate::types::{DiskInfo, InstallConfig, InstallProfile, InstallProgress, UserConfig};

/// Render the welcome step
pub fn render_welcome(ui: &mut Ui, system_info: &SystemInfo) {
    ui.label("Welcome to the Buckos installer! This wizard will guide you through installing Buckos on your system.");

    ui.add_space(16.0);

    ui.label(RichText::new("What is Buckos?").strong());
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
                    system_info
                        .cpu_brand
                        .as_deref()
                        .unwrap_or("Unknown")
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

/// Render the disk setup step
pub fn render_disk_setup(
    ui: &mut Ui,
    disks: &[DiskInfo],
    selected_disk: &mut usize,
    auto_partition: &mut bool,
) {
    ui.label("Select the disk to install Buckos on. You can use automatic partitioning or set up partitions manually.");

    ui.add_space(16.0);

    if disks.is_empty() {
        ui.label(
            RichText::new("No disks detected!")
                .color(egui::Color32::RED)
                .strong(),
        );
        ui.label("Please ensure your disk is properly connected.");
        ui.add_space(8.0);
        ui.label("For manual installation, you can partition the disk yourself and skip this step.");
        *auto_partition = false;
        return;
    }

    ui.label(RichText::new("Available Disks:").strong());
    ui.add_space(8.0);

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

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.checkbox(auto_partition, "Use automatic partitioning");

    if *auto_partition {
        ui.indent("auto_part_info", |ui| {
            ui.label("Automatic partitioning will create:");
            ui.label(if system::is_efi_system() {
                "  • EFI System Partition (512 MB, FAT32)"
            } else {
                "  • BIOS Boot Partition (1 MB)"
            });
            ui.label("  • Swap Partition (based on RAM size)");
            ui.label("  • Root Partition (remaining space, ext4)");

            ui.add_space(8.0);
            ui.label(
                RichText::new("Warning: This will erase all data on the selected disk!")
                    .color(egui::Color32::RED),
            );
        });
    } else {
        ui.indent("manual_part_info", |ui| {
            ui.label("You will need to partition the disk manually before proceeding.");
            ui.label("Mount your root partition to the target directory.");
        });
    }
}

/// Render the profile selection step
pub fn render_profile_selection(ui: &mut Ui, profile: &mut InstallProfile) {
    ui.label("Select an installation profile. This determines the default package set to install.");

    ui.add_space(16.0);

    let profiles = [
        InstallProfile::Desktop,
        InstallProfile::Minimal,
        InstallProfile::Server,
        InstallProfile::Custom,
    ];

    for p in profiles {
        let is_selected = *profile == p;
        let response = ui.selectable_label(
            is_selected,
            RichText::new(format!("{:?}", p)).strong(),
        );
        if response.clicked() {
            *profile = p.clone();
        }

        ui.indent("profile_desc", |ui| {
            ui.label(p.description());
            ui.label(
                RichText::new(format!("Packages: {}", p.package_sets().join(", ")))
                    .small()
                    .weak(),
            );
        });
        ui.add_space(8.0);
    }

    ui.add_space(16.0);

    ui.label(RichText::new("Package Sets Explained:").strong());
    ui.indent("package_sets", |ui| {
        ui.label("• @system - Core system utilities and libraries");
        ui.label("• @desktop - Desktop environment and common applications");
        ui.label("• @server - Server services and management tools");
        ui.label("• @audio - Audio subsystem and utilities");
        ui.label("• @network - Network tools and services");
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

        if ui.add_enabled(can_add, egui::Button::new("Add User")).clicked() {
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
pub fn render_summary(ui: &mut Ui, config: &InstallConfig, disks: &[DiskInfo], selected_disk: usize) {
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

    egui::Grid::new("summary_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Target:").strong());
            ui.label(config.target_root.display().to_string());
            ui.end_row();

            ui.label(RichText::new("Profile:").strong());
            ui.label(format!("{:?}", config.profile));
            ui.end_row();

            if let Some(disk_config) = &config.disk {
                ui.label(RichText::new("Disk:").strong());
                if let Some(disk) = disks.get(selected_disk) {
                    ui.label(format!("{} ({})", disk.device, system::format_size(disk.size)));
                } else {
                    ui.label(&disk_config.device);
                }
                ui.end_row();

                ui.label(RichText::new("Partitions:").strong());
                ui.label(format!("{} partitions", disk_config.partitions.len()));
                ui.end_row();
            }

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
    });

    ui.add_space(16.0);

    ui.label(
        RichText::new("Click 'Install' to begin the installation process.")
            .strong(),
    );
}

/// Render the installing step
pub fn render_installing(ui: &mut Ui, progress: &InstallProgress) {
    ui.label("Installing Buckos to your system...");

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
        ui.label(
            RichText::new("Errors:")
                .strong()
                .color(egui::Color32::RED),
        );
        for err in &progress.errors {
            ui.label(
                RichText::new(err)
                    .color(egui::Color32::RED)
                    .small(),
            );
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

    ui.label("Buckos has been successfully installed to your system.");

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

    ui.label(
        RichText::new("Thank you for choosing Buckos!")
            .strong(),
    );
}
