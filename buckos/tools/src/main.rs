//! Buckos System Tools
//!
//! A collection of system administration and development utilities.

use clap::{Parser, Subcommand};
use console::style;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System};

#[derive(Parser)]
#[command(
    name = "buckos-tools",
    about = "Buckos System Tools - Collection of system utilities",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List block devices
    Lsblk,

    /// Show hardware information
    Hwinfo,

    /// Display directory tree
    Tree(TreeArgs),

    /// Show environment information
    Envinfo,

    /// Show network interfaces
    Netinfo,

    /// Show memory information
    Meminfo,

    /// Show CPU information
    Cpuinfo,

    /// System health check
    Syscheck,

    /// Show disk usage
    Diskfree,

    /// Show process information
    Ps(PsArgs),

    /// Generate system report
    Report(ReportArgs),
}

#[derive(clap::Args)]
struct TreeArgs {
    /// Directory to display
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum depth
    #[arg(short, long, default_value = "3")]
    depth: usize,

    /// Show hidden files
    #[arg(short = 'a', long)]
    all: bool,
}

#[derive(clap::Args)]
struct PsArgs {
    /// Show all processes
    #[arg(short, long)]
    all: bool,

    /// Sort by field (cpu, mem, pid)
    #[arg(short, long, default_value = "cpu")]
    sort: String,
}

#[derive(clap::Args)]
struct ReportArgs {
    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Lsblk => cmd_lsblk(),
        Commands::Hwinfo => cmd_hwinfo(),
        Commands::Tree(args) => cmd_tree(args),
        Commands::Envinfo => cmd_envinfo(),
        Commands::Netinfo => cmd_netinfo(),
        Commands::Meminfo => cmd_meminfo(),
        Commands::Cpuinfo => cmd_cpuinfo(),
        Commands::Syscheck => cmd_syscheck(),
        Commands::Diskfree => cmd_diskfree(),
        Commands::Ps(args) => cmd_ps(args),
        Commands::Report(args) => cmd_report(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", style("Error").red().bold(), e);
            ExitCode::FAILURE
        }
    }
}

fn cmd_lsblk() -> Result<(), String> {
    println!("{}", style("Block Devices").bold().underlined());
    println!();

    let disks = Disks::new_with_refreshed_list();

    println!(
        "{:<20} {:<15} {:<12} {:<10} {}",
        "NAME", "SIZE", "TYPE", "MOUNT", "FS"
    );
    println!("{}", "-".repeat(70));

    for disk in disks.list() {
        let name = disk
            .name()
            .to_string_lossy()
            .to_string();
        let mount = disk.mount_point().to_string_lossy().to_string();
        let fs = disk.file_system().to_string_lossy().to_string();
        let total = format_bytes(disk.total_space());
        let kind = format!("{:?}", disk.kind());

        println!(
            "{:<20} {:<15} {:<12} {:<10} {}",
            name, total, kind, mount, fs
        );
    }

    Ok(())
}

fn cmd_hwinfo() -> Result<(), String> {
    let sys = System::new_all();

    println!("{}", style("Hardware Information").bold().underlined());
    println!();

    // System info
    println!("{}", style("System:").cyan().bold());
    println!("  Host name: {}", System::host_name().unwrap_or_default());
    println!("  Kernel: {}", System::kernel_version().unwrap_or_default());
    println!("  OS: {}", System::name().unwrap_or_default());
    println!("  OS Version: {}", System::os_version().unwrap_or_default());
    println!();

    // CPU info
    println!("{}", style("CPU:").cyan().bold());
    if let Some(cpu) = sys.cpus().first() {
        println!("  Brand: {}", cpu.brand());
        println!("  Vendor: {}", cpu.vendor_id());
    }
    println!("  Cores: {}", sys.cpus().len());
    println!("  Physical cores: {}", sys.physical_core_count().unwrap_or(0));
    println!();

    // Memory info
    println!("{}", style("Memory:").cyan().bold());
    println!("  Total: {}", format_bytes(sys.total_memory()));
    println!("  Used: {}", format_bytes(sys.used_memory()));
    println!("  Free: {}", format_bytes(sys.free_memory()));
    println!("  Swap Total: {}", format_bytes(sys.total_swap()));
    println!("  Swap Used: {}", format_bytes(sys.used_swap()));
    println!();

    // Disk info
    let disks = Disks::new_with_refreshed_list();
    println!("{}", style("Storage:").cyan().bold());
    let mut total_storage = 0u64;
    let mut total_available = 0u64;
    for disk in disks.list() {
        total_storage += disk.total_space();
        total_available += disk.available_space();
    }
    println!("  Total: {}", format_bytes(total_storage));
    println!("  Available: {}", format_bytes(total_available));
    println!("  Disks: {}", disks.list().len());

    Ok(())
}

