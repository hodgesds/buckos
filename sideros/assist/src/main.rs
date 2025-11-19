//! Sideros System Diagnostic CLI
//!
//! A command-line tool for collecting system diagnostic information
//! while maintaining privacy control.

use anyhow::{Context, Result};
use clap::Parser;
use console::{style, Term};
use dialoguer::{Confirm, MultiSelect};
use tracing_subscriber::EnvFilter;

use sideros_assist::{
    cli::{Cli, CollectArgs, Commands, PrivacyCommands, PrivacyPreset, SummaryArgs},
    collectors::{hardware::format_bytes, software::format_uptime, SystemDiagnostics},
    privacy::PrivacySettings,
    report::{DiagnosticReport, OutputFormat},
};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Collect(args) => run_collect(args, cli.quiet),
        Commands::Summary(args) => run_summary(args, cli.quiet),
        Commands::Privacy(args) => match args.command {
            PrivacyCommands::Show => show_privacy_settings(),
            PrivacyCommands::Presets => list_privacy_presets(),
            PrivacyCommands::Configure => configure_privacy(),
        },
    }
}

/// Run the collect command.
fn run_collect(args: CollectArgs, quiet: bool) -> Result<()> {
    // Build privacy settings from arguments
    let mut settings = match args.privacy {
        PrivacyPreset::Default => PrivacySettings::default(),
        PrivacyPreset::Minimal => PrivacySettings::minimal(),
        PrivacyPreset::Full => PrivacySettings::full(),
    };

    // Apply command-line overrides
    settings.collect_hardware = args.hardware;
    settings.collect_software = args.software;
    settings.collect_network = args.network;
    settings.collect_processes = args.processes;
    settings.redact_usernames = !args.no_redact_usernames;
    settings.redact_ips = !args.no_redact_ips;
    settings.redact_macs = !args.no_redact_macs;
    settings.redact_home_paths = !args.no_redact_home;
    settings.redact_hostnames = args.redact_hostnames;

    if !quiet {
        eprintln!("{}", style("Collecting system diagnostics...").cyan());
    }

    // Collect diagnostics
    let diagnostics =
        SystemDiagnostics::collect(&settings).context("Failed to collect system diagnostics")?;

    // Create report
    let report = DiagnosticReport::new(diagnostics, settings);
    let format: OutputFormat = args.format.into();
    let output = report.export(format).context("Failed to export report")?;

    // Interactive mode - preview and confirm
    if args.interactive {
        let term = Term::stdout();
        term.clear_screen()?;

        println!("{}", style("=== Report Preview ===").bold().green());
        println!();

        // Show a truncated preview
        let preview_lines: Vec<&str> = output.lines().take(50).collect();
        for line in &preview_lines {
            println!("{}", line);
        }

        if output.lines().count() > 50 {
            println!(
                "{}",
                style(format!("... ({} more lines)", output.lines().count() - 50)).dim()
            );
        }

        println!();

        // Confirm before saving
        if let Some(path) = &args.output {
            let confirmed = Confirm::new()
                .with_prompt(format!("Save report to {}?", path.display()))
                .default(true)
                .interact()?;

            if confirmed {
                std::fs::write(path, &output)
                    .with_context(|| format!("Failed to write to {}", path.display()))?;

                if !quiet {
                    eprintln!(
                        "{}",
                        style(format!("Report saved to {}", path.display())).green()
                    );
                }
            } else {
                eprintln!("{}", style("Report not saved.").yellow());
            }
        } else {
            let confirmed = Confirm::new()
                .with_prompt("Display full report?")
                .default(true)
                .interact()?;

            if confirmed {
                println!("{}", output);
            }
        }
    } else {
        // Non-interactive mode
        if let Some(path) = &args.output {
            std::fs::write(path, &output)
                .with_context(|| format!("Failed to write to {}", path.display()))?;

            if !quiet {
                eprintln!(
                    "{}",
                    style(format!("Report saved to {}", path.display())).green()
                );
            }
        } else {
            println!("{}", output);
        }
    }

    Ok(())
}

/// Run the summary command.
fn run_summary(args: SummaryArgs, _quiet: bool) -> Result<()> {
    let mut settings = PrivacySettings::default();
    settings.collect_processes = args.processes;

    let diagnostics =
        SystemDiagnostics::collect(&settings).context("Failed to collect system diagnostics")?;

    println!("{}", style("System Summary").bold().cyan());
    println!("{}", style("=".repeat(50)).dim());
    println!();

    // Hardware summary
    if let Some(hw) = &diagnostics.hardware {
        println!("{}", style("Hardware:").bold());
        println!("  CPU: {} ({} cores)", hw.cpu.brand, hw.cpu.logical_cores);
        println!(
            "  Memory: {} / {} ({:.1}% used)",
            format_bytes(hw.memory.used_ram),
            format_bytes(hw.memory.total_ram),
            hw.memory.ram_usage_percent()
        );

        // Show disk summary
        let total_disk: u64 = hw.disks.iter().map(|d| d.total_space).sum();
        let available_disk: u64 = hw.disks.iter().map(|d| d.available_space).sum();
        println!(
            "  Storage: {} available of {}",
            format_bytes(available_disk),
            format_bytes(total_disk)
        );
        println!();
    }

    // Software summary
    if let Some(sw) = &diagnostics.software {
        println!("{}", style("System:").bold());
        println!("  OS: {} {}", sw.os.name, sw.os.version);
        println!("  Kernel: {}", sw.os.kernel_version);
        println!("  Uptime: {}", format_uptime(sw.os.uptime));
        println!();

        // Process summary
        if let Some(procs) = &sw.processes {
            println!("{}", style("Processes:").bold());
            println!("  Total: {}", procs.total_count);
            println!("  Running: {}", procs.running_count);
            println!();

            // Top 3 by memory
            println!("  Top by memory:");
            for proc in procs.top_by_memory.iter().take(3) {
                println!(
                    "    - {} ({}): {}",
                    proc.name,
                    proc.pid,
                    format_bytes(proc.memory)
                );
            }
        }
    }

    Ok(())
}

