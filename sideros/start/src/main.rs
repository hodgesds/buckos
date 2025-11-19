//! Sideros init system binary.
//!
//! This is the main entry point for the sideros init system.
//! It can run as PID 1 or as a service management tool.

use clap::{Parser, Subcommand};
use sideros_start::{create_test_init, Init, InitConfig, ServiceDefinition, ServiceStatus, ShutdownType};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "start",
    about = "Sideros init system - PID 1 service manager",
    version,
    author
)]
struct Cli {
    /// Services directory
    #[arg(short, long, default_value = "/etc/sideros/services")]
    services_dir: PathBuf,

    /// Don't require running as PID 1
    #[arg(long)]
    no_pid1: bool,

    /// Don't mount virtual filesystems
    #[arg(long)]
    no_mount: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as init system (PID 1)
    Init,

    /// Start a service
    Start {
        /// Service name
        name: String,
    },

    /// Stop a service
    Stop {
        /// Service name
        name: String,
    },

    /// Restart a service
    Restart {
        /// Service name
        name: String,
    },

    /// Reload a service configuration
    Reload {
        /// Service name
        name: String,
    },

    /// Show service status
    Status {
        /// Service name (optional, shows all if not specified)
        name: Option<String>,
    },

    /// List all services
    List,

    /// Enable a service for auto-start
    Enable {
        /// Service name
        name: String,
    },

    /// Disable a service from auto-start
    Disable {
        /// Service name
        name: String,
    },

    /// Mask a service to prevent it from starting
    Mask {
        /// Service name
        name: String,
    },

    /// Unmask a service to allow it to start
    Unmask {
        /// Service name
        name: String,
    },

