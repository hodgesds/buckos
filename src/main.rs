mod repository;

use anyhow::Result;
use clap::{Parser, Subcommand};
use package::{Config, InstallOptions, PackageManager};
use std::path::PathBuf;

/// Buckos Package Manager - Source-based package manager inspired by Gentoo
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to buckos-build repository (auto-detected if not specified)
    #[clap(long = "repo-path", env = "BUCKOS_BUILD_PATH")]
    repo_path: Option<String>,

    /// Target root directory for installation (default: /)
    #[clap(long)]
    root: Option<String>,

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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Detect repository location
    let repo_path = repository::detect_repository_path(args.repo_path.as_deref())?;

    // Parse global root option
    let global_root = args.root.as_ref().map(PathBuf::from);

    if args.verbose {
        eprintln!("Using repository: {}", repo_path.display());
        if let Some(root) = &global_root {
            eprintln!("Using root: {}", root.display());
        }
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
            let pm = create_package_manager(&repo_path, global_root.as_ref()).await?;
            println!("Syncing repository from {}", repo_path.display());
            pm.sync().await?;
            println!("Repository synced successfully");
        }
        Some(Commands::Install { packages, root }) => {
            // Command-specific root overrides global root
            let target_root = root.as_ref().map(PathBuf::from).or(global_root);
            let pm = create_package_manager(&repo_path, target_root.as_ref()).await?;

            // Expand package sets like @system, @world
            let expanded = expand_package_sets(&pm, &packages).await?;

            println!("Installing packages: {:?}", expanded);
            if let Some(root_path) = &target_root {
                println!("Target root: {}", root_path.display());
            }

            let opts = InstallOptions {
                build: true,
                ..Default::default()
            };

            pm.install(&expanded, opts).await?;
            println!("Installation complete");
        }
        Some(Commands::Search { query }) => {
            let pm = create_package_manager(&repo_path, global_root.as_ref()).await?;
            println!("Searching for: {}", query);
            let results = pm.search(&query).await?;

            if results.is_empty() {
                println!("No packages found matching '{}'", query);
            } else {
                println!("\nFound {} package(s):", results.len());
                for pkg in results {
                    println!("  {}/{}-{}", pkg.id.category, pkg.id.name, pkg.version);
                    if !pkg.description.is_empty() {
                        println!("    {}", pkg.description);
                    }
                }
            }
        }
        Some(Commands::Show { package }) => {
            let pm = create_package_manager(&repo_path, global_root.as_ref()).await?;
            println!("Package information: {}", package);

            if let Some(pkg) = pm.info(&package).await? {
                println!("\nPackage: {}/{}", pkg.id.category, pkg.id.name);
                println!("Version: {}", pkg.version);
                if !pkg.description.is_empty() {
                    println!("Description: {}", pkg.description);
                }
                if !pkg.use_flags.is_empty() {
                    println!("\nUSE flags:");
                    for flag in &pkg.use_flags {
                        println!("  {} - {}", flag.name, flag.description);
                    }
                }
                if !pkg.dependencies.is_empty() {
                    println!("\nDependencies: {:?}", pkg.dependencies);
                }
            } else {
                println!("Package '{}' not found", package);
            }
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

/// Create a PackageManager instance with the given repository path
async fn create_package_manager(
    repo_path: &PathBuf,
    target_root: Option<&PathBuf>,
) -> Result<PackageManager> {
    let mut config = Config::load().unwrap_or_default();

    // Set buck_repo to the detected buckos-build path
    config.buck_repo = repo_path.clone();

    // Set target root if provided
    if let Some(root) = target_root {
        config.root = root.clone();
        // Also update db and cache paths to be under the target root
        config.db_path = root.join("var/db/buckos");
        config.cache_dir = root.join("var/cache/buckos");
    }

    // Create the package manager
    Ok(PackageManager::new(config).await?)
}

/// Expand package sets (@system, @world, @selected) to individual packages
async fn expand_package_sets(pm: &PackageManager, packages: &[String]) -> Result<Vec<String>> {
    let mut result = Vec::new();

    for pkg in packages {
        if pkg.starts_with('@') {
            match pkg.as_str() {
                "@world" => {
                    let world = pm.get_world_set().await?;
                    result.extend(world.packages.iter().map(|p| p.full_name()));
                }
                "@system" => {
                    let system = pm.get_system_set().await?;
                    result.extend(system.packages.iter().map(|p| p.full_name()));
                }
                "@selected" => {
                    let selected = pm.get_selected_set().await?;
                    result.extend(selected.packages.iter().map(|p| p.full_name()));
                }
                _ => {
                    return Err(anyhow::anyhow!("Unknown package set: {}", pkg));
                }
            }
        } else {
            result.push(pkg.clone());
        }
    }

    Ok(result)
}