/// Show current privacy settings.
fn show_privacy_settings() -> Result<()> {
    let settings = PrivacySettings::default();

    println!("{}", style("Default Privacy Settings").bold().cyan());
    println!("{}", style("=".repeat(50)).dim());
    println!();

    println!("{}", style("Collection:").bold());
    println!("  Hardware: {}", bool_status(settings.collect_hardware));
    println!("  Software: {}", bool_status(settings.collect_software));
    println!("  Network: {}", bool_status(settings.collect_network));
    println!("  Processes: {}", bool_status(settings.collect_processes));
    println!();

    println!("{}", style("Redaction:").bold());
    println!("  Usernames: {}", bool_status(settings.redact_usernames));
    println!("  IP addresses: {}", bool_status(settings.redact_ips));
    println!("  MAC addresses: {}", bool_status(settings.redact_macs));
    println!("  Hostnames: {}", bool_status(settings.redact_hostnames));
    println!("  Home paths: {}", bool_status(settings.redact_home_paths));

    Ok(())
}

/// List available privacy presets.
fn list_privacy_presets() -> Result<()> {
    println!("{}", style("Available Privacy Presets").bold().cyan());
    println!("{}", style("=".repeat(50)).dim());
    println!();

    println!("{}", style("default").bold());
    println!("  Balanced privacy with useful diagnostics.");
    println!("  Collects all categories, redacts sensitive data.");
    println!();

    println!("{}", style("minimal").bold());
    println!("  Maximum privacy, minimum data collection.");
    println!("  Only collects essential hardware info.");
    println!();

    println!("{}", style("full").bold());
    println!("  Complete collection without redaction.");
    println!("  For local debugging only - do not share.");
    println!();

    println!(
        "Use with: {} collect --privacy <preset>",
        style("sideros-assist").green()
    );

    Ok(())
}

/// Interactively configure privacy settings.
fn configure_privacy() -> Result<()> {
    println!("{}", style("Privacy Configuration").bold().cyan());
    println!();

    // Select what to collect
    let collection_options = vec![
        "Hardware information (CPU, memory, disks)",
        "Software information (OS, kernel, environment)",
        "Network information (interfaces, traffic)",
        "Process information (running processes)",
    ];

    let collection_selections = MultiSelect::new()
        .with_prompt("What information should be collected?")
        .items(&collection_options)
        .defaults(&[true, true, true, true])
        .interact()?;

    // Select what to redact
    let redaction_options = vec![
        "Usernames",
        "IP addresses",
        "MAC addresses",
        "Hostnames",
        "Home directory paths",
    ];

    let redaction_selections = MultiSelect::new()
        .with_prompt("What information should be redacted?")
        .items(&redaction_options)
        .defaults(&[true, true, true, false, true])
        .interact()?;

    // Show summary
    println!();
    println!("{}", style("Configuration Summary").bold().green());
    println!();

    println!("Collecting:");
    for (i, opt) in collection_options.iter().enumerate() {
        let status = if collection_selections.contains(&i) {
            style("Yes").green()
        } else {
            style("No").red()
        };
        println!("  {} - {}", status, opt);
    }

    println!();
    println!("Redacting:");
    for (i, opt) in redaction_options.iter().enumerate() {
        let status = if redaction_selections.contains(&i) {
            style("Yes").green()
        } else {
            style("No").red()
        };
        println!("  {} - {}", status, opt);
    }

    println!();
    println!(
        "{}",
        style("Use these settings with command-line flags:").dim()
    );

    // Generate example command
    let mut flags = Vec::new();
    if !collection_selections.contains(&0) {
        flags.push("--hardware=false");
    }
    if !collection_selections.contains(&1) {
        flags.push("--software=false");
    }
    if !collection_selections.contains(&2) {
        flags.push("--network=false");
    }
    if !collection_selections.contains(&3) {
        flags.push("--processes=false");
    }
    if !redaction_selections.contains(&0) {
        flags.push("--no-redact-usernames");
    }
    if !redaction_selections.contains(&1) {
        flags.push("--no-redact-ips");
    }
    if !redaction_selections.contains(&2) {
        flags.push("--no-redact-macs");
    }
    if redaction_selections.contains(&3) {
        flags.push("--redact-hostnames");
    }
    if !redaction_selections.contains(&4) {
        flags.push("--no-redact-home");
    }

    if flags.is_empty() {
        println!("  sideros-assist collect");
    } else {
        println!("  sideros-assist collect {}", flags.join(" "));
    }

    Ok(())
}

/// Format a boolean as a styled status string.
fn bool_status(value: bool) -> console::StyledObject<&'static str> {
    if value {
        style("Enabled").green()
    } else {
        style("Disabled").red()
    }
}