fn cmd_tree(args: TreeArgs) -> Result<(), String> {
    let path = args.path.canonicalize().map_err(|e| e.to_string())?;

    println!("{}", path.display());
    print_tree(&path, "", args.depth, 0, args.all)?;

    Ok(())
}

fn print_tree(
    path: &Path,
    prefix: &str,
    max_depth: usize,
    current_depth: usize,
    show_hidden: bool,
) -> Result<(), String> {
    if current_depth >= max_depth {
        return Ok(());
    }

    let entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            show_hidden || !e.file_name().to_string_lossy().starts_with('.')
        })
        .collect();

    let count = entries.len();

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let new_prefix = if is_last { "    " } else { "│   " };

        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);

        if is_dir {
            println!("{}{}{}", prefix, connector, style(&name).blue().bold());
            print_tree(
                &entry.path(),
                &format!("{}{}", prefix, new_prefix),
                max_depth,
                current_depth + 1,
                show_hidden,
            )?;
        } else {
            let size = metadata.map(|m| m.len()).unwrap_or(0);
            println!(
                "{}{}{} ({})",
                prefix,
                connector,
                name,
                format_bytes(size)
            );
        }
    }

    Ok(())
}

fn cmd_envinfo() -> Result<(), String> {
    println!("{}", style("Environment Information").bold().underlined());
    println!();

    let important_vars = [
        "USER",
        "HOME",
        "SHELL",
        "PATH",
        "LANG",
        "TERM",
        "EDITOR",
        "XDG_SESSION_TYPE",
        "XDG_CURRENT_DESKTOP",
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "RUST_LOG",
        "CC",
        "CXX",
        "CFLAGS",
        "CXXFLAGS",
        "LDFLAGS",
        "PKG_CONFIG_PATH",
    ];

    for var in &important_vars {
        if let Ok(value) = std::env::var(var) {
            let display_value = if value.len() > 80 {
                format!("{}...", &value[..77])
            } else {
                value
            };
            println!("  {}: {}", style(var).cyan(), display_value);
        }
    }

    Ok(())
}

fn cmd_netinfo() -> Result<(), String> {
    println!("{}", style("Network Interfaces").bold().underlined());
    println!();

    let networks = Networks::new_with_refreshed_list();

    println!(
        "{:<20} {:<15} {:<15} {:<15} {}",
        "INTERFACE", "RX", "TX", "RX/s", "TX/s"
    );
    println!("{}", "-".repeat(80));

    for (name, data) in networks.list() {
        println!(
            "{:<20} {:<15} {:<15} {:<15} {}",
            name,
            format_bytes(data.total_received()),
            format_bytes(data.total_transmitted()),
            format_bytes(data.received()),
            format_bytes(data.transmitted())
        );
    }

    Ok(())
}

