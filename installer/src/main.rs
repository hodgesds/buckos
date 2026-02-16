//! BuckOS Installer - Graphical system installation tool
//!
//! This installer provides a beginner-friendly GUI for installing BuckOS
//! while maintaining the flexibility for manual installation similar to Gentoo.

mod app;
mod disk;
mod install;
mod kernel_config;
mod steps;
mod system;
mod tui;
mod types;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// BuckOS Installer - Install BuckOS to your system
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in text-only mode (no GUI)
    #[arg(long)]
    text_mode: bool,

    /// Target root directory for installation
    #[arg(long, default_value = "/mnt/buckos")]
    target: String,

    /// Path to buckos-build repository (auto-detected if not specified)
    #[arg(long)]
    buckos_build_path: Option<String>,

    /// Skip system requirements check
    #[arg(long)]
    skip_checks: bool,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Perform a dry run without making changes
    #[arg(long)]
    dry_run: bool,
}

/// Check that we have the necessary environment variables to connect to a display server.
/// This is especially important when running with sudo.
fn check_display_environment() -> Result<()> {
    use std::env;

    // Check if we're running as root
    let is_root = unsafe { libc::geteuid() } == 0;

    if !is_root {
        // Not running as root, environment should be fine
        return Ok(());
    }

    // Running as root - check for necessary environment variables
    let has_wayland = env::var("WAYLAND_DISPLAY").is_ok();
    let has_xdg_runtime = env::var("XDG_RUNTIME_DIR").is_ok();
    let has_display = env::var("DISPLAY").is_ok();

    // If we have neither Wayland nor X11 environment variables, we'll likely fail
    if !has_wayland && !has_display {
        eprintln!("\n╔════════════════════════════════════════════════════════════════════╗");
        eprintln!("║              ERROR: Display Server Connection Missing              ║");
        eprintln!("╚════════════════════════════════════════════════════════════════════╝\n");
        eprintln!("The installer is running as root but cannot connect to your display");
        eprintln!("server. This happens when environment variables are not preserved.\n");

        if !has_wayland {
            eprintln!("Missing: WAYLAND_DISPLAY environment variable");
        }
        if !has_xdg_runtime {
            eprintln!("Missing: XDG_RUNTIME_DIR environment variable");
        }
        if !has_display {
            eprintln!("Missing: DISPLAY environment variable");
        }

        eprintln!("\n📋 SOLUTIONS:\n");
        eprintln!("  1. Run with preserved environment variables:");
        eprintln!("     $ sudo -E ./target/release/buckos-installer\n");

        eprintln!("  2. For Wayland (recommended), explicitly preserve variables:");
        eprintln!("     $ sudo WAYLAND_DISPLAY=\"$WAYLAND_DISPLAY\" \\");
        eprintln!("            XDG_RUNTIME_DIR=\"$XDG_RUNTIME_DIR\" \\");
        eprintln!("            ./target/release/buckos-installer\n");

        eprintln!("  3. Use the text-mode installer (no GUI):");
        eprintln!("     $ sudo ./target/release/buckos-installer --text-mode\n");

        eprintln!("  4. Run without sudo and use polkit/pkexec for privilege escalation");
        eprintln!("     when needed (GUI will prompt for password):\n");
        eprintln!("     $ ./target/release/buckos-installer\n");

        return Err(anyhow::anyhow!(
            "Cannot connect to display server. Please use one of the solutions above."
        ));
    }

    // Warn if we're missing Wayland-specific variables even though WAYLAND_DISPLAY is set
    if has_wayland && !has_xdg_runtime {
        tracing::warn!(
            "WAYLAND_DISPLAY is set but XDG_RUNTIME_DIR is missing. This may cause issues."
        );
        eprintln!("\n⚠️  WARNING: XDG_RUNTIME_DIR is not set.");
        eprintln!("    The installer may have trouble connecting to Wayland.\n");
        eprintln!("    Consider running with:");
        eprintln!("    $ sudo -E ./target/release/buckos-installer\n");
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.debug {
        "buckos_installer=debug,info"
    } else {
        "buckos_installer=info,warn"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("BuckOS Installer starting...");

    // Check for proper environment when running with sudo (GUI mode only)
    if !args.text_mode {
        check_display_environment()?;
    }

    // Detect or validate buckos-build path
    let buckos_build_path = system::detect_buckos_build_path(args.buckos_build_path.as_deref())?;
    tracing::info!("Using buckos-build at: {}", buckos_build_path.display());

    // Check system requirements
    if !args.skip_checks {
        if let Err(e) = system::check_requirements() {
            tracing::error!("System requirements not met: {}", e);
            eprintln!("\nSystem requirements check failed:");
            eprintln!("  {}\n", e);
            eprintln!("You can skip this check with --skip-checks, but installation may fail.");
            eprintln!("For manual installation, please ensure the required tools are available.");
            std::process::exit(1);
        }
    }

    if args.text_mode {
        // Run text-based installer
        run_text_installer(&args, buckos_build_path)
    } else {
        // Run graphical installer
        run_gui_installer(&args, buckos_build_path)
    }
}

fn run_gui_installer(args: &Args, buckos_build_path: std::path::PathBuf) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("BuckOS Installer"),
        ..Default::default()
    };

    let target = args.target.clone();
    let dry_run = args.dry_run;

    eframe::run_native(
        "BuckOS Installer",
        options,
        Box::new(move |cc| {
            // Setup custom fonts and styles
            setup_custom_styles(&cc.egui_ctx);
            Ok(Box::new(app::InstallerApp::new(
                cc,
                target,
                dry_run,
                buckos_build_path,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

fn run_text_installer(args: &Args, buckos_build_path: std::path::PathBuf) -> Result<()> {
    tui::run_tui_installer(args.target.clone(), args.dry_run, buckos_build_path)
}

fn setup_custom_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Use slightly larger text for readability
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(22.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );

    // Improve spacing
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);

    ctx.set_style(style);
}
