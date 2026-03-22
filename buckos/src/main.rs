mod repository;

use anyhow::Result;
use buckos_package::{Config, InstallOptions, PackageManager};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

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
    /// Manage system configuration
    Config {
        #[clap(subcommand)]
        action: ConfigAction,
    },
    /// Manage USE flags (modifier constraints for Buck2 builds)
    Use {
        #[clap(subcommand)]
        action: Option<UseAction>,
        /// USE flag changes (e.g., +wayland -gtk +ssl)
        #[clap(trailing_var_arg = true)]
        flags: Vec<String>,
    },
    /// Start MCP server (Model Context Protocol for AI assistants)
    Mcp {
        /// MCP server configuration file
        #[clap(long)]
        mcp_config: Option<String>,
        /// Run in user-mode (install to ~/.local, no root required)
        #[clap(long)]
        user_mode: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Display current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (use, profile, arch, cflags, cxxflags, ldflags, makeopts)
        key: String,
        /// Configuration value
        value: String,
    },
}

#[derive(Subcommand, Debug)]
enum UseAction {
    /// Show current USE flag configuration
    Show,
    /// Set global USE flags (replaces all)
    Set {
        /// USE flags (e.g., "X wayland ssl ipv6")
        flags: String,
    },
    /// Add or remove global USE flags
    Modify {
        /// Flag changes (e.g., +wayland -gtk +ssl)
        flags: Vec<String>,
    },
    /// Set per-package USE flags
    Package {
        /// Package atom (category/name, e.g., linux/editors/vim)
        package: String,
        /// Flag changes (e.g., +python -perl +lua)
        flags: Vec<String>,
    },
    /// Set USE_EXPAND variable (e.g., VIDEO_CARDS, INPUT_DEVICES)
    Expand {
        /// Variable name (e.g., VIDEO_CARDS)
        variable: String,
        /// Values (e.g., amdgpu intel)
        values: Vec<String>,
    },
    /// Apply a profile's USE flag defaults
    Profile {
        /// Profile name (minimal, server, desktop, developer, hardened)
        name: String,
    },
    /// Show diff from last build configuration
    Diff,
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
            println!(
                "Default path: {}",
                repository::default_repository_path().display()
            );
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
        Some(Commands::Config { action }) => {
            handle_config(action, &repo_path).await?;
        }
        Some(Commands::Use { action, flags }) => {
            handle_use(action, flags, &repo_path).await?;
        }
        Some(Commands::Mcp {
            mcp_config,
            user_mode,
        }) => {
            use buckos_mcp::{ExecutionContext, McpServer, ServerConfig};

            let pm = create_package_manager(&repo_path, global_root.as_ref()).await?;

            let config = match mcp_config {
                Some(path) => ServerConfig::load_from(&path)?,
                None => ServerConfig::default(),
            };

            // Detect execution context (root vs non-root)
            let mut context = ExecutionContext::detect();
            if user_mode {
                context.enable_user_mode();
            }

            let server = McpServer::new(pm, config, context);
            server.serve_stdio().await?;
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

/// Handle `buckos config` subcommands
async fn handle_config(action: ConfigAction, repo_path: &Path) -> Result<()> {
    let config_path = PathBuf::from("/etc/buckos/buckos.toml");
    let mut config = Config::load().unwrap_or_default();
    config.buck_repo = repo_path.to_path_buf();

    match action {
        ConfigAction::Show => {
            println!("BuckOS Configuration");
            println!("====================");
            println!("Config file: {}", config_path.display());
            println!();
            println!("[general]");
            println!("  root = {}", config.root.display());
            println!("  db_path = {}", config.db_path.display());
            println!("  cache_dir = {}", config.cache_dir.display());
            println!("  buck_repo = {}", config.buck_repo.display());
            println!("  buck_path = {}", config.buck_path.display());
            println!("  parallelism = {}", config.parallelism);
            println!();
            println!("[use_flags]");
            let mut flags: Vec<_> = config.use_flags.global.iter().collect();
            flags.sort();
            println!("  global = {:?}", flags);
            if !config.use_flags.package.is_empty() {
                println!();
                println!("  [per-package]");
                for (pkg, pkg_flags) in &config.use_flags.package {
                    let mut sorted_flags: Vec<_> = pkg_flags.iter().collect();
                    sorted_flags.sort();
                    println!("  {}/{} = {:?}", pkg.category, pkg.name, sorted_flags);
                }
            }
            println!();
            println!("[build]");
            println!("  arch = {}", config.arch);
            println!("  chost = {}", config.chost);
            println!("  cflags = {}", config.cflags);
            println!("  cxxflags = {}", config.cxxflags);
            println!("  ldflags = {}", config.ldflags);
            println!("  makeopts = {}", config.makeopts);
        }
        ConfigAction::Set { key, value } => {
            match key.as_str() {
                "arch" => config.arch = value.clone(),
                "chost" => config.chost = value.clone(),
                "cflags" => config.cflags = value.clone(),
                "cxxflags" => config.cxxflags = value.clone(),
                "ldflags" => config.ldflags = value.clone(),
                "makeopts" => config.makeopts = value.clone(),
                "parallelism" => {
                    config.parallelism = value
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid parallelism value: {}", value))?;
                }
                "use" => {
                    config.use_flags.global =
                        value.split_whitespace().map(|s| s.to_string()).collect();
                }
                other => {
                    anyhow::bail!("Unknown config key: '{}'. Valid keys: arch, chost, cflags, cxxflags, ldflags, makeopts, parallelism, use", other);
                }
            }

            // Save config
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            config.save_to(&config_path)?;
            println!("Configuration updated: {} = {}", key, value);

            // Sync to buckos-build repo
            if let Err(e) = buckos_package::buck::sync_config_to_repo(&config) {
                eprintln!("Warning: failed to sync to buckos-build: {}", e);
            } else {
                println!("Synced modifier config to {}", config.buck_repo.display());
            }
        }
    }

    Ok(())
}

/// Handle `buckos use` subcommands
async fn handle_use(action: Option<UseAction>, flags: Vec<String>, repo_path: &Path) -> Result<()> {
    use buckos_package::PackageId;

    let config_path = PathBuf::from("/etc/buckos/buckos.toml");
    let mut config = Config::load().unwrap_or_default();
    config.buck_repo = repo_path.to_path_buf();

    // If no subcommand but flags provided via trailing args, treat as modify
    let action = if action.is_none() && !flags.is_empty() {
        Some(UseAction::Modify { flags })
    } else {
        action
    };

    match action {
        None | Some(UseAction::Show) => {
            println!("Global USE flags:");
            let mut flags: Vec<_> = config.use_flags.global.iter().collect();
            flags.sort();
            for flag in &flags {
                println!("  + {}", flag);
            }
            if !config.use_flags.package.is_empty() {
                println!();
                println!("Per-package USE flags:");
                for (pkg, pkg_flags) in &config.use_flags.package {
                    let mut sorted_flags: Vec<_> = pkg_flags.iter().collect();
                    sorted_flags.sort();
                    println!("  {}/{}: {:?}", pkg.category, pkg.name, sorted_flags);
                }
            }
        }
        Some(UseAction::Set { flags }) => {
            config.use_flags.global = flags.split_whitespace().map(|s| s.to_string()).collect();
            save_and_sync(&config, &config_path)?;
            println!("Global USE flags set to: {}", flags);
        }
        Some(UseAction::Modify { flags }) => {
            for flag_str in &flags {
                if let Some(flag) = flag_str.strip_prefix('+') {
                    config.use_flags.global.insert(flag.to_string());
                    println!("  + {}", flag);
                } else if let Some(flag) = flag_str.strip_prefix('-') {
                    config.use_flags.global.remove(flag);
                    println!("  - {}", flag);
                } else {
                    // No prefix means enable
                    config.use_flags.global.insert(flag_str.to_string());
                    println!("  + {}", flag_str);
                }
            }
            save_and_sync(&config, &config_path)?;
        }
        Some(UseAction::Package { package, flags }) => {
            // Parse package atom (category/name)
            let (category, name) = if package.contains('/') {
                let parts: Vec<&str> = package.splitn(2, '/').collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("unknown".to_string(), package.clone())
            };
            let pkg_id = PackageId::new(category, &name);

            let pkg_flags = config.use_flags.package.entry(pkg_id.clone()).or_default();

            for flag_str in &flags {
                if let Some(flag) = flag_str.strip_prefix('+') {
                    pkg_flags.insert(flag.to_string());
                    println!("  {}: + {}", package, flag);
                } else if let Some(flag) = flag_str.strip_prefix('-') {
                    pkg_flags.remove(flag);
                    println!("  {}: - {}", package, flag);
                } else {
                    pkg_flags.insert(flag_str.to_string());
                    println!("  {}: + {}", package, flag_str);
                }
            }
            save_and_sync(&config, &config_path)?;
        }
        Some(UseAction::Expand { variable, values }) => {
            // USE_EXPAND variables are stored as global flags with prefix
            let prefix = variable.to_lowercase();
            // Remove existing values for this expand variable
            let to_remove: Vec<String> = config
                .use_flags
                .global
                .iter()
                .filter(|f| f.starts_with(&format!("{}_", prefix)))
                .cloned()
                .collect();
            for f in to_remove {
                config.use_flags.global.remove(&f);
            }
            // Add new values
            for value in &values {
                let flag = format!("{}_{}", prefix, value.to_lowercase());
                config.use_flags.global.insert(flag.clone());
                println!("  + {}", flag);
            }
            save_and_sync(&config, &config_path)?;
            println!("USE_EXPAND {} set to: {:?}", variable, values);
        }
        Some(UseAction::Profile { name }) => {
            let profile_flags: Vec<&str> = match name.as_str() {
                "minimal" => vec!["ipv6", "ssl", "zlib", "strip", "threads", "unicode"],
                "server" => vec![
                    "hardened", "ssl", "ipv6", "threads", "caps", "zlib", "zstd", "brotli",
                    "http2", "pam", "acl",
                ],
                "desktop" => vec![
                    "X",
                    "wayland",
                    "opengl",
                    "vulkan",
                    "dbus",
                    "pulseaudio",
                    "pipewire",
                    "alsa",
                    "ssl",
                    "ipv6",
                    "threads",
                    "unicode",
                    "nls",
                    "gtk",
                    "pam",
                    "udev",
                ],
                "developer" => vec![
                    "debug", "doc", "test", "ssl", "ipv6", "threads", "X", "dbus", "python", "lua",
                ],
                "hardened" => vec![
                    "hardened", "pie", "ssp", "seccomp", "caps", "ssl", "ipv6", "acl", "pam",
                ],
                other => {
                    anyhow::bail!(
                        "Unknown profile: '{}'. Valid profiles: minimal, server, desktop, developer, hardened",
                        other
                    );
                }
            };
            config.use_flags.global = profile_flags.iter().map(|s| s.to_string()).collect();
            save_and_sync(&config, &config_path)?;
            println!("Applied profile '{}' USE flags: {:?}", name, profile_flags);
        }
        Some(UseAction::Diff) => {
            let state_path = config.db_path.join("last_build_config.json");
            if state_path.exists() {
                let json = std::fs::read_to_string(&state_path)?;
                let old: buckos_package::UseConfig = serde_json::from_str(&json)?;

                let current = &config.use_flags.global;
                let old_flags = &old.global;

                let added: Vec<_> = current.difference(old_flags).collect();
                let removed: Vec<_> = old_flags.difference(current).collect();

                if added.is_empty() && removed.is_empty() {
                    println!("No USE flag changes since last build.");
                } else {
                    println!("USE flag changes since last build:");
                    for flag in &added {
                        println!("  + {}", flag);
                    }
                    for flag in &removed {
                        println!("  - {}", flag);
                    }
                }
            } else {
                println!("No previous build state found at {}", state_path.display());
                println!("Build state is recorded after the first successful build.");
            }
        }
    }

    Ok(())
}

/// Save config and sync modifiers to buckos-build repo
fn save_and_sync(config: &Config, config_path: &Path) -> Result<()> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    config.save_to(config_path)?;

    // Sync to buckos-build repo
    if let Err(e) = buckos_package::buck::sync_config_to_repo(config) {
        eprintln!("Warning: failed to sync to buckos-build: {}", e);
    } else {
        println!("Synced modifier config to {}", config.buck_repo.display());
    }

    Ok(())
}

/// Create a PackageManager instance with the given repository path
async fn create_package_manager(
    repo_path: &Path,
    target_root: Option<&PathBuf>,
) -> Result<PackageManager> {
    let mut config = Config::load().unwrap_or_default();

    // Set buck_repo to the detected buckos-build path
    config.buck_repo = repo_path.to_path_buf();

    // Also update the repository location for package discovery
    if let Some(repo) = config.repositories.get_mut(0) {
        repo.location = repo_path.to_path_buf();
        repo.sync_type = buckos_package::config::SyncType::Local;
    }

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