    /// Show service logs
    Logs {
        /// Service name
        name: String,
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "100")]
        lines: usize,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Show service dependency graph
    Deps {
        /// Service name (optional, shows all if not specified)
        name: Option<String>,
    },

    /// Analyze boot performance
    Analyze {
        /// Analysis type: blame, critical-chain, or time
        #[arg(default_value = "blame")]
        analysis_type: String,
    },

    /// Create a new service definition
    New {
        /// Service name
        name: String,
        /// Command to execute
        exec: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Instantiate a template service
    Instantiate {
        /// Template service name
        template: String,
        /// Instance name
        instance: String,
    },

    /// Shutdown the system
    Shutdown {
        /// Shutdown type: poweroff, reboot, or halt
        #[arg(default_value = "poweroff")]
        shutdown_type: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) | None => {
            // Run as init system
            run_init(&cli).await?;
        }

        Some(Commands::Start { name }) => {
            // Start a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().start_service(&name).await?;
            info!(service = %name, "Service started");
        }

        Some(Commands::Stop { name }) => {
            // Stop a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().stop_service(&name).await?;
            info!(service = %name, "Service stopped");
        }

        Some(Commands::Restart { name }) => {
            // Restart a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().restart_service(&name).await?;
            info!(service = %name, "Service restarted");
        }

        Some(Commands::Reload { name }) => {
            // Reload a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().reload_service(&name).await?;
            info!(service = %name, "Service reloaded");
        }

        Some(Commands::Status { name }) => {
            // Show service status
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;

            if let Some(name) = name {
                let status = init.manager().get_status(&name).await?;
                print_status(&status);
            } else {
                let statuses = init.manager().get_all_status().await;
                if statuses.is_empty() {
                    println!("No services found");
                } else {
                    for status in statuses {
                        print_status(&status);
                        println!();
                    }
                }
            }
        }

        Some(Commands::List) => {
            // List all services
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;

            let services = init.manager().list_services().await;
            if services.is_empty() {
                println!("No services found");
            } else {
                println!("Services:");
                for name in services {
                    println!("  {}", name);
                }
            }
        }

        Some(Commands::Enable { name }) => {
            // Enable a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().enable_service(&name).await?;
            println!("Enabled {}", name);
        }

        Some(Commands::Disable { name }) => {
            // Disable a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().disable_service(&name).await?;
            println!("Disabled {}", name);
        }

        Some(Commands::Mask { name }) => {
            // Mask a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().mask_service(&name).await?;
            println!("Masked {}", name);
        }

        Some(Commands::Unmask { name }) => {
            // Unmask a service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().unmask_service(&name).await?;
            println!("Unmasked {}", name);
        }

        Some(Commands::Logs { name, lines, follow: _ }) => {
            // Show service logs
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;

            let logs = init.manager().get_logs(&name, Some(lines)).await;
            if logs.is_empty() {
                println!("No logs found for {}", name);
            } else {
                for entry in logs {
                    println!("{}", entry.format());
                }
            }
        }

        Some(Commands::Deps { name }) => {
            // Show dependency graph
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;

            let graph = init.manager().get_dependency_graph().await;

            if let Some(name) = name {
                // Show deps for specific service
                if let Some(node) = graph.iter().find(|n| n.name == name) {
                    println!("Dependencies for {}:", name);
                    if !node.requires.is_empty() {
                        println!("  Requires: {}", node.requires.join(", "));
                    }
                    if !node.wants.is_empty() {
                        println!("  Wants: {}", node.wants.join(", "));
                    }
                    if !node.before.is_empty() {
                        println!("  Before: {}", node.before.join(", "));
                    }
                    if !node.after.is_empty() {
                        println!("  After: {}", node.after.join(", "));
                    }
                } else {
                    error!("Service not found: {}", name);
                }
            } else {
                // Show all deps
                println!("Dependency Graph:");
                for node in graph {
                    if !node.requires.is_empty() || !node.wants.is_empty() {
                        println!("  {} -> {:?}", node.name, node.requires);
                    }
                }
            }
        }

        Some(Commands::Analyze { analysis_type }) => {
            // Analyze boot performance
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;

            match analysis_type.as_str() {
                "blame" => {
                    let blame = init.manager().get_boot_blame().await;
                    if blame.is_empty() {
                        println!("No boot timing data available");
                    } else {
                        println!("Startup finished in {}ms", init.manager().get_total_boot_time());
                        println!();
                        for timing in blame {
                            println!("{:>8}ms {}", timing.duration_ms, timing.name);
                        }
                    }
                }
                "critical-chain" => {
                    let chain = init.manager().get_critical_chain().await;
                    if chain.is_empty() {
                        println!("No critical chain data available");
                    } else {
                        println!("Critical chain:");
                        for (i, name) in chain.iter().enumerate() {
                            let indent = "  ".repeat(i);
                            println!("{}{}", indent, name);
                        }
                    }
                }
                "time" => {
                    println!("Total boot time: {}ms", init.manager().get_total_boot_time());
                }
                _ => {
                    error!("Unknown analysis type: {}", analysis_type);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Instantiate { template, instance }) => {
            // Instantiate a template service
            let init = create_test_init(cli.services_dir)?;
            init.manager().load_services().await?;
            init.manager().instantiate_template(&template, &instance).await?;
            println!("Instantiated {}@{}", template, instance);
        }

        Some(Commands::New { name, exec, output }) => {
            // Create a new service definition
            let def = ServiceDefinition::new(&name, &exec);

            let path = output.unwrap_or_else(|| {
                cli.services_dir.join(format!("{}.toml", name))
            });

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            def.to_file(&path)?;
            println!("Created service definition: {}", path.display());
        }

        Some(Commands::Shutdown { shutdown_type }) => {
            // Request shutdown
            let shutdown_type = match shutdown_type.as_str() {
                "poweroff" | "power-off" => ShutdownType::PowerOff,
                "reboot" => ShutdownType::Reboot,
                "halt" => ShutdownType::Halt,
                _ => {
                    error!("Unknown shutdown type: {}", shutdown_type);
                    std::process::exit(1);
                }
            };

            // TODO: Communicate with running init process
            // For now, just print a message
            println!("Shutdown type: {:?}", shutdown_type);
            println!("Note: Direct shutdown communication not yet implemented");
        }
    }

    Ok(())
}

/// Run as the init system.
async fn run_init(cli: &Cli) -> anyhow::Result<()> {
    let config = InitConfig {
        services_dir: cli.services_dir.clone(),
        mount_filesystems: !cli.no_mount,
        require_pid1: !cli.no_pid1,
    };

    let init = Init::new(config)?;
    init.run().await?;

    Ok(())
}

/// Print service status.
fn print_status(status: &ServiceStatus) {
    let state_symbol = match status.state {
        sideros_start::ServiceState::Running => "●",
        sideros_start::ServiceState::Failed => "×",
        sideros_start::ServiceState::Inactive | sideros_start::ServiceState::Stopped => "○",
        _ => "◌",
    };

    println!("{} {} - {}", state_symbol, status.name, status.description);
    println!("   State: {}", status.state);

    if status.masked {
        println!("   Masked: yes");
    }

    if status.enabled {
        println!("   Enabled: yes");
    }

    if let Some(pid) = status.main_pid {
        println!("   PID: {}", pid);
    }

    if let Some(uptime) = status.uptime_secs {
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;
        println!("   Uptime: {}h {}m {}s", hours, minutes, seconds);
    }

    if status.restart_count > 0 {
        println!("   Restarts: {}", status.restart_count);
    }

    // Show health status if not "none"
    if status.health_status != sideros_start::HealthStatus::None {
        println!("   Health: {}", status.health_status);
    }

    if let Some(boot_ms) = status.boot_duration_ms {
        println!("   Boot time: {}ms", boot_ms);
    }

    if !status.requires.is_empty() {
        println!("   Requires: {}", status.requires.join(", "));
    }
}
