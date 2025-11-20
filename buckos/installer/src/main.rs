//! Buckos Installer - Graphical system installation tool
//!
//! This installer provides a beginner-friendly GUI for installing Buckos
//! while maintaining the flexibility for manual installation similar to Gentoo.

mod app;
mod steps;
mod system;
mod types;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Buckos Installer - Install Buckos to your system
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in text-only mode (no GUI)
    #[arg(long)]
    text_mode: bool,

    /// Target root directory for installation
    #[arg(long, default_value = "/mnt/buckos")]
    target: String,

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

    tracing::info!("Buckos Installer starting...");

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
        run_text_installer(&args)
    } else {
        // Run graphical installer
        run_gui_installer(&args)
    }
}

fn run_gui_installer(args: &Args) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Buckos Installer"),
        ..Default::default()
    };

    let target = args.target.clone();
    let dry_run = args.dry_run;

    eframe::run_native(
        "Buckos Installer",
        options,
        Box::new(move |cc| {
            // Setup custom fonts and styles
            setup_custom_styles(&cc.egui_ctx);
            Ok(Box::new(app::InstallerApp::new(cc, target, dry_run)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

fn run_text_installer(args: &Args) -> Result<()> {
    use console::style;

    println!(
        "\n{}",
        style("═══════════════════════════════════════").cyan()
    );
    println!(
        "{}",
        style("       Buckos Text-Mode Installer       ")
            .cyan()
            .bold()
    );
    println!(
        "{}",
        style("═══════════════════════════════════════").cyan()
    );
    println!();
    println!(
        "Target installation directory: {}",
        style(&args.target).yellow()
    );
    if args.dry_run {
        println!(
            "{}",
            style("DRY RUN MODE - No changes will be made")
                .yellow()
                .bold()
        );
    }
    println!();

    // Text-mode installation steps
    let steps = [
        "Disk Partitioning",
        "Filesystem Setup",
        "Base System Installation",
        "Bootloader Configuration",
        "User Setup",
        "Network Configuration",
        "Finalization",
    ];

    println!("Installation steps:");
    for (i, step) in steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }
    println!();

    println!(
        "{}",
        style("For manual installation, you can perform these steps yourself:").cyan()
    );
    println!();
    println!("  1. Partition your disk:");
    println!("     # fdisk /dev/sdX  or  parted /dev/sdX");
    println!();
    println!("  2. Create filesystems:");
    println!("     # mkfs.ext4 /dev/sdX1");
    println!("     # mkswap /dev/sdX2");
    println!();
    println!("  3. Mount the target:");
    println!("     # mount /dev/sdX1 {}", args.target);
    println!();
    println!("  4. Install the base system:");
    println!("     # buckos --root {} install @system", args.target);
    println!();
    println!("  5. Configure the bootloader:");
    println!("     # chroot {} grub-install /dev/sdX", args.target);
    println!(
        "     # chroot {} grub-mkconfig -o /boot/grub/grub.cfg",
        args.target
    );
    println!();
    println!("  6. Set up users and finalize:");
    println!("     # chroot {} passwd root", args.target);
    println!("     # chroot {} useradd -m -G wheel username", args.target);
    println!();

    println!(
        "{}",
        style("Text-mode interactive installer coming soon!").yellow()
    );
    println!("For now, please use the GUI installer or follow the manual steps above.");

    Ok(())
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