fn cmd_meminfo() -> Result<(), String> {
    let sys = System::new_all();

    println!("{}", style("Memory Information").bold().underlined());
    println!();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = sys.free_memory();
    let available = sys.available_memory();

    println!("  {:<15} {}", style("Total:").cyan(), format_bytes(total));
    println!("  {:<15} {}", style("Used:").cyan(), format_bytes(used));
    println!("  {:<15} {}", style("Free:").cyan(), format_bytes(free));
    println!("  {:<15} {}", style("Available:").cyan(), format_bytes(available));
    println!();

    // Memory usage bar
    let usage_percent = (used as f64 / total as f64 * 100.0) as u32;
    print!("  [");
    let bar_width = 50;
    let filled = (bar_width * usage_percent / 100) as usize;
    for i in 0..bar_width as usize {
        if i < filled {
            if usage_percent > 90 {
                print!("{}", style("█").red());
            } else if usage_percent > 70 {
                print!("{}", style("█").yellow());
            } else {
                print!("{}", style("█").green());
            }
        } else {
            print!("░");
        }
    }
    println!("] {}%", usage_percent);
    println!();

    // Swap
    println!("{}", style("Swap:").cyan().bold());
    let swap_total = sys.total_swap();
    let swap_used = sys.used_swap();

    println!("  {:<15} {}", "Total:", format_bytes(swap_total));
    println!("  {:<15} {}", "Used:", format_bytes(swap_used));

    if swap_total > 0 {
        let swap_percent = (swap_used as f64 / swap_total as f64 * 100.0) as u32;
        print!("  [");
        let filled = (bar_width * swap_percent / 100) as usize;
        for i in 0..bar_width as usize {
            if i < filled {
                print!("{}", style("█").yellow());
            } else {
                print!("░");
            }
        }
        println!("] {}%", swap_percent);
    }

    Ok(())
}

fn cmd_cpuinfo() -> Result<(), String> {
    let sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
    );

    println!("{}", style("CPU Information").bold().underlined());
    println!();

    if let Some(cpu) = sys.cpus().first() {
        println!("  {:<20} {}", style("Brand:").cyan(), cpu.brand());
        println!("  {:<20} {}", style("Vendor:").cyan(), cpu.vendor_id());
    }

    println!(
        "  {:<20} {}",
        style("Logical cores:").cyan(),
        sys.cpus().len()
    );
    println!(
        "  {:<20} {}",
        style("Physical cores:").cyan(),
        sys.physical_core_count().unwrap_or(0)
    );
    println!();

    println!("{}", style("Per-Core Usage:").cyan().bold());

    // Need to wait a bit for accurate CPU usage
    std::thread::sleep(std::time::Duration::from_millis(200));
    let sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
    );

    for (i, cpu) in sys.cpus().iter().enumerate() {
        let usage = cpu.cpu_usage() as u32;
        print!("  CPU{:<3} [", i);
        let bar_width = 30;
        let filled = (bar_width * usage / 100) as usize;
        for j in 0..bar_width as usize {
            if j < filled {
                if usage > 90 {
                    print!("{}", style("█").red());
                } else if usage > 70 {
                    print!("{}", style("█").yellow());
                } else {
                    print!("{}", style("█").green());
                }
            } else {
                print!("░");
            }
        }
        println!("] {:>3}% @ {} MHz", usage, cpu.frequency());
    }

    Ok(())
}

