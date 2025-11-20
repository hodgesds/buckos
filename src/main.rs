mod repository;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Buckos Package Manager - Source-based package manager inspired by Gentoo
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to buckos-build repository (auto-detected if not specified)
    #[clap(long = "repo-path", env = "BUCKOS_BUILD_PATH")]
    repo_path: Option<String>,

    /// Enable verbose output
    #[clap(short, long)]
    verbose: bool,

    /// Subcommand to execute
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show repository information
    Info,
    /// Sync the repository
    Sync,
    /// Install packages
    Install {
        /// Packages to install
        packages: Vec<String>,
        /// Target root directory for installation
        #[clap(long)]
        root: Option<String>,
    },
    /// Search for packages
    Search {
        /// Search query
        query: String,
    },
    /// Show package information
    Show {
        /// Package name
        package: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Detect repository location
    let repo_path = repository::detect_repository_path(args.repo_path.as_deref())?;

    if args.verbose {
        eprintln!("Using repository: {}", repo_path.display());
    }

    // Execute command
    match args.command {
        Some(Commands::Info) => {
            println!("Buckos Build Repository");
            println!("=======================");
            println!("Location: {}", repo_path.display());
            println!();
            println!("Standard repository locations:");
            for location in repository::STANDARD_REPO_LOCATIONS {
                println!("  - {}", location);
            }
            println!();
            println!("Environment variable: BUCKOS_BUILD_PATH");
        }
        Some(Commands::Sync) => {
            println!("Syncing repository from {}", repo_path.display());
            println!("TODO: Implement repository sync");
        }
        Some(Commands::Install { packages, root }) => {
            println!("Installing packages: {:?}", packages);
            if let Some(root_path) = root {
                println!("Target root: {}", root_path);
            }
            println!("Repository: {}", repo_path.display());
            println!("TODO: Implement package installation");
        }
        Some(Commands::Search { query }) => {
            println!("Searching for: {}", query);
            println!("Repository: {}", repo_path.display());
            println!("TODO: Implement package search");
        }
        Some(Commands::Show { package }) => {
            println!("Package information: {}", package);
            println!("Repository: {}", repo_path.display());
            println!("TODO: Implement package info display");
        }
        None => {
            println!("Buckos Package Manager");
            println!();
            println!("Repository: {}", repo_path.display());
            println!();
            println!("Use --help to see available commands");
        }
    }

    Ok(())
}