fn cmd_syscheck() -> Result<(), String> {
    println!("{}", style("System Health Check").bold().underlined());
    println!();

    let sys = System::new_all();
    let mut issues = Vec::new();

    // Check memory usage
    let mem_usage = sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0;
    if mem_usage > 90.0 {
        issues.push(format!("High memory usage: {:.1}%", mem_usage));
        println!("  {} Memory usage: {:.1}%", style("✗").red(), mem_usage);
    } else if mem_usage > 70.0 {
        println!("  {} Memory usage: {:.1}%", style("⚠").yellow(), mem_usage);
    } else {
        println!("  {} Memory usage: {:.1}%", style("✓").green(), mem_usage);
    }

    // Check swap usage
    if sys.total_swap() > 0 {
        let swap_usage = sys.used_swap() as f64 / sys.total_swap() as f64 * 100.0;
        if swap_usage > 80.0 {
            issues.push(format!("High swap usage: {:.1}%", swap_usage));
            println!("  {} Swap usage: {:.1}%", style("✗").red(), swap_usage);
        } else if swap_usage > 50.0 {
            println!("  {} Swap usage: {:.1}%", style("⚠").yellow(), swap_usage);
        } else {
            println!("  {} Swap usage: {:.1}%", style("✓").green(), swap_usage);
        }
    }

    // Check disk space
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy().to_string();
        if disk.total_space() > 0 {
            let usage = (disk.total_space() - disk.available_space()) as f64
                / disk.total_space() as f64
                * 100.0;
            if usage > 90.0 {
                issues.push(format!("Low disk space on {}: {:.1}% used", mount, usage));
                println!(
                    "  {} Disk {} usage: {:.1}%",
                    style("✗").red(),
                    mount,
                    usage
                );
            } else if usage > 80.0 {
                println!(
                    "  {} Disk {} usage: {:.1}%",
                    style("⚠").yellow(),
                    mount,
                    usage
                );
            } else {
                println!(
                    "  {} Disk {} usage: {:.1}%",
                    style("✓").green(),
                    mount,
                    usage
                );
            }
        }
    }

    // Check load average
    let load = System::load_average();
    let cpu_count = sys.cpus().len() as f64;
    if load.one > cpu_count * 2.0 {
        issues.push(format!("High load average: {:.2}", load.one));
        println!(
            "  {} Load average: {:.2} {:.2} {:.2}",
            style("✗").red(),
            load.one,
            load.five,
            load.fifteen
        );
    } else if load.one > cpu_count {
        println!(
            "  {} Load average: {:.2} {:.2} {:.2}",
            style("⚠").yellow(),
            load.one,
            load.five,
            load.fifteen
        );
    } else {
        println!(
            "  {} Load average: {:.2} {:.2} {:.2}",
            style("✓").green(),
            load.one,
            load.five,
            load.fifteen
        );
    }

    // Check uptime
    let uptime = System::uptime();
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let mins = (uptime % 3600) / 60;
    println!(
        "  {} Uptime: {}d {}h {}m",
        style("ℹ").blue(),
        days,
        hours,
        mins
    );

    println!();
    if issues.is_empty() {
        println!(
            "{}",
            style("System health: OK").green().bold()
        );
    } else {
        println!(
            "{} {} issue(s) found",
            style("System health:").yellow().bold(),
            issues.len()
        );
    }

    Ok(())
}

fn cmd_diskfree() -> Result<(), String> {
    println!("{}", style("Disk Usage").bold().underlined());
    println!();

    let disks = Disks::new_with_refreshed_list();

    println!(
        "{:<30} {:<12} {:<12} {:<12} {:<8} {}",
        "FILESYSTEM", "SIZE", "USED", "AVAIL", "USE%", "MOUNT"
    );
    println!("{}", "-".repeat(90));

    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy().to_string();
        let name = disk.name().to_string_lossy().to_string();
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;

        if total == 0 {
            continue;
        }

        let usage_percent = (used as f64 / total as f64 * 100.0) as u32;

        let usage_str = if usage_percent > 90 {
            style(format!("{}%", usage_percent)).red().to_string()
        } else if usage_percent > 70 {
            style(format!("{}%", usage_percent)).yellow().to_string()
        } else {
            format!("{}%", usage_percent)
        };

        println!(
            "{:<30} {:<12} {:<12} {:<12} {:<8} {}",
            name,
            format_bytes(total),
            format_bytes(used),
            format_bytes(available),
            usage_str,
            mount
        );
    }

    Ok(())
}

fn cmd_ps(args: PsArgs) -> Result<(), String> {
    let sys = System::new_all();

    println!("{}", style("Process List").bold().underlined());
    println!();

    println!(
        "{:<8} {:<10} {:<10} {:<40}",
        "PID", "CPU%", "MEM", "NAME"
    );
    println!("{}", "-".repeat(70));

    let mut processes: Vec<_> = sys.processes().iter().collect();

    // Sort based on argument
    match args.sort.as_str() {
        "mem" => processes.sort_by(|a, b| {
            b.1.memory().cmp(&a.1.memory())
        }),
        "pid" => processes.sort_by(|a, b| a.0.cmp(b.0)),
        _ => processes.sort_by(|a, b| {
            b.1.cpu_usage()
                .partial_cmp(&a.1.cpu_usage())
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }

    let limit = if args.all { processes.len() } else { 20 };

    for (pid, process) in processes.iter().take(limit) {
        let name = process.name().to_string_lossy();
        let name_display = if name.len() > 38 {
            format!("{}...", &name[..35])
        } else {
            name.to_string()
        };

        println!(
            "{:<8} {:<10.1} {:<10} {:<40}",
            pid.as_u32(),
            process.cpu_usage(),
            format_bytes(process.memory()),
            name_display
        );
    }

    if !args.all && processes.len() > limit {
        println!("\n... and {} more processes", processes.len() - limit);
    }

    Ok(())
}

fn cmd_report(args: ReportArgs) -> Result<(), String> {
    let sys = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();

    let report = if args.format == "json" {
        // JSON format
        let mut json = String::from("{\n");

        // System info
        json.push_str(&format!(
            "  \"system\": {{\n    \"hostname\": \"{}\",\n    \"kernel\": \"{}\",\n    \"os\": \"{}\"\n  }},\n",
            System::host_name().unwrap_or_default(),
            System::kernel_version().unwrap_or_default(),
            System::name().unwrap_or_default()
        ));

        // Memory
        json.push_str(&format!(
            "  \"memory\": {{\n    \"total\": {},\n    \"used\": {},\n    \"free\": {}\n  }},\n",
            sys.total_memory(),
            sys.used_memory(),
            sys.free_memory()
        ));

        // CPU
        json.push_str(&format!(
            "  \"cpu\": {{\n    \"cores\": {},\n    \"physical_cores\": {}\n  }},\n",
            sys.cpus().len(),
            sys.physical_core_count().unwrap_or(0)
        ));

        // Disks
        json.push_str("  \"disks\": [\n");
        for (i, disk) in disks.list().iter().enumerate() {
            let comma = if i < disks.list().len() - 1 { "," } else { "" };
            json.push_str(&format!(
                "    {{\"mount\": \"{}\", \"total\": {}, \"available\": {}}}{}\\n",
                disk.mount_point().to_string_lossy(),
                disk.total_space(),
                disk.available_space(),
                comma
            ));
        }
        json.push_str("  ]\n");
        json.push_str("}\n");
        json
    } else {
        // Text format
        let mut report = String::new();

        report.push_str("=== System Report ===\n\n");

        report.push_str("System:\n");
        report.push_str(&format!("  Hostname: {}\n", System::host_name().unwrap_or_default()));
        report.push_str(&format!("  Kernel: {}\n", System::kernel_version().unwrap_or_default()));
        report.push_str(&format!("  OS: {} {}\n", System::name().unwrap_or_default(), System::os_version().unwrap_or_default()));
        report.push_str("\n");

        report.push_str("Memory:\n");
        report.push_str(&format!("  Total: {}\n", format_bytes(sys.total_memory())));
        report.push_str(&format!("  Used: {}\n", format_bytes(sys.used_memory())));
        report.push_str(&format!("  Free: {}\n", format_bytes(sys.free_memory())));
        report.push_str("\n");

        report.push_str("CPU:\n");
        if let Some(cpu) = sys.cpus().first() {
            report.push_str(&format!("  Brand: {}\n", cpu.brand()));
        }
        report.push_str(&format!("  Cores: {}\n", sys.cpus().len()));
        report.push_str("\n");

        report.push_str("Disks:\n");
        for disk in disks.list() {
            report.push_str(&format!(
                "  {}: {} total, {} available\n",
                disk.mount_point().to_string_lossy(),
                format_bytes(disk.total_space()),
                format_bytes(disk.available_space())
            ));
        }
        report.push_str("\n");

        report.push_str("Network Interfaces:\n");
        for (name, _data) in networks.list() {
            report.push_str(&format!("  {}\n", name));
        }

        report
    };

    if let Some(output) = args.output {
        fs::write(&output, &report).map_err(|e| e.to_string())?;
        println!("Report saved to {}", output.display());
    } else {
        print!("{}", report);
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
