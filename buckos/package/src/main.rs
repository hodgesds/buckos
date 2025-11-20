//! Buckos Package Manager CLI
//!
//! Command-line interface for the Buckos package manager.
//! Designed to be compatible with Gentoo's emerge command.

use buckos_package::{
    config::SyncType, overlay::{OverlayConfig, OverlayManager, OverlayQuality},
    BuildOptions, CleanOptions, Config, DepcleanOptions, EmergeOptions, InstallOptions,
    PackageManager, RemoveOptions, Resolution, SyncOptions, UpdateOptions,
};
use clap::{Args, Parser, Subcommand};
use console::style;
use dialoguer::Confirm;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::process::ExitCode;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "buckos",
    about = "Buckos Package Manager - A scalable Buck-based package manager (emerge-compatible)",
    version,
    author
)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Quiet output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Pretend mode (don't actually do anything)
    #[arg(short, long, global = true)]
    pretend: bool,

    /// Ask for confirmation before performing actions
    #[arg(short, long, global = true)]
    ask: bool,

    /// Only download packages, don't install
    #[arg(long = "fetchonly", global = true)]
    fetch_only: bool,

    /// Don't add packages to the world set
    #[arg(long = "oneshot", short = '1', global = true)]
    oneshot: bool,

    /// Update dependencies of packages too
    #[arg(long, short = 'D', global = true)]
    deep: bool,

    /// Rebuild packages with USE flag changes
    #[arg(long = "newuse", short = 'N', global = true)]
    newuse: bool,

    /// Show what packages would be built with USE flags
    #[arg(long = "tree", short = 't', global = true)]
    tree: bool,

    /// Number of parallel jobs
    #[arg(short, long, global = true)]
    jobs: Option<usize>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages (emerge-style)
    Install(InstallArgs),

    /// Remove/unmerge packages
    #[command(alias = "unmerge")]
    Remove(RemoveArgs),

    /// Update packages (@world update)
    Update(UpdateArgs),

    /// Sync package repositories (emerge --sync)
    Sync(SyncArgs),

    /// Search for packages (emerge --search)
    Search(SearchArgs),

    /// Show package information (emerge --info / equery)
    Info(InfoArgs),

    /// List installed packages
    List(ListArgs),

    /// Build a package from source
    Build(BuildArgs),

    /// Clean cache (eclean equivalent)
    Clean(CleanArgs),

    /// Verify installed packages (qcheck equivalent)
    Verify,

    /// Query package database (equery equivalent)
    Query(QueryArgs),

    /// Show package that owns a file (equery belongs)
    Owner(OwnerArgs),

    /// Show dependency tree (equery depends)
    Depgraph(DepgraphArgs),

    /// Show configuration (emerge --info)
    Config,

    /// Remove unused packages (emerge --depclean)
    Depclean(DepcleanArgs),

    /// Resume interrupted operation (emerge --resume)
    Resume,

    /// Rebuild packages with changed USE flags
    Newuse(NewuseArgs),

    /// Check for security vulnerabilities (glsa-check equivalent)
    Audit,

    /// Manage USE flags
    #[command(alias = "use")]
    Useflags(UseflagsArgs),

    /// Detect system capabilities and hardware
    Detect(DetectArgs),

    /// Generate system configuration
    Configure(ConfigureArgs),

    /// Manage package sets
    Set(SetArgs),

    /// Manage patches
    Patch(PatchArgs),

    /// Show package dependencies (shortcut for query deps)
    Deps(DepsArgs),

    /// Show reverse dependencies (shortcut for query rdeps)
    Rdeps(RdepsArgs),

    /// Manage system profiles
    Profile(ProfileArgs),

    /// Export configuration in various formats
    Export(ExportArgs),

    /// Rebuild packages with broken library dependencies (revdep-rebuild)
    Revdep(RevdepArgs),

    /// Manage package signing and verification
    Sign(SignArgs),

    /// Manage overlays (additional package repositories)
    Overlay(OverlayArgs),
}

#[derive(Args)]
struct InstallArgs {
    /// Packages to install (supports @world, @system, @selected sets)
    #[arg(required = true)]
    packages: Vec<String>,

    /// Force reinstall even if already installed
    #[arg(short, long)]
    force: bool,

    /// Don't install dependencies
    #[arg(long = "nodeps")]
    no_deps: bool,

    /// Build from source
    #[arg(short, long)]
    build: bool,

    /// USE flags to enable
    #[arg(long, value_delimiter = ',')]
    use_flags: Vec<String>,

    /// USE flags to disable
    #[arg(long = "disable-use", value_delimiter = ',')]
    disable_use_flags: Vec<String>,

    /// Only install if not already installed (skip installed)
    #[arg(long = "noreplace")]
    no_replace: bool,

    /// Empty dependency tree before installing
    #[arg(long = "emptytree", short = 'e')]
    empty_tree: bool,
}

#[derive(Args)]
struct RemoveArgs {
    /// Packages to remove
    #[arg(required = true)]
    packages: Vec<String>,

    /// Force removal even with dependents
    #[arg(short, long)]
    force: bool,

    /// Also remove unused dependencies
    #[arg(short, long)]
    recursive: bool,
}

#[derive(Args)]
struct UpdateArgs {
    /// Packages to update (all if not specified, use @world for world set)
    packages: Vec<String>,

    /// Don't sync repositories first
    #[arg(long = "nosync")]
    no_sync: bool,

    /// Only check for updates (like emerge -pvu @world)
    #[arg(short, long)]
    check: bool,

    /// Only update if newer version available (don't rebuild same version)
    #[arg(long = "update", short = 'u')]
    update_only: bool,

    /// Include deep dependencies
    #[arg(long)]
    with_bdeps: bool,
}

#[derive(Args)]
struct SyncArgs {
    /// Specific repositories to sync
    repos: Vec<String>,

    /// Sync all repositories
    #[arg(short, long)]
    all: bool,

    /// Web sync mode
    #[arg(long = "webrsync")]
    webrsync: bool,
}

#[derive(Args)]
struct DepcleanArgs {
    /// Specific packages to depclean
    packages: Vec<String>,

    /// Only show what would be removed
    #[arg(long)]
    pretend: bool,

    /// Remove all packages not in world or system
    #[arg(short, long)]
    all: bool,
}

#[derive(Args)]
struct NewuseArgs {
    /// Packages to check for USE flag changes (all if not specified)
    packages: Vec<String>,

    /// Include deep dependencies
    #[arg(short = 'D', long)]
    deep: bool,
}

#[derive(Args)]
struct SearchArgs {
    /// Search query
    query: String,
}

#[derive(Args)]
struct InfoArgs {
    /// Package name
    package: String,
}

#[derive(Args)]
struct ListArgs {
    /// Show only explicitly installed packages
    #[arg(short, long)]
    explicit: bool,

    /// Show package sizes
    #[arg(short, long)]
    size: bool,
}

#[derive(Args)]
struct BuildArgs {
    /// Buck target to build
    target: String,

    /// Number of parallel jobs
    #[arg(short, long)]
    jobs: Option<usize>,

    /// Build in release mode
    #[arg(short, long)]
    release: bool,

    /// Additional Buck arguments
    #[arg(last = true)]
    buck_args: Vec<String>,
}

#[derive(Args)]
struct CleanArgs {
    /// Clean everything
    #[arg(short, long)]
    all: bool,

    /// Clean only downloads
    #[arg(short, long)]
    downloads: bool,

    /// Clean only builds
    #[arg(short, long)]
    builds: bool,
}

#[derive(Args)]
struct QueryArgs {
    /// Query type
    #[command(subcommand)]
    query_type: QueryType,
}

#[derive(Subcommand)]
enum QueryType {
    /// List files owned by package
    Files { package: String },
    /// List dependencies
    Deps { package: String },
    /// List reverse dependencies
    Rdeps { package: String },
}

#[derive(Args)]
struct OwnerArgs {
    /// File path to query
    path: String,
}

#[derive(Args)]
struct DepgraphArgs {
    /// Package to show dependencies for
    package: String,

    /// Maximum depth
    #[arg(short, long, default_value = "5")]
    depth: usize,
}

#[derive(Args)]
struct UseflagsArgs {
    /// USE flags subcommand
    #[command(subcommand)]
    subcommand: UseflagsCommand,
}

#[derive(Subcommand)]
enum UseflagsCommand {
    /// List available USE flags
    List {
        /// Filter by category (e.g., network, security, graphics)
        #[arg(short, long)]
        category: Option<String>,
        /// Show only global flags
        #[arg(short, long)]
        global: bool,
        /// Show detailed descriptions
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show information about a specific USE flag
    Info {
        /// The USE flag to query
        flag: String,
    },
    /// Set global USE flags
    Set {
        /// USE flags to set (prefix with - to disable)
        #[arg(required = true)]
        flags: Vec<String>,
    },
    /// Get current USE flag configuration
    Get {
        /// Output format (text, json, toml)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Set USE flags for a specific package
    Package {
        /// Package name (e.g., dev-libs/openssl)
        package: String,
        /// USE flags for this package
        #[arg(required = true)]
        flags: Vec<String>,
    },
    /// Show USE_EXPAND variables (CPU_FLAGS, VIDEO_CARDS, etc.)
    Expand {
        /// Specific variable to show
        variable: Option<String>,
    },
    /// Validate USE flag configuration
    Validate,
}

#[derive(Args)]
struct DetectArgs {
    /// Output format (text, json, toml, shell)
    #[arg(short, long, default_value = "text")]
    format: String,
    /// Detect CPU features
    #[arg(long)]
    cpu: bool,
    /// Detect GPU/video hardware
    #[arg(long)]
    gpu: bool,
    /// Detect audio hardware
    #[arg(long)]
    audio: bool,
    /// Detect network capabilities
    #[arg(long)]
    network: bool,
    /// Detect all hardware (default)
    #[arg(short, long)]
    all: bool,
    /// Output to file instead of stdout
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Args)]
struct ConfigureArgs {
    /// Profile to use (minimal, server, desktop, developer, hardened)
    #[arg(short, long, default_value = "default")]
    profile: String,
    /// USE flags to enable/disable
    #[arg(long = "use", value_delimiter = ' ')]
    use_flags: Vec<String>,
    /// Target architecture
    #[arg(long, default_value = "x86_64")]
    arch: String,
    /// Output file path
    #[arg(short, long)]
    output: Option<String>,
    /// Output format (bzl, json, toml, shell)
    #[arg(short, long, default_value = "bzl")]
    format: String,
    /// Auto-detect hardware and add appropriate flags
    #[arg(long)]
    auto_detect: bool,
}

#[derive(Args)]
struct SetArgs {
    /// Set subcommand
    #[command(subcommand)]
    subcommand: SetCommand,
}

#[derive(Subcommand)]
enum SetCommand {
    /// List available package sets
    List {
        /// Filter by set type (system, task, desktop)
        #[arg(short, long)]
        r#type: Option<String>,
    },
    /// Show contents of a package set
    Show {
        /// Set name
        set_name: String,
    },
    /// Install all packages in a set
    Install {
        /// Set name
        set_name: String,
    },
    /// Compare two package sets
    Compare {
        /// First set name
        set1: String,
        /// Second set name
        set2: String,
    },
}

#[derive(Args)]
struct PatchArgs {
    /// Patch subcommand
    #[command(subcommand)]
    subcommand: PatchCommand,
}

#[derive(Subcommand)]
enum PatchCommand {
    /// List patches for a package
    List {
        /// Package name
        package: String,
    },
    /// Show patch information
    Info {
        /// Package name
        package: String,
        /// Patch name
        patch_name: String,
    },
    /// Add a user patch
    Add {
        /// Package name
        package: String,
        /// Path to patch file
        patch_file: String,
    },
    /// Remove a user patch
    Remove {
        /// Package name
        package: String,
        /// Patch name
        patch_name: String,
    },
    /// Check if patches apply cleanly
    Check {
        /// Package name
        package: String,
    },
    /// Show patch application order
    Order {
        /// Package name
        package: String,
    },
}

#[derive(Args)]
struct DepsArgs {
    /// Package name
    package: String,
    /// Show as tree
    #[arg(short, long)]
    tree: bool,
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
}

#[derive(Args)]
struct RdepsArgs {
    /// Package name
    package: String,
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
}

#[derive(Args)]
struct ProfileArgs {
    /// Profile subcommand
    #[command(subcommand)]
    subcommand: ProfileCommand,
}

#[derive(Subcommand)]
enum ProfileCommand {
    /// List available profiles
    List,
    /// Show profile information
    Show {
        /// Profile name
        profile: String,
    },
    /// Set the active profile
    Set {
        /// Profile name
        profile: String,
    },
    /// Show current profile
    Current,
}

#[derive(Args)]
struct ExportArgs {
    /// Output format (json, toml, shell, buck)
    #[arg(short, long, default_value = "json")]
    format: String,
    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<String>,
    /// Include package list
    #[arg(long)]
    with_packages: bool,
}

#[derive(Args)]
struct RevdepArgs {
    /// Only show packages that would be rebuilt (don't actually rebuild)
    #[arg(short, long)]
    pretend: bool,
    /// Library path to check (default: system library paths)
    #[arg(short, long)]
    library: Option<String>,
    /// Specific packages to check
    packages: Vec<String>,
    /// Ignore specific packages during rebuild
    #[arg(long, value_delimiter = ',')]
    ignore: Vec<String>,
}

#[derive(Args)]
struct SignArgs {
    /// Signing subcommand
    #[command(subcommand)]
    subcommand: SignCommand,
}

#[derive(Subcommand)]
enum SignCommand {
    /// List available signing keys
    ListKeys {
        /// Show only secret keys
        #[arg(short, long)]
        secret: bool,
    },
    /// Import a signing key
    ImportKey {
        /// Key source (file path, URL, or key ID)
        source: String,
    },
    /// Export a signing key
    ExportKey {
        /// Key ID or fingerprint
        key_id: String,
        /// Output file
        output: String,
        /// ASCII armor output
        #[arg(short, long)]
        armor: bool,
    },
    /// Sign a package manifest
    SignManifest {
        /// Package directory
        package_dir: String,
        /// Key ID to use (defaults to default key)
        #[arg(short, long)]
        key: Option<String>,
    },
    /// Verify a package manifest signature
    VerifyManifest {
        /// Path to Manifest file
        manifest: String,
    },
    /// Sign a repository
    SignRepo {
        /// Repository directory
        repo_dir: String,
        /// Key ID to use
        #[arg(short, long)]
        key: Option<String>,
    },
    /// Verify a repository signature
    VerifyRepo {
        /// Repository directory
        repo_dir: String,
    },
    /// Sign a file
    SignFile {
        /// File to sign
        file: String,
        /// Key ID to use
        #[arg(short, long)]
        key: Option<String>,
    },
    /// Verify a file signature
    VerifyFile {
        /// File to verify
        file: String,
        /// Signature file (defaults to file.asc)
        #[arg(short, long)]
        signature: Option<String>,
    },
    /// Show key information
    KeyInfo {
        /// Key ID or fingerprint
        key_id: String,
    },
    /// Set trust level for a key
    SetTrust {
        /// Key ID or fingerprint
        key_id: String,
        /// Trust level (unknown, never, marginal, full, ultimate)
        trust: String,
    },
}

#[derive(Args)]
struct OverlayArgs {
    /// Overlay subcommand
    #[command(subcommand)]
    subcommand: OverlayCommand,
}

#[derive(Subcommand)]
enum OverlayCommand {
    /// List overlays
    List {
        /// Show only enabled overlays
        #[arg(short, long)]
        enabled: bool,
        /// Show all available overlays (including disabled)
        #[arg(short, long)]
        all: bool,
    },
    /// Add a new overlay
    Add {
        /// Overlay name
        name: String,
        /// Sync URI (git URL, rsync path, or http URL)
        #[arg(short, long)]
        uri: Option<String>,
        /// Sync type (git, rsync, http, local)
        #[arg(short = 't', long, default_value = "git")]
        sync_type: String,
        /// Priority (higher = preferred)
        #[arg(short, long, default_value = "50")]
        priority: i32,
        /// Local path (for local overlays)
        #[arg(short, long)]
        location: Option<String>,
    },
    /// Remove an overlay
    Remove {
        /// Overlay name
        name: String,
        /// Delete overlay files
        #[arg(short, long)]
        delete: bool,
    },
    /// Enable an overlay
    Enable {
        /// Overlay name
        name: String,
    },
    /// Disable an overlay
    Disable {
        /// Overlay name
        name: String,
    },
    /// Sync an overlay
    Sync {
        /// Overlay name (all if not specified)
        name: Option<String>,
    },
    /// Show overlay information
    Info {
        /// Overlay name
        name: String,
    },
    /// Set overlay priority
    Priority {
        /// Overlay name
        name: String,
        /// New priority
        priority: i32,
    },
    /// Search for overlays
    Search {
        /// Search query
        query: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Initialize logging
    let filter = match cli.verbose {
        0 if cli.quiet => "error",
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .with_target(false)
        .init();

    // Load configuration
    let config = match cli.config {
        Some(path) => match Config::load_from(std::path::Path::new(&path)) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load config: {}", e);
                return ExitCode::FAILURE;
            }
        },
        None => Config::default(),
    };

    // Create package manager
    let pkg_manager = match PackageManager::new(config).await {
        Ok(pm) => pm,
        Err(e) => {
            error!("Failed to initialize package manager: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Build global emerge options
    let emerge_opts = EmergeOptions {
        pretend: cli.pretend,
        ask: cli.ask,
        fetch_only: cli.fetch_only,
        oneshot: cli.oneshot,
        deep: cli.deep,
        newuse: cli.newuse,
        tree: cli.tree,
        verbose: cli.verbose,
        quiet: cli.quiet,
        jobs: cli.jobs,
        ..Default::default()
    };

    // Execute command
    let result = match cli.command {
        Commands::Install(args) => cmd_install(&pkg_manager, args, &emerge_opts).await,
        Commands::Remove(args) => cmd_remove(&pkg_manager, args, &emerge_opts).await,
        Commands::Update(args) => cmd_update(&pkg_manager, args, &emerge_opts).await,
        Commands::Sync(args) => cmd_sync(&pkg_manager, args).await,
        Commands::Search(args) => cmd_search(&pkg_manager, args).await,
        Commands::Info(args) => cmd_info(&pkg_manager, args).await,
        Commands::List(args) => cmd_list(&pkg_manager, args).await,
        Commands::Build(args) => cmd_build(&pkg_manager, args).await,
        Commands::Clean(args) => cmd_clean(&pkg_manager, args).await,
        Commands::Verify => cmd_verify(&pkg_manager).await,
        Commands::Query(args) => cmd_query(&pkg_manager, args).await,
        Commands::Owner(args) => cmd_owner(&pkg_manager, args).await,
        Commands::Depgraph(args) => cmd_depgraph(&pkg_manager, args).await,
        Commands::Config => cmd_config().await,
        Commands::Depclean(args) => cmd_depclean(&pkg_manager, args, &emerge_opts).await,
        Commands::Resume => cmd_resume(&pkg_manager).await,
        Commands::Newuse(args) => cmd_newuse(&pkg_manager, args, &emerge_opts).await,
        Commands::Audit => cmd_audit(&pkg_manager).await,
        Commands::Useflags(args) => cmd_useflags(&pkg_manager, args).await,
        Commands::Detect(args) => cmd_detect(args).await,
        Commands::Configure(args) => cmd_configure(args).await,
        Commands::Set(args) => cmd_set(&pkg_manager, args, &emerge_opts).await,
        Commands::Patch(args) => cmd_patch(args).await,
        Commands::Deps(args) => cmd_deps(&pkg_manager, args).await,
        Commands::Rdeps(args) => cmd_rdeps(&pkg_manager, args).await,
        Commands::Profile(args) => cmd_profile(args).await,
        Commands::Export(args) => cmd_export(args).await,
        Commands::Revdep(args) => cmd_revdep(&pkg_manager, args, &emerge_opts).await,
        Commands::Sign(args) => cmd_sign(args).await,
        Commands::Overlay(args) => cmd_overlay(args).await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{}", e);
            ExitCode::FAILURE
        }
    }
}

async fn cmd_install(
    pm: &PackageManager,
    args: InstallArgs,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    // Expand package sets
    let packages = expand_package_sets(pm, &args.packages).await?;

    let opts = InstallOptions {
        force: args.force,
        no_deps: args.no_deps,
        build: args.build,
        use_flags: args.use_flags,
        oneshot: emerge_opts.oneshot,
        fetch_only: emerge_opts.fetch_only,
        deep: emerge_opts.deep,
        newuse: emerge_opts.newuse,
        empty_tree: args.empty_tree,
        no_replace: args.no_replace,
        use_pkg: emerge_opts.use_pkg,
        use_pkg_only: emerge_opts.use_pkg_only,
        get_binpkg: emerge_opts.get_binpkg,
        get_binpkg_only: emerge_opts.get_binpkg_only,
        build_pkg: emerge_opts.build_pkg,
        build_pkg_only: emerge_opts.build_pkg_only,
    };

    // Resolve dependencies first to show what will be installed
    let resolution = pm.resolve_packages(&packages, &opts).await?;

    if resolution.packages.is_empty() {
        if !emerge_opts.quiet {
            println!("\n{}", style(">>> No packages to install").green().bold());
        }
        return Ok(());
    }

    // Display emerge-style package list
    print_emerge_list(&resolution, emerge_opts, "install")?;

    // Pretend mode - just show what would be done
    if emerge_opts.pretend {
        return Ok(());
    }

    // Ask mode - prompt for confirmation
    if emerge_opts.ask {
        if !Confirm::new()
            .with_prompt("Would you like to merge these packages?")
            .default(true)
            .interact()?
        {
            println!("{}", style(">>> Exiting.").yellow().bold());
            return Ok(());
        }
        println!();
    }

    // Actually install
    pm.install(&packages, opts).await?;

    println!(
        "\n{} {} packages installed",
        style(">>>").green().bold(),
        resolution.packages.len()
    );

    Ok(())
}

async fn cmd_remove(
    pm: &PackageManager,
    args: RemoveArgs,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    // Expand package sets
    let packages = expand_package_sets(pm, &args.packages).await?;

    let opts = RemoveOptions {
        force: args.force,
        recursive: args.recursive,
    };

    // Get packages that would be removed
    let to_remove = pm.get_removal_list(&packages, &opts).await?;

    if to_remove.is_empty() {
        println!("{} No packages to unmerge", style(">>>").yellow().bold());
        return Ok(());
    }

    // Display unmerge list
    println!(
        "\n{} These are the packages that would be unmerged:\n",
        style(">>>").red().bold()
    );

    for pkg in &to_remove {
        println!(
            "  {} {}/{}",
            style("R").red().bold(),
            style(&pkg.id.category).cyan(),
            style(format!("{}-{}", &pkg.name, &pkg.version)).red()
        );
    }

    println!(
        "\n>>> Unmerging {} package(s)...",
        style(to_remove.len()).bold()
    );

    // Pretend mode
    if emerge_opts.pretend {
        return Ok(());
    }

    // Ask mode
    if emerge_opts.ask {
        if !Confirm::new()
            .with_prompt("Would you like to unmerge these packages?")
            .default(false)
            .interact()?
        {
            println!("{}", style(">>> Exiting.").yellow().bold());
            return Ok(());
        }
        println!();
    }

    pm.remove(&packages, opts).await?;

    println!(
        "{} {} packages unmerged",
        style(">>>").green().bold(),
        to_remove.len()
    );

    Ok(())
}

async fn cmd_update(
    pm: &PackageManager,
    args: UpdateArgs,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    // Expand package sets (default to @world if nothing specified)
    let packages = if args.packages.is_empty() {
        vec!["@world".to_string()]
    } else {
        args.packages.clone()
    };
    let expanded = expand_package_sets(pm, &packages).await?;

    let opts = UpdateOptions {
        sync: !args.no_sync,
        check_only: args.check || emerge_opts.pretend,
        deep: emerge_opts.deep,
        newuse: emerge_opts.newuse,
        with_bdeps: args.with_bdeps,
    };

    // Sync first if requested
    if opts.sync && !emerge_opts.quiet {
        println!("{} Syncing repositories...", style(">>>").blue().bold());
        pm.sync().await?;
    }

    if !emerge_opts.quiet {
        println!("{} Calculating dependencies...", style(">>>").blue().bold());
    }

    let packages_slice = if expanded.is_empty() {
        None
    } else {
        Some(expanded.as_slice())
    };

    // Get update resolution
    let resolution = pm.get_update_resolution(packages_slice, &opts).await?;

    if resolution.packages.is_empty() {
        if !emerge_opts.quiet {
            println!("\n{} @world set is up-to-date", style(">>>").green().bold());
        }
        return Ok(());
    }

    // Display emerge-style list
    print_emerge_list(&resolution, emerge_opts, "update")?;

    // Pretend or check mode
    if emerge_opts.pretend || args.check {
        return Ok(());
    }

    // Ask mode
    if emerge_opts.ask {
        if !Confirm::new()
            .with_prompt("Would you like to merge these packages?")
            .default(true)
            .interact()?
        {
            println!("{}", style(">>> Exiting.").yellow().bold());
            return Ok(());
        }
        println!();
    }

    pm.update(packages_slice, opts).await?;

    println!(
        "\n{} {} packages updated",
        style(">>>").green().bold(),
        resolution.packages.len()
    );

    Ok(())
}

async fn cmd_sync(pm: &PackageManager, args: SyncArgs) -> buckos_package::Result<()> {
    if args.repos.is_empty() || args.all {
        println!("{} Syncing all repositories...", style(">>>").blue().bold());
        pm.sync().await?;
    } else {
        for repo in &args.repos {
            println!(
                "{} Syncing repository: {}...",
                style(">>>").blue().bold(),
                repo
            );
            pm.sync_repo(repo).await?;
        }
    }
    println!("{} Sync complete", style(">>>").green().bold());
    Ok(())
}

async fn cmd_search(pm: &PackageManager, args: SearchArgs) -> buckos_package::Result<()> {
    let results = pm.search(&args.query).await?;

    if results.is_empty() {
        println!("No packages found matching '{}'", args.query);
        return Ok(());
    }

    println!("Found {} packages:\n", results.len());

    for pkg in results {
        println!(
            "{}/{} {}",
            style(&pkg.id.category).cyan(),
            style(&pkg.id.name).green().bold(),
            style(&pkg.version.to_string()).yellow()
        );
        println!("    {}", pkg.description);
    }

    Ok(())
}

async fn cmd_info(pm: &PackageManager, args: InfoArgs) -> buckos_package::Result<()> {
    match pm.info(&args.package).await? {
        Some(pkg) => {
            println!("{}", style("Package Information").bold().underlined());
            println!();
            println!(
                "  {}: {}/{}",
                style("Name").bold(),
                pkg.id.category,
                pkg.id.name
            );
            println!("  {}: {}", style("Version").bold(), pkg.version);
            println!("  {}: {}", style("Slot").bold(), pkg.slot);
            println!("  {}: {}", style("License").bold(), pkg.license);
            if let Some(homepage) = &pkg.homepage {
                println!("  {}: {}", style("Homepage").bold(), homepage);
            }
            println!("  {}: {}", style("Description").bold(), pkg.description);

            if !pkg.use_flags.is_empty() {
                println!("  {}:", style("USE flags").bold());
                for flag in &pkg.use_flags {
                    println!("    {} - {}", style(&flag.name).cyan(), flag.description);
                }
            }

            if !pkg.dependencies.is_empty() {
                println!("  {}:", style("Dependencies").bold());
                for dep in &pkg.dependencies {
                    println!("    {}", dep.package);
                }
            }

            println!(
                "  {}: {}",
                style("Size").bold(),
                format_size(pkg.installed_size)
            );
        }
        None => {
            println!("Package '{}' not found", args.package);
        }
    }

    Ok(())
}

async fn cmd_list(pm: &PackageManager, args: ListArgs) -> buckos_package::Result<()> {
    let packages = pm.list_installed().await?;

    let filtered: Vec<_> = if args.explicit {
        packages.into_iter().filter(|p| p.explicit).collect()
    } else {
        packages
    };

    if filtered.is_empty() {
        println!("No packages installed");
        return Ok(());
    }

    println!("Installed packages ({}):\n", filtered.len());

    for pkg in filtered {
        if args.size {
            println!(
                "{}/{} {} [{}]",
                style(&pkg.id.category).cyan(),
                style(&pkg.name).green(),
                style(&pkg.version.to_string()).yellow(),
                format_size(pkg.size)
            );
        } else {
            println!(
                "{}/{} {}",
                style(&pkg.id.category).cyan(),
                style(&pkg.name).green(),
                style(&pkg.version.to_string()).yellow()
            );
        }
    }

    Ok(())
}

async fn cmd_build(pm: &PackageManager, args: BuildArgs) -> buckos_package::Result<()> {
    println!(
        "{} Building target: {}",
        style(">>>").blue().bold(),
        args.target
    );

    let opts = BuildOptions {
        jobs: args.jobs,
        release: args.release,
        buck_args: args.buck_args,
        config_options: None,
    };

    let result = pm.build(&args.target, opts).await?;

    if result.success {
        println!(
            "{} Build successful in {:?}",
            style(">>>").green().bold(),
            result.duration
        );
        if let Some(path) = result.output_path {
            println!("  Output: {}", path.display());
        }
    } else {
        println!("{} Build failed", style(">>>").red().bold());
        if !result.stderr.is_empty() {
            eprintln!("{}", result.stderr);
        }
    }

    Ok(())
}

async fn cmd_clean(pm: &PackageManager, args: CleanArgs) -> buckos_package::Result<()> {
    let opts = CleanOptions {
        all: args.all,
        downloads: args.downloads,
        builds: args.builds,
    };

    pm.clean(opts).await?;
    println!("{} Cache cleaned", style(">>>").green().bold());

    Ok(())
}

async fn cmd_verify(pm: &PackageManager) -> buckos_package::Result<()> {
    println!(
        "{} Verifying installed packages...",
        style(">>>").blue().bold()
    );

    let results = pm.verify().await?;

    let mut all_ok = true;
    for result in &results {
        if !result.ok {
            all_ok = false;
            println!(
                "{}: {}",
                style(&result.package).red().bold(),
                if !result.missing.is_empty() {
                    format!("{} missing files", result.missing.len())
                } else {
                    format!("{} modified files", result.modified.len())
                }
            );
        }
    }

    if all_ok {
        println!(
            "{} All {} packages verified successfully",
            style(">>>").green().bold(),
            results.len()
        );
    } else {
        println!("{} Verification found issues", style(">>>").yellow().bold());
    }

    Ok(())
}

async fn cmd_query(pm: &PackageManager, args: QueryArgs) -> buckos_package::Result<()> {
    match args.query_type {
        QueryType::Files { package } => {
            let installed = pm.list_installed().await?;
            if let Some(pkg) = installed.iter().find(|p| p.name == package) {
                println!("Files owned by {}:\n", package);
                for file in &pkg.files {
                    println!("  {}", file.path);
                }
            } else {
                println!("Package '{}' not installed", package);
            }
        }
        QueryType::Deps { package } => {
            if let Some(pkg) = pm.info(&package).await? {
                println!("Dependencies of {}:\n", package);
                for dep in &pkg.dependencies {
                    println!("  {}", dep.package);
                }
                for dep in &pkg.runtime_dependencies {
                    println!("  {} (runtime)", dep.package);
                }
            } else {
                println!("Package '{}' not found", package);
            }
        }
        QueryType::Rdeps { package } => {
            let rdeps = pm.get_reverse_dependencies(&package).await?;
            if rdeps.is_empty() {
                println!("No packages depend on '{}'", package);
            } else {
                println!("Packages that depend on {}:\n", package);
                for rdep in rdeps {
                    println!("  {}", rdep);
                }
            }
        }
    }

    Ok(())
}

async fn cmd_owner(pm: &PackageManager, args: OwnerArgs) -> buckos_package::Result<()> {
    println!(
        "{} Searching for owner of: {}",
        style(">>>").blue().bold(),
        args.path
    );

    // First try exact match
    if let Some(result) = pm.find_file_owner(&args.path).await? {
        println!(
            "\n{}/{} {} owns {}",
            style(&result.package.category).cyan(),
            style(&result.package.name).green().bold(),
            style(format!("({})", result.version)).yellow(),
            result.file_path
        );
        return Ok(());
    }

    // If no exact match, try pattern search
    let results = pm.find_file_owners_by_pattern(&args.path).await?;

    if results.is_empty() {
        println!(
            "{} No package owns '{}'",
            style(">>>").yellow().bold(),
            args.path
        );
    } else {
        println!("\nFound {} matching file(s):\n", results.len());
        for result in results {
            println!(
                "  {} {}/{} {}",
                result.file_path,
                style(&result.package.category).cyan(),
                style(&result.package.name).green().bold(),
                style(format!("({})", result.version)).yellow()
            );
        }
    }

    Ok(())
}

async fn cmd_depgraph(pm: &PackageManager, args: DepgraphArgs) -> buckos_package::Result<()> {
    if let Some(pkg) = pm.info(&args.package).await? {
        println!("Dependency graph for {}:\n", args.package);
        print_deps(
            &pkg.dependencies
                .iter()
                .map(|d| d.package.to_string())
                .collect::<Vec<_>>(),
            0,
            args.depth,
        );
    } else {
        println!("Package '{}' not found", args.package);
    }
    Ok(())
}

fn print_deps(deps: &[String], level: usize, max_depth: usize) {
    if level >= max_depth {
        return;
    }

    for dep in deps {
        let indent = "  ".repeat(level);
        println!("{}|- {}", indent, dep);
    }
}

async fn cmd_config() -> buckos_package::Result<()> {
    let config = Config::default();

    println!("{}", style("Current Configuration").bold().underlined());
    println!();
    println!("  Root: {}", config.root.display());
    println!("  DB Path: {}", config.db_path.display());
    println!("  Cache Dir: {}", config.cache_dir.display());
    println!("  Buck Path: {}", config.buck_path.display());
    println!("  Parallelism: {}", config.parallelism);
    println!("  Architecture: {}", config.arch);
    println!("  CHOST: {}", config.chost);
    println!("  CFLAGS: {}", config.cflags);
    println!("  MAKEOPTS: {}", config.makeopts);

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Expand package sets (@world, @system, @selected) to individual packages
async fn expand_package_sets(
    pm: &PackageManager,
    packages: &[String],
) -> buckos_package::Result<Vec<String>> {
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
                "@installed" => {
                    let installed = pm.list_installed().await?;
                    result.extend(installed.iter().map(|p| p.id.full_name()));
                }
                _ => {
                    // Unknown set, treat as literal
                    result.push(pkg.clone());
                }
            }
        } else {
            result.push(pkg.clone());
        }
    }

    Ok(result)
}

/// Print emerge-style package list with colors and USE flags
fn print_emerge_list(
    resolution: &Resolution,
    opts: &EmergeOptions,
    action: &str,
) -> buckos_package::Result<()> {
    println!(
        "\n{} These are the packages that would be {}:\n",
        style(">>>").green().bold(),
        match action {
            "install" => "merged",
            "update" => "merged",
            "rebuild" => "rebuilt",
            _ => action,
        }
    );

    // Calculate counts
    let new_count = resolution.packages.iter().filter(|p| !p.is_upgrade).count();
    let update_count = resolution.packages.iter().filter(|p| p.is_upgrade).count();
    let rebuild_count = resolution.packages.iter().filter(|p| p.is_rebuild).count();

    for (idx, pkg) in resolution.packages.iter().enumerate() {
        // Determine status marker
        let marker = if pkg.is_rebuild {
            style("R").yellow().bold() // Rebuild
        } else if pkg.is_upgrade {
            style("U").blue().bold() // Update
        } else {
            style("N").green().bold() // New
        };

        // Build slot string
        let slot = if pkg.slot != "0" {
            format!(":{}", pkg.slot)
        } else {
            String::new()
        };

        // Print package line
        print!(
            "[{:>3}] {} {}/{}",
            idx + 1,
            marker,
            style(&pkg.id.category).cyan(),
            style(format!("{}-{}{}", &pkg.id.name, &pkg.version, slot)).bold()
        );

        // Show USE flags if verbose or tree mode
        if opts.verbose > 0 || opts.tree {
            if !pkg.use_flags.is_empty() {
                print!(" USE=\"");
                for (i, flag) in pkg.use_flags.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    if flag.enabled {
                        print!("{}", style(&flag.name).green());
                    } else {
                        print!("{}", style(format!("-{}", flag.name)).red());
                    }
                }
                print!("\"");
            }
        }

        // Show size if verbose
        if opts.verbose > 1 {
            print!(" [{}]", format_size(pkg.installed_size));
        }

        println!();

        // Show tree if requested
        if opts.tree && !pkg.dependencies.is_empty() {
            for dep in &pkg.dependencies {
                println!("      └── {}", dep.package);
            }
        }
    }

    // Print summary
    println!();
    println!(
        "Total: {} package(s)",
        style(resolution.packages.len()).bold()
    );
    if new_count > 0 {
        print!("{} new, ", style(new_count).green());
    }
    if update_count > 0 {
        print!("{} updates, ", style(update_count).blue());
    }
    if rebuild_count > 0 {
        print!("{} rebuilds, ", style(rebuild_count).yellow());
    }
    println!();

    // Size totals
    println!(
        "Download size: {}",
        style(format_size(resolution.download_size)).cyan()
    );
    println!(
        "Space required: {}",
        style(format_size(resolution.install_size)).cyan()
    );

    Ok(())
}

/// Depclean command - remove unused packages
async fn cmd_depclean(
    pm: &PackageManager,
    args: DepcleanArgs,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    println!("{} Calculating dependencies...", style(">>>").blue().bold());

    let opts = DepcleanOptions {
        packages: args.packages.clone(),
        pretend: args.pretend || emerge_opts.pretend,
    };

    let to_remove = pm.calculate_depclean(&opts).await?;

    if to_remove.is_empty() {
        println!("{} No packages to depclean", style(">>>").green().bold());
        return Ok(());
    }

    // Display packages to remove
    println!(
        "\n{} These are the packages that would be unmerged:\n",
        style(">>>").red().bold()
    );

    let mut total_size = 0u64;
    for pkg in &to_remove {
        println!(
            "  {} {}/{}",
            style("D").red().bold(),
            style(&pkg.id.category).cyan(),
            style(format!("{}-{}", &pkg.name, &pkg.version)).red()
        );
        total_size += pkg.size;
    }

    println!(
        "\n>>> {} package(s) selected for depclean",
        style(to_remove.len()).bold()
    );
    println!(">>> Space freed: {}", style(format_size(total_size)).cyan());

    // Pretend mode
    if opts.pretend || emerge_opts.pretend {
        return Ok(());
    }

    // Ask mode
    if emerge_opts.ask {
        if !Confirm::new()
            .with_prompt("Would you like to unmerge these packages?")
            .default(false)
            .interact()?
        {
            println!("{}", style(">>> Exiting.").yellow().bold());
            return Ok(());
        }
        println!();
    }

    // Actually remove
    pm.depclean(&opts).await?;

    println!(
        "{} {} packages unmerged",
        style(">>>").green().bold(),
        to_remove.len()
    );

    Ok(())
}

/// Resume interrupted operation
async fn cmd_resume(pm: &PackageManager) -> buckos_package::Result<()> {
    println!("{} Resuming last operation...", style(">>>").blue().bold());

    if pm.resume().await? {
        println!("{} Resume complete", style(">>>").green().bold());
    } else {
        println!(
            "{} No interrupted operation to resume",
            style(">>>").yellow().bold()
        );
    }

    Ok(())
}

/// Rebuild packages with changed USE flags
async fn cmd_newuse(
    pm: &PackageManager,
    args: NewuseArgs,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    println!(
        "{} Checking for USE flag changes...",
        style(">>>").blue().bold()
    );

    let packages = if args.packages.is_empty() {
        None
    } else {
        Some(args.packages.as_slice())
    };

    let to_rebuild = pm.find_newuse_packages(packages, args.deep).await?;

    if to_rebuild.is_empty() {
        println!(
            "{} No packages need rebuilding",
            style(">>>").green().bold()
        );
        return Ok(());
    }

    // Display packages to rebuild
    println!(
        "\n{} These packages have USE flag changes:\n",
        style(">>>").yellow().bold()
    );

    for pkg in &to_rebuild {
        println!(
            "  {} {}/{}",
            style("R").yellow().bold(),
            style(&pkg.id.category).cyan(),
            style(format!("{}-{}", &pkg.name, &pkg.version)).yellow()
        );

        // Show USE flag changes
        if !pkg.use_changes.is_empty() {
            print!("      USE: ");
            for (i, change) in pkg.use_changes.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                if change.added {
                    print!("{}", style(format!("+{}", change.flag)).green());
                } else {
                    print!("{}", style(format!("-{}", change.flag)).red());
                }
            }
            println!();
        }
    }

    println!(
        "\n>>> {} package(s) with USE flag changes",
        style(to_rebuild.len()).bold()
    );

    // Pretend mode
    if emerge_opts.pretend {
        return Ok(());
    }

    // Ask mode
    if emerge_opts.ask {
        if !Confirm::new()
            .with_prompt("Would you like to rebuild these packages?")
            .default(true)
            .interact()?
        {
            println!("{}", style(">>> Exiting.").yellow().bold());
            return Ok(());
        }
        println!();
    }

    // Rebuild packages
    let pkg_names: Vec<String> = to_rebuild.iter().map(|p| p.id.full_name()).collect();

    let opts = InstallOptions {
        force: true,
        ..Default::default()
    };

    pm.install(&pkg_names, opts).await?;

    println!(
        "{} {} packages rebuilt",
        style(">>>").green().bold(),
        to_rebuild.len()
    );

    Ok(())
}

/// Audit for security vulnerabilities
async fn cmd_audit(pm: &PackageManager) -> buckos_package::Result<()> {
    println!(
        "{} Checking for security vulnerabilities...",
        style(">>>").blue().bold()
    );

    let vulnerabilities = pm.audit().await?;

    if vulnerabilities.is_empty() {
        println!(
            "{} No known vulnerabilities found",
            style(">>>").green().bold()
        );
        return Ok(());
    }

    println!(
        "\n{} Found {} security issue(s):\n",
        style(">>>").red().bold(),
        vulnerabilities.len()
    );

    for vuln in &vulnerabilities {
        println!(
            "  {} {}/{} - {}",
            match vuln.severity.as_str() {
                "critical" => style("!").red().bold(),
                "high" => style("!").red(),
                "medium" => style("*").yellow(),
                _ => style("*").white(),
            },
            style(&vuln.package.category).cyan(),
            style(&vuln.package.name).bold(),
            vuln.id
        );
        if !vuln.title.is_empty() {
            println!("    {}", vuln.title);
        }
    }

    println!(
        "\n>>> Run '{} install <package>' to update affected packages",
        style("buckos").bold()
    );

    Ok(())
}

/// USE flags management command
async fn cmd_useflags(_pm: &PackageManager, args: UseflagsArgs) -> buckos_package::Result<()> {
    match args.subcommand {
        UseflagsCommand::List {
            category,
            global,
            verbose,
        } => cmd_useflags_list(category, global, verbose).await,
        UseflagsCommand::Info { flag } => cmd_useflags_info(&flag).await,
        UseflagsCommand::Set { flags } => cmd_useflags_set(&flags).await,
        UseflagsCommand::Get { format } => cmd_useflags_get(&format).await,
        UseflagsCommand::Package { package, flags } => cmd_useflags_package(&package, &flags).await,
        UseflagsCommand::Expand { variable } => cmd_useflags_expand(variable).await,
        UseflagsCommand::Validate => cmd_useflags_validate().await,
    }
}

/// List available USE flags
async fn cmd_useflags_list(
    category: Option<String>,
    _global: bool,
    verbose: bool,
) -> buckos_package::Result<()> {
    // Define categorized USE flags
    let flags_by_category = get_use_flags_by_category();

    if let Some(cat) = category {
        // Show only requested category
        if let Some(flags) = flags_by_category.get(&cat) {
            println!(
                "{}",
                style(format!("USE Flags: {}", cat)).bold().underlined()
            );
            println!();
            for (flag, description) in flags {
                if verbose {
                    println!("  {} - {}", style(flag).green(), description);
                } else {
                    print!("{} ", style(flag).green());
                }
            }
            if !verbose {
                println!();
            }
        } else {
            println!("{} Unknown category: {}", style(">>>").yellow().bold(), cat);
            println!("\nAvailable categories:");
            for cat in flags_by_category.keys() {
                println!("  - {}", cat);
            }
        }
    } else {
        // Show all categories
        println!(
            "{}",
            style("Available USE Flags by Category").bold().underlined()
        );
        println!();

        for (cat, flags) in &flags_by_category {
            println!("{}", style(cat).cyan().bold());
            for (flag, description) in flags {
                if verbose {
                    println!("  {} - {}", style(flag).green(), description);
                } else {
                    print!("  {}", style(flag).green());
                }
            }
            if !verbose {
                println!();
            }
            println!();
        }
    }

    Ok(())
}

/// Get USE flags organized by category
fn get_use_flags_by_category() -> HashMap<String, Vec<(&'static str, &'static str)>> {
    let mut categories = HashMap::new();

    categories.insert(
        "build".to_string(),
        vec![
            ("debug", "Enable debugging symbols and assertions"),
            ("doc", "Build and install documentation"),
            ("examples", "Install example files"),
            ("static", "Build static libraries"),
            ("test", "Enable test suite during build"),
            ("lto", "Enable Link Time Optimization"),
        ],
    );

    categories.insert(
        "security".to_string(),
        vec![
            ("hardened", "Enable security hardening features"),
            ("pie", "Build position independent executables"),
            ("ssp", "Enable stack smashing protection"),
            ("caps", "Use Linux capabilities library"),
            ("seccomp", "Enable seccomp sandboxing"),
            ("selinux", "Enable SELinux support"),
        ],
    );

    categories.insert(
        "network".to_string(),
        vec![
            ("ipv6", "Enable IPv6 support"),
            ("ssl", "Enable SSL/TLS support (OpenSSL)"),
            ("gnutls", "Enable GnuTLS support"),
            ("http2", "Enable HTTP/2 support"),
            ("curl", "Use libcurl for HTTP operations"),
        ],
    );

    categories.insert(
        "compression".to_string(),
        vec![
            ("zlib", "Enable zlib compression"),
            ("bzip2", "Enable bzip2 compression"),
            ("zstd", "Enable Zstandard compression"),
            ("lz4", "Enable LZ4 compression"),
            ("brotli", "Enable Brotli compression"),
        ],
    );

    categories.insert(
        "graphics".to_string(),
        vec![
            ("X", "Enable X11 support"),
            ("wayland", "Enable Wayland support"),
            ("opengl", "Enable OpenGL support"),
            ("vulkan", "Enable Vulkan support"),
            ("gtk", "Enable GTK+ toolkit"),
            ("qt5", "Enable Qt5 toolkit"),
            ("qt6", "Enable Qt6 toolkit"),
        ],
    );

    categories.insert(
        "audio".to_string(),
        vec![
            ("alsa", "Enable ALSA audio support"),
            ("pulseaudio", "Enable PulseAudio support"),
            ("pipewire", "Enable PipeWire support"),
            ("ffmpeg", "Enable FFmpeg support"),
        ],
    );

    categories.insert(
        "language".to_string(),
        vec![
            ("python", "Build Python bindings"),
            ("perl", "Build Perl bindings"),
            ("ruby", "Build Ruby bindings"),
            ("lua", "Build Lua bindings"),
        ],
    );

    categories.insert(
        "system".to_string(),
        vec![
            ("dbus", "Enable D-Bus support"),
            ("systemd", "Enable systemd integration"),
            ("pam", "Enable PAM authentication"),
            ("acl", "Enable Access Control Lists"),
            ("udev", "Enable udev device management"),
        ],
    );

    categories
}

/// Show information about a specific USE flag
async fn cmd_useflags_info(flag: &str) -> buckos_package::Result<()> {
    let flags_by_category = get_use_flags_by_category();

    for (category, flags) in &flags_by_category {
        for (name, description) in flags {
            if *name == flag {
                println!("{}", style("USE Flag Information").bold().underlined());
                println!();
                println!("  {}: {}", style("Flag").bold(), style(name).green());
                println!("  {}: {}", style("Category").bold(), category);
                println!("  {}: {}", style("Description").bold(), description);
                return Ok(());
            }
        }
    }

    // Check USE_EXPAND variables
    let expand_vars = get_use_expand_variables();
    for (var_name, values) in &expand_vars {
        if values.contains(&flag.to_string()) {
            println!("{}", style("USE_EXPAND Variable").bold().underlined());
            println!();
            println!("  {}: {}", style("Value").bold(), style(flag).green());
            println!("  {}: {}", style("Variable").bold(), var_name);
            return Ok(());
        }
    }

    println!(
        "{} USE flag '{}' not found",
        style(">>>").yellow().bold(),
        flag
    );
    Ok(())
}

/// Set global USE flags
async fn cmd_useflags_set(flags: &[String]) -> buckos_package::Result<()> {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/etc/buckos"))
        .join("buckos")
        .join("use.conf");

    // Parse flags
    let mut enabled = Vec::new();
    let mut disabled = Vec::new();

    for flag in flags {
        if flag.starts_with('-') {
            disabled.push(&flag[1..]);
        } else {
            enabled.push(flag.as_str());
        }
    }

    println!("{}", style("Setting USE flags").bold().underlined());
    println!();

    if !enabled.is_empty() {
        println!("  {}: {}", style("Enabling").green(), enabled.join(" "));
    }
    if !disabled.is_empty() {
        println!("  {}: {}", style("Disabling").red(), disabled.join(" "));
    }

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Build USE string
    let use_string = flags.join(" ");

    // Write to config file
    let content = format!(
        "# BuckOs USE flags configuration\n# Generated by buckos useflags set\n\nUSE=\"{}\"\n",
        use_string
    );

    match fs::write(&config_path, content) {
        Ok(_) => {
            println!();
            println!(
                "{} Configuration saved to: {}",
                style(">>>").green().bold(),
                config_path.display()
            );
        }
        Err(e) => {
            println!();
            println!(
                "{} Failed to save configuration: {}",
                style(">>>").red().bold(),
                e
            );
            println!("You may need to run with elevated privileges or set USE flags manually.");
            println!();
            println!("Add this to your make.conf or buckos config:");
            println!("  USE=\"{}\"", use_string);
        }
    }

    Ok(())
}

/// Get current USE flag configuration
async fn cmd_useflags_get(format: &str) -> buckos_package::Result<()> {
    let config = Config::default();

    // Get USE flags from config
    let use_flags: Vec<String> = vec![
        "ssl".to_string(),
        "http2".to_string(),
        "ipv6".to_string(),
        "zstd".to_string(),
    ]; // Default example flags

    match format {
        "json" => {
            let output = serde_json::json!({
                "use_flags": use_flags,
                "arch": config.arch,
                "chost": config.chost,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
        }
        "toml" => {
            println!("[use]");
            println!("flags = {:?}", use_flags);
            println!();
            println!("[system]");
            println!("arch = \"{}\"", config.arch);
            println!("chost = \"{}\"", config.chost);
        }
        _ => {
            println!("{}", style("Current USE Configuration").bold().underlined());
            println!();
            println!("  {}: {}", style("USE").bold(), use_flags.join(" "));
            println!("  {}: {}", style("ARCH").bold(), config.arch);
            println!("  {}: {}", style("CHOST").bold(), config.chost);
            println!("  {}: {}", style("CFLAGS").bold(), config.cflags);
        }
    }

    Ok(())
}

/// Set per-package USE flags
async fn cmd_useflags_package(package: &str, flags: &[String]) -> buckos_package::Result<()> {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/etc/buckos"))
        .join("buckos")
        .join("package.use");

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Format the package.use entry
    let entry = format!("{} {}\n", package, flags.join(" "));

    println!(
        "{}",
        style("Setting per-package USE flags").bold().underlined()
    );
    println!();
    println!("  {}: {}", style("Package").bold(), package);
    println!("  {}: {}", style("Flags").bold(), flags.join(" "));

    // Append to package.use file
    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                println!();
                println!(
                    "{} Failed to write configuration: {}",
                    style(">>>").red().bold(),
                    e
                );
            } else {
                println!();
                println!(
                    "{} Configuration saved to: {}",
                    style(">>>").green().bold(),
                    config_path.display()
                );
            }
        }
        Err(e) => {
            println!();
            println!(
                "{} Failed to open configuration file: {}",
                style(">>>").red().bold(),
                e
            );
            println!("\nAdd this to your package.use:");
            println!("  {}", entry.trim());
        }
    }

    Ok(())
}

/// Show USE_EXPAND variables
async fn cmd_useflags_expand(variable: Option<String>) -> buckos_package::Result<()> {
    let expand_vars = get_use_expand_variables();

    if let Some(var) = variable {
        if let Some(values) = expand_vars.get(&var.to_uppercase()) {
            println!(
                "{}",
                style(format!("{}", var.to_uppercase())).bold().underlined()
            );
            println!();
            for value in values {
                println!("  {}", style(value).green());
            }
        } else {
            println!(
                "{} Unknown USE_EXPAND variable: {}",
                style(">>>").yellow().bold(),
                var
            );
            println!("\nAvailable variables:");
            for var_name in expand_vars.keys() {
                println!("  - {}", var_name);
            }
        }
    } else {
        println!("{}", style("USE_EXPAND Variables").bold().underlined());
        println!();
        for (var_name, values) in &expand_vars {
            println!("{}:", style(var_name).cyan().bold());
            let values_str: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
            println!("  {}", values_str.join(" "));
            println!();
        }
    }

    Ok(())
}

/// Get USE_EXPAND variables
fn get_use_expand_variables() -> HashMap<String, Vec<String>> {
    let mut vars = HashMap::new();

    vars.insert(
        "CPU_FLAGS_X86".to_string(),
        vec![
            "aes", "avx", "avx2", "avx512f", "avx512dq", "avx512cd", "avx512bw", "avx512vl", "mmx",
            "mmxext", "pclmul", "popcnt", "sse", "sse2", "sse3", "ssse3", "sse4_1", "sse4_2",
            "sse4a", "f16c", "fma", "fma4", "xop",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    vars.insert(
        "VIDEO_CARDS".to_string(),
        vec![
            "amdgpu",
            "ast",
            "dummy",
            "fbdev",
            "i915",
            "i965",
            "intel",
            "mga",
            "nouveau",
            "nvidia",
            "r128",
            "r600",
            "radeon",
            "radeonsi",
            "vesa",
            "via",
            "virtualbox",
            "vmware",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    vars.insert(
        "INPUT_DEVICES".to_string(),
        vec![
            "evdev",
            "joystick",
            "keyboard",
            "libinput",
            "mouse",
            "synaptics",
            "vmmouse",
            "wacom",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    vars.insert(
        "PYTHON_TARGETS".to_string(),
        vec!["python3_10", "python3_11", "python3_12", "python3_13"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    vars.insert(
        "RUBY_TARGETS".to_string(),
        vec!["ruby31", "ruby32", "ruby33"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    vars
}

/// Validate USE flag configuration
async fn cmd_useflags_validate() -> buckos_package::Result<()> {
    println!(
        "{}",
        style("Validating USE flag configuration")
            .bold()
            .underlined()
    );
    println!();

    let mut issues = Vec::new();

    // Check for conflicting flags
    let conflicts = vec![
        ("systemd", "elogind"),
        ("pulseaudio", "pipewire"),
        ("ssl", "gnutls"),
        ("gtk", "qt5"),
    ];

    // Simulate checking current config
    let current_flags: HashSet<&str> = ["ssl", "http2", "systemd"].into_iter().collect();

    for (flag1, flag2) in &conflicts {
        if current_flags.contains(flag1) && current_flags.contains(flag2) {
            issues.push(format!(
                "Conflicting flags: {} and {} are both enabled",
                flag1, flag2
            ));
        }
    }

    if issues.is_empty() {
        println!("{} No issues found", style(">>>").green().bold());
    } else {
        println!(
            "{} Found {} issue(s):",
            style(">>>").yellow().bold(),
            issues.len()
        );
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    Ok(())
}

/// System detection data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemDetection {
    cpu_features: Vec<String>,
    gpu_drivers: Vec<String>,
    audio_systems: Vec<String>,
    network_features: Vec<String>,
    recommended_use_flags: Vec<String>,
}

/// Detect system capabilities
async fn cmd_detect(args: DetectArgs) -> buckos_package::Result<()> {
    let detect_all = args.all || (!args.cpu && !args.gpu && !args.audio && !args.network);

    let mut detection = SystemDetection {
        cpu_features: Vec::new(),
        gpu_drivers: Vec::new(),
        audio_systems: Vec::new(),
        network_features: Vec::new(),
        recommended_use_flags: Vec::new(),
    };

    if detect_all || args.cpu {
        detection.cpu_features = detect_cpu_features();
    }

    if detect_all || args.gpu {
        detection.gpu_drivers = detect_gpu();
    }

    if detect_all || args.audio {
        detection.audio_systems = detect_audio();
    }

    if detect_all || args.network {
        detection.network_features = detect_network();
    }

    // Generate recommended USE flags based on detection
    detection.recommended_use_flags = generate_recommended_flags(&detection);

    // Output in requested format
    let output = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&detection).unwrap_or_default(),
        "toml" => format_detection_toml(&detection),
        "shell" => format_detection_shell(&detection),
        _ => format_detection_text(&detection),
    };

    // Write to file or stdout
    if let Some(path) = args.output {
        fs::write(&path, &output)?;
        println!(
            "{} Detection results saved to: {}",
            style(">>>").green().bold(),
            path
        );
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Detect CPU features
fn detect_cpu_features() -> Vec<String> {
    let mut features = Vec::new();

    // Read /proc/cpuinfo on Linux
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        let cpu_flags = [
            "aes", "avx", "avx2", "avx512f", "avx512dq", "avx512cd", "avx512bw", "avx512vl", "mmx",
            "pclmul", "popcnt", "sse", "sse2", "sse3", "ssse3", "sse4_1", "sse4_2", "f16c", "fma",
        ];

        for flag in cpu_flags {
            if cpuinfo.contains(flag) {
                features.push(flag.to_string());
            }
        }
    }

    if features.is_empty() {
        // Default to basic x86_64 features
        features = vec!["sse".to_string(), "sse2".to_string(), "mmx".to_string()];
    }

    features
}

/// Detect GPU/video hardware
fn detect_gpu() -> Vec<String> {
    let mut drivers = Vec::new();

    // Check for common GPU vendors
    let checks = vec![
        ("/sys/module/nvidia", "nvidia"),
        ("/sys/module/amdgpu", "amdgpu"),
        ("/sys/module/i915", "i915"),
        ("/sys/module/nouveau", "nouveau"),
        ("/sys/module/radeon", "radeon"),
    ];

    for (path, driver) in checks {
        if std::path::Path::new(path).exists() {
            drivers.push(driver.to_string());
        }
    }

    if drivers.is_empty() {
        // Check lspci output if available
        drivers.push("fbdev".to_string());
        drivers.push("vesa".to_string());
    }

    drivers
}

/// Detect audio systems
fn detect_audio() -> Vec<String> {
    let mut systems = Vec::new();

    if std::path::Path::new("/proc/asound").exists() {
        systems.push("alsa".to_string());
    }

    if std::path::Path::new("/run/user/1000/pulse").exists()
        || std::path::Path::new("/var/run/pulse").exists()
    {
        systems.push("pulseaudio".to_string());
    }

    if std::path::Path::new("/run/user/1000/pipewire-0").exists() {
        systems.push("pipewire".to_string());
    }

    if systems.is_empty() {
        systems.push("alsa".to_string());
    }

    systems
}

/// Detect network features
fn detect_network() -> Vec<String> {
    let mut features = Vec::new();

    // Check for IPv6 support
    if std::path::Path::new("/proc/net/if_inet6").exists() {
        features.push("ipv6".to_string());
    }

    // SSL/TLS is generally always available
    features.push("ssl".to_string());
    features.push("http2".to_string());

    features
}

/// Generate recommended USE flags based on detection
fn generate_recommended_flags(detection: &SystemDetection) -> Vec<String> {
    let mut flags = Vec::new();

    // Add CPU flags
    for feature in &detection.cpu_features {
        flags.push(format!("cpu_flags_x86_{}", feature));
    }

    // Add GPU-related flags
    if detection
        .gpu_drivers
        .iter()
        .any(|d| d == "nvidia" || d == "amdgpu" || d == "i915")
    {
        flags.push("vulkan".to_string());
        flags.push("opengl".to_string());
    }

    // Add audio flags
    for audio in &detection.audio_systems {
        flags.push(audio.clone());
    }

    // Add network flags
    for net in &detection.network_features {
        flags.push(net.clone());
    }

    // Add common flags
    flags.push("zstd".to_string());
    flags.push("dbus".to_string());

    flags
}

/// Format detection output as text
fn format_detection_text(detection: &SystemDetection) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{}\n\n",
        style("System Detection Results").bold().underlined()
    ));

    if !detection.cpu_features.is_empty() {
        output.push_str(&format!("{}:\n", style("CPU Features").cyan().bold()));
        output.push_str(&format!("  {}\n\n", detection.cpu_features.join(" ")));
    }

    if !detection.gpu_drivers.is_empty() {
        output.push_str(&format!("{}:\n", style("GPU Drivers").cyan().bold()));
        output.push_str(&format!("  {}\n\n", detection.gpu_drivers.join(" ")));
    }

    if !detection.audio_systems.is_empty() {
        output.push_str(&format!("{}:\n", style("Audio Systems").cyan().bold()));
        output.push_str(&format!("  {}\n\n", detection.audio_systems.join(" ")));
    }

    if !detection.network_features.is_empty() {
        output.push_str(&format!("{}:\n", style("Network Features").cyan().bold()));
        output.push_str(&format!("  {}\n\n", detection.network_features.join(" ")));
    }

    output.push_str(&format!(
        "{}:\n",
        style("Recommended USE Flags").green().bold()
    ));
    output.push_str(&format!(
        "  {}\n",
        detection.recommended_use_flags.join(" ")
    ));

    output
}

/// Format detection output as TOML
fn format_detection_toml(detection: &SystemDetection) -> String {
    format!(
        r#"# BuckOs System Detection
# Generated by buckos detect

[cpu]
features = {:?}

[gpu]
drivers = {:?}

[audio]
systems = {:?}

[network]
features = {:?}

[recommended]
use_flags = {:?}
"#,
        detection.cpu_features,
        detection.gpu_drivers,
        detection.audio_systems,
        detection.network_features,
        detection.recommended_use_flags
    )
}

/// Format detection output as shell script
fn format_detection_shell(detection: &SystemDetection) -> String {
    let mut output = String::new();

    output.push_str("#!/bin/bash\n");
    output.push_str("# BuckOs System Detection\n");
    output.push_str("# Generated by buckos detect\n\n");

    output.push_str(&format!(
        "CPU_FLAGS_X86=\"{}\"\n",
        detection.cpu_features.join(" ")
    ));

    output.push_str(&format!(
        "VIDEO_CARDS=\"{}\"\n",
        detection.gpu_drivers.join(" ")
    ));

    output.push_str(&format!(
        "USE=\"{}\"\n",
        detection.recommended_use_flags.join(" ")
    ));

    output
}

/// Generate system configuration
async fn cmd_configure(args: ConfigureArgs) -> buckos_package::Result<()> {
    println!("{} Generating configuration...", style(">>>").blue().bold());

    // Get profile settings
    let profile_flags = get_profile_flags(&args.profile);

    // Combine with user-specified flags
    let mut all_flags: Vec<String> = profile_flags;
    all_flags.extend(args.use_flags.clone());

    // Auto-detect hardware if requested
    let mut detection = None;
    if args.auto_detect {
        let detect_result = SystemDetection {
            cpu_features: detect_cpu_features(),
            gpu_drivers: detect_gpu(),
            audio_systems: detect_audio(),
            network_features: detect_network(),
            recommended_use_flags: Vec::new(),
        };

        // Add detected features
        for feature in &detect_result.cpu_features {
            all_flags.push(format!("cpu_flags_x86_{}", feature));
        }

        detection = Some(detect_result);
    }

    // Generate output in requested format
    let output = match args.format.as_str() {
        "json" => generate_config_json(&args.profile, &all_flags, &args.arch),
        "toml" => generate_config_toml(&args.profile, &all_flags, &args.arch),
        "shell" => generate_config_shell(&args.profile, &all_flags, &args.arch),
        _ => generate_config_bzl(&args.profile, &all_flags, &args.arch),
    };

    // Write to file or stdout
    if let Some(path) = args.output {
        fs::write(&path, &output)?;
        println!(
            "{} Configuration saved to: {}",
            style(">>>").green().bold(),
            path
        );

        if args.format == "bzl" {
            println!();
            println!("Usage:");
            println!("  buck2 build //packages/linux/... --config {}", path);
        }
    } else {
        println!("{}", output);
    }

    // Print summary
    println!();
    println!("{}", style("Configuration Summary").bold().underlined());
    println!("  Profile: {}", style(&args.profile).cyan());
    println!("  Architecture: {}", args.arch);
    println!("  USE flags: {}", all_flags.len());

    if let Some(det) = detection {
        println!("  Detected CPU features: {}", det.cpu_features.len());
        println!("  Detected GPU drivers: {}", det.gpu_drivers.len());
    }

    Ok(())
}

/// Get USE flags for a profile
fn get_profile_flags(profile: &str) -> Vec<String> {
    match profile {
        "minimal" => vec![
            "-X".to_string(),
            "-wayland".to_string(),
            "-pulseaudio".to_string(),
            "-pipewire".to_string(),
            "-gtk".to_string(),
            "-qt5".to_string(),
            "ipv6".to_string(),
            "ssl".to_string(),
        ],
        "server" => vec![
            "-X".to_string(),
            "-wayland".to_string(),
            "-pulseaudio".to_string(),
            "-gtk".to_string(),
            "ipv6".to_string(),
            "ssl".to_string(),
            "http2".to_string(),
            "zstd".to_string(),
            "lz4".to_string(),
            "caps".to_string(),
            "seccomp".to_string(),
        ],
        "desktop" => vec![
            "X".to_string(),
            "wayland".to_string(),
            "pulseaudio".to_string(),
            "pipewire".to_string(),
            "dbus".to_string(),
            "gtk".to_string(),
            "opengl".to_string(),
            "vulkan".to_string(),
            "ipv6".to_string(),
            "ssl".to_string(),
            "http2".to_string(),
            "zstd".to_string(),
        ],
        "developer" => vec![
            "X".to_string(),
            "wayland".to_string(),
            "dbus".to_string(),
            "debug".to_string(),
            "doc".to_string(),
            "test".to_string(),
            "examples".to_string(),
            "python".to_string(),
            "ipv6".to_string(),
            "ssl".to_string(),
            "http2".to_string(),
        ],
        "hardened" => vec![
            "hardened".to_string(),
            "pie".to_string(),
            "ssp".to_string(),
            "caps".to_string(),
            "seccomp".to_string(),
            "ipv6".to_string(),
            "ssl".to_string(),
            "-debug".to_string(),
            "-test".to_string(),
        ],
        _ => vec![
            "ipv6".to_string(),
            "ssl".to_string(),
            "http2".to_string(),
            "dbus".to_string(),
            "zstd".to_string(),
        ],
    }
}

/// Generate Buck2 configuration
fn generate_config_bzl(profile: &str, flags: &[String], arch: &str) -> String {
    format!(
        r#"# BuckOs Configuration
# Generated by buckos configure
# Profile: {}

load("//defs:use_flags.bzl", "set_use_flags", "package_config")

BUCKOS_CONFIG = package_config(
    # Global USE flags
    use_flags = [{}],

    # Base profile
    profile = "{}",

    # Target architecture
    arch = "{}",
)

# Export for use in BUCK files
GLOBAL_USE = set_use_flags(BUCKOS_CONFIG.use_flags)
"#,
        profile,
        flags
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(", "),
        profile,
        arch
    )
}

/// Generate JSON configuration
fn generate_config_json(profile: &str, flags: &[String], arch: &str) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "profile": profile,
        "use_flags": flags,
        "arch": arch
    }))
    .unwrap_or_default()
}

/// Generate TOML configuration
fn generate_config_toml(profile: &str, flags: &[String], arch: &str) -> String {
    format!(
        r#"# BuckOs Configuration
# Generated by buckos configure

[system]
profile = "{}"
arch = "{}"

[use]
flags = [
{}
]
"#,
        profile,
        arch,
        flags
            .iter()
            .map(|f| format!("    \"{}\",", f))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Generate shell configuration
fn generate_config_shell(profile: &str, flags: &[String], arch: &str) -> String {
    format!(
        r#"#!/bin/bash
# BuckOs Configuration
# Generated by buckos configure

# Profile: {}
BUCKOS_PROFILE="{}"

# Architecture
ARCH="{}"

# USE flags
USE="{}"

export BUCKOS_PROFILE ARCH USE
"#,
        profile,
        profile,
        arch,
        flags.join(" ")
    )
}

/// Package set management command
async fn cmd_set(
    pm: &PackageManager,
    args: SetArgs,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    match args.subcommand {
        SetCommand::List { r#type } => cmd_set_list(r#type).await,
        SetCommand::Show { set_name } => cmd_set_show(&set_name).await,
        SetCommand::Install { set_name } => cmd_set_install(pm, &set_name, emerge_opts).await,
        SetCommand::Compare { set1, set2 } => cmd_set_compare(&set1, &set2).await,
    }
}

/// List available package sets
async fn cmd_set_list(set_type: Option<String>) -> buckos_package::Result<()> {
    println!("{}", style("Available Package Sets").bold().underlined());
    println!();

    let sets = get_package_sets();

    if let Some(t) = set_type {
        // Filter by type
        if let Some(type_sets) = sets.get(&t) {
            println!("{}:", style(&t).cyan().bold());
            for (name, info) in type_sets {
                println!("  {} - {}", style(name).green(), info);
            }
        } else {
            println!("{} Unknown set type: {}", style(">>>").yellow().bold(), t);
            println!("\nAvailable types: system, task, desktop");
        }
    } else {
        // Show all sets
        for (set_type, type_sets) in &sets {
            println!("{}:", style(set_type).cyan().bold());
            for (name, description) in type_sets {
                println!("  {} - {}", style(name).green(), description);
            }
            println!();
        }
    }

    Ok(())
}

/// Get package sets organized by type
fn get_package_sets() -> HashMap<String, Vec<(&'static str, &'static str)>> {
    let mut sets = HashMap::new();

    sets.insert(
        "system".to_string(),
        vec![
            ("minimal", "Minimal bootable system"),
            ("server", "Server base system"),
            ("desktop", "Desktop base system"),
            ("developer", "Development environment"),
            ("hardened", "Security-hardened system"),
        ],
    );

    sets.insert(
        "task".to_string(),
        vec![
            ("web-server", "Web server packages"),
            ("database", "Database packages"),
            ("container", "Container runtime packages"),
            ("virtualization", "Virtualization packages"),
            ("monitoring", "System monitoring packages"),
        ],
    );

    sets.insert(
        "desktop".to_string(),
        vec![
            ("gnome", "GNOME desktop environment"),
            ("kde", "KDE Plasma desktop environment"),
            ("xfce", "Xfce desktop environment"),
            ("sway", "Sway Wayland compositor"),
        ],
    );

    sets
}

/// Show contents of a package set
async fn cmd_set_show(set_name: &str) -> buckos_package::Result<()> {
    let packages = get_set_packages(set_name);

    if packages.is_empty() {
        println!("{} Unknown set: {}", style(">>>").yellow().bold(), set_name);
        return Ok(());
    }

    println!(
        "{}",
        style(format!("Package Set: {}", set_name))
            .bold()
            .underlined()
    );
    println!();

    for pkg in &packages {
        println!("  {}", style(pkg).green());
    }

    println!();
    println!("Total: {} packages", packages.len());

    Ok(())
}

/// Get packages in a set
fn get_set_packages(set_name: &str) -> Vec<String> {
    match set_name {
        "minimal" => vec![
            "core/bash".to_string(),
            "core/busybox".to_string(),
            "core/musl".to_string(),
            "core/linux-headers".to_string(),
        ],
        "server" => vec![
            "core/bash".to_string(),
            "core/openssl".to_string(),
            "core/zlib".to_string(),
            "core/glibc".to_string(),
            "network/openssh".to_string(),
            "system/systemd".to_string(),
        ],
        "desktop" => vec![
            "core/bash".to_string(),
            "core/openssl".to_string(),
            "graphics/mesa".to_string(),
            "graphics/xorg-server".to_string(),
            "audio/pipewire".to_string(),
            "desktop/dbus".to_string(),
        ],
        "developer" => vec![
            "core/bash".to_string(),
            "dev-tools/gcc".to_string(),
            "dev-tools/clang".to_string(),
            "dev-tools/cmake".to_string(),
            "dev-tools/git".to_string(),
            "dev-tools/gdb".to_string(),
        ],
        "hardened" => vec![
            "core/bash".to_string(),
            "core/openssl".to_string(),
            "security/audit".to_string(),
            "security/libcap".to_string(),
        ],
        "web-server" => vec![
            "www/nginx".to_string(),
            "www/apache".to_string(),
            "network/curl".to_string(),
        ],
        "database" => vec![
            "database/postgresql".to_string(),
            "database/mariadb".to_string(),
            "database/sqlite".to_string(),
        ],
        "container" => vec![
            "app-containers/docker".to_string(),
            "app-containers/podman".to_string(),
            "app-containers/containerd".to_string(),
        ],
        "gnome" => vec![
            "desktop/gnome-shell".to_string(),
            "desktop/gnome-terminal".to_string(),
            "desktop/nautilus".to_string(),
        ],
        "kde" => vec![
            "desktop/plasma-desktop".to_string(),
            "desktop/konsole".to_string(),
            "desktop/dolphin".to_string(),
        ],
        "xfce" => vec![
            "desktop/xfce4-panel".to_string(),
            "desktop/xfce4-terminal".to_string(),
            "desktop/thunar".to_string(),
        ],
        "sway" => vec![
            "desktop/sway".to_string(),
            "desktop/foot".to_string(),
            "desktop/waybar".to_string(),
        ],
        _ => Vec::new(),
    }
}

/// Install all packages in a set
async fn cmd_set_install(
    pm: &PackageManager,
    set_name: &str,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    let packages = get_set_packages(set_name);

    if packages.is_empty() {
        println!("{} Unknown set: {}", style(">>>").yellow().bold(), set_name);
        return Ok(());
    }

    println!(
        "{} Installing set: {} ({} packages)",
        style(">>>").blue().bold(),
        set_name,
        packages.len()
    );

    let opts = InstallOptions {
        force: false,
        no_deps: false,
        build: true,
        use_flags: Vec::new(),
        oneshot: emerge_opts.oneshot,
        fetch_only: emerge_opts.fetch_only,
        deep: emerge_opts.deep,
        newuse: emerge_opts.newuse,
        empty_tree: false,
        no_replace: true,
        use_pkg: emerge_opts.use_pkg,
        use_pkg_only: emerge_opts.use_pkg_only,
        get_binpkg: emerge_opts.get_binpkg,
        get_binpkg_only: emerge_opts.get_binpkg_only,
        build_pkg: emerge_opts.build_pkg,
        build_pkg_only: emerge_opts.build_pkg_only,
    };

    // Resolve dependencies
    let resolution = pm.resolve_packages(&packages, &opts).await?;

    if resolution.packages.is_empty() {
        println!(
            "\n{} All packages in set are already installed",
            style(">>>").green().bold()
        );
        return Ok(());
    }

    // Display package list
    print_emerge_list(&resolution, emerge_opts, "install")?;

    // Pretend mode
    if emerge_opts.pretend {
        return Ok(());
    }

    // Ask mode
    if emerge_opts.ask {
        if !Confirm::new()
            .with_prompt("Would you like to merge these packages?")
            .default(true)
            .interact()?
        {
            println!("{}", style(">>> Exiting.").yellow().bold());
            return Ok(());
        }
        println!();
    }

    pm.install(&packages, opts).await?;

    println!(
        "\n{} Set '{}' installed successfully",
        style(">>>").green().bold(),
        set_name
    );

    Ok(())
}

/// Compare two package sets
async fn cmd_set_compare(set1: &str, set2: &str) -> buckos_package::Result<()> {
    let packages1: HashSet<String> = get_set_packages(set1).into_iter().collect();
    let packages2: HashSet<String> = get_set_packages(set2).into_iter().collect();

    if packages1.is_empty() {
        println!("{} Unknown set: {}", style(">>>").yellow().bold(), set1);
        return Ok(());
    }
    if packages2.is_empty() {
        println!("{} Unknown set: {}", style(">>>").yellow().bold(), set2);
        return Ok(());
    }

    let added: Vec<_> = packages2.difference(&packages1).collect();
    let removed: Vec<_> = packages1.difference(&packages2).collect();
    let common: Vec<_> = packages1.intersection(&packages2).collect();

    println!(
        "{} vs {}",
        style(set1).cyan().bold(),
        style(set2).cyan().bold()
    );
    println!();

    if !added.is_empty() {
        println!("{}:", style("Added in second set").green());
        for pkg in &added {
            println!("  + {}", pkg);
        }
        println!();
    }

    if !removed.is_empty() {
        println!("{}:", style("Removed from first set").red());
        for pkg in &removed {
            println!("  - {}", pkg);
        }
        println!();
    }

    println!("Common packages: {}", style(common.len()).bold());

    Ok(())
}

/// Patch management command
async fn cmd_patch(args: PatchArgs) -> buckos_package::Result<()> {
    match args.subcommand {
        PatchCommand::List { package } => cmd_patch_list(&package).await,
        PatchCommand::Info {
            package,
            patch_name,
        } => cmd_patch_info(&package, &patch_name).await,
        PatchCommand::Add {
            package,
            patch_file,
        } => cmd_patch_add(&package, &patch_file).await,
        PatchCommand::Remove {
            package,
            patch_name,
        } => cmd_patch_remove(&package, &patch_name).await,
        PatchCommand::Check { package } => cmd_patch_check(&package).await,
        PatchCommand::Order { package } => cmd_patch_order(&package).await,
    }
}

/// List patches for a package
async fn cmd_patch_list(package: &str) -> buckos_package::Result<()> {
    println!(
        "{}",
        style(format!("Patches for {}", package))
            .bold()
            .underlined()
    );
    println!();

    // Check for patches in standard locations
    let patch_dirs = vec![
        format!("/etc/portage/patches/{}", package),
        format!("/var/db/buckos/patches/{}", package),
    ];

    let mut found_patches = Vec::new();

    for dir in &patch_dirs {
        let path = std::path::Path::new(dir);
        if path.exists() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    if entry.path().extension().map_or(false, |e| e == "patch") {
                        let name = entry.file_name().to_string_lossy().to_string();
                        found_patches.push((name, dir.clone()));
                    }
                }
            }
        }
    }

    if found_patches.is_empty() {
        println!("No patches found for {}", package);
        println!();
        println!("Patch locations checked:");
        for dir in &patch_dirs {
            println!("  {}", dir);
        }
    } else {
        for (patch, source) in &found_patches {
            println!("  {} ({})", style(patch).green(), source);
        }
        println!();
        println!("Total: {} patches", found_patches.len());
    }

    Ok(())
}

/// Show patch information
async fn cmd_patch_info(package: &str, patch_name: &str) -> buckos_package::Result<()> {
    let patch_path = format!("/etc/portage/patches/{}/{}", package, patch_name);

    if !std::path::Path::new(&patch_path).exists() {
        println!(
            "{} Patch not found: {}",
            style(">>>").yellow().bold(),
            patch_path
        );
        return Ok(());
    }

    println!("{}", style("Patch Information").bold().underlined());
    println!();
    println!("  {}: {}", style("Name").bold(), patch_name);
    println!("  {}: {}", style("Package").bold(), package);
    println!("  {}: {}", style("Path").bold(), patch_path);

    // Read first few lines of patch to show description
    if let Ok(content) = fs::read_to_string(&patch_path) {
        let lines: Vec<&str> = content.lines().take(10).collect();
        if !lines.is_empty() {
            println!();
            println!("{}:", style("Header").bold());
            for line in lines {
                println!("  {}", line);
            }
        }
    }

    Ok(())
}

/// Add a user patch
async fn cmd_patch_add(package: &str, patch_file: &str) -> buckos_package::Result<()> {
    let patch_dir = format!("/etc/portage/patches/{}", package);
    let patch_path = std::path::Path::new(&patch_dir);

    // Create directory if it doesn't exist
    fs::create_dir_all(&patch_dir)?;

    // Copy patch file
    let source = std::path::Path::new(patch_file);
    let file_name = source.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid patch file name")
    })?;
    let dest = patch_path.join(file_name);

    fs::copy(source, &dest)?;

    println!(
        "{} Added patch: {}",
        style(">>>").green().bold(),
        dest.display()
    );
    println!();
    println!(
        "The patch will be applied during the next build of {}",
        package
    );

    Ok(())
}

/// Remove a user patch
async fn cmd_patch_remove(package: &str, patch_name: &str) -> buckos_package::Result<()> {
    let patch_path = format!("/etc/portage/patches/{}/{}", package, patch_name);

    if !std::path::Path::new(&patch_path).exists() {
        println!(
            "{} Patch not found: {}",
            style(">>>").yellow().bold(),
            patch_path
        );
        return Ok(());
    }

    fs::remove_file(&patch_path)?;

    println!(
        "{} Removed patch: {}",
        style(">>>").green().bold(),
        patch_path
    );

    Ok(())
}

/// Check if patches apply cleanly
async fn cmd_patch_check(package: &str) -> buckos_package::Result<()> {
    println!(
        "{} Checking patches for {}...",
        style(">>>").blue().bold(),
        package
    );

    let patch_dir = format!("/etc/portage/patches/{}", package);
    let path = std::path::Path::new(&patch_dir);

    if !path.exists() {
        println!("{} No patches to check", style(">>>").green().bold());
        return Ok(());
    }

    // In a real implementation, this would:
    // 1. Download and extract the source
    // 2. Apply patches with --dry-run
    // 3. Report any failures

    println!(
        "{} Patch check completed (dry-run not implemented yet)",
        style(">>>").yellow().bold()
    );

    Ok(())
}

/// Show patch application order
async fn cmd_patch_order(package: &str) -> buckos_package::Result<()> {
    println!(
        "{}",
        style(format!("Patch Order for {}", package))
            .bold()
            .underlined()
    );
    println!();

    let patch_dir = format!("/etc/portage/patches/{}", package);
    let series_file = format!("{}/series", patch_dir);

    if std::path::Path::new(&series_file).exists() {
        // Use series file if it exists
        if let Ok(content) = fs::read_to_string(&series_file) {
            let mut idx = 1;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    println!("  {}. {}", idx, line);
                    idx += 1;
                }
            }
        }
    } else {
        // List patches alphabetically
        let path = std::path::Path::new(&patch_dir);
        if path.exists() {
            let mut patches: Vec<_> = fs::read_dir(path)?
                .flatten()
                .filter(|e| e.path().extension().map_or(false, |e| e == "patch"))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            patches.sort();

            for (idx, patch) in patches.iter().enumerate() {
                println!("  {}. {}", idx + 1, patch);
            }

            if patches.is_empty() {
                println!("  No patches found");
            }
        } else {
            println!("  No patches found");
        }
    }

    Ok(())
}

/// Show package dependencies
async fn cmd_deps(pm: &PackageManager, args: DepsArgs) -> buckos_package::Result<()> {
    if let Some(pkg) = pm.info(&args.package).await? {
        if args.format == "json" {
            let output = serde_json::json!({
                "package": args.package,
                "dependencies": pkg.dependencies.iter().map(|d| &d.package).collect::<Vec<_>>(),
                "runtime_dependencies": pkg.runtime_dependencies.iter().map(|d| &d.package).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
        } else {
            println!(
                "{}",
                style(format!("Dependencies of {}", args.package))
                    .bold()
                    .underlined()
            );
            println!();

            if !pkg.dependencies.is_empty() {
                println!("{}:", style("Build Dependencies").cyan());
                for dep in &pkg.dependencies {
                    println!("  {}", dep.package);
                }
            }

            if !pkg.runtime_dependencies.is_empty() {
                println!();
                println!("{}:", style("Runtime Dependencies").cyan());
                for dep in &pkg.runtime_dependencies {
                    println!("  {}", dep.package);
                }
            }

            if pkg.dependencies.is_empty() && pkg.runtime_dependencies.is_empty() {
                println!("  No dependencies");
            }
        }
    } else {
        println!(
            "{} Package '{}' not found",
            style(">>>").yellow().bold(),
            args.package
        );
    }

    Ok(())
}

/// Show reverse dependencies
async fn cmd_rdeps(pm: &PackageManager, args: RdepsArgs) -> buckos_package::Result<()> {
    let rdeps = pm.get_reverse_dependencies(&args.package).await?;

    if args.format == "json" {
        let output = serde_json::json!({
            "package": args.package,
            "reverse_dependencies": rdeps,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!(
            "{}",
            style(format!("Reverse Dependencies of {}", args.package))
                .bold()
                .underlined()
        );
        println!();

        if rdeps.is_empty() {
            println!("  No packages depend on {}", args.package);
        } else {
            for rdep in &rdeps {
                println!("  {}", rdep);
            }
            println!();
            println!("Total: {} packages", rdeps.len());
        }
    }

    Ok(())
}

/// Profile management command
async fn cmd_profile(args: ProfileArgs) -> buckos_package::Result<()> {
    match args.subcommand {
        ProfileCommand::List => cmd_profile_list().await,
        ProfileCommand::Show { profile } => cmd_profile_show(&profile).await,
        ProfileCommand::Set { profile } => cmd_profile_set(&profile).await,
        ProfileCommand::Current => cmd_profile_current().await,
    }
}

/// List available profiles
async fn cmd_profile_list() -> buckos_package::Result<()> {
    println!("{}", style("Available Profiles").bold().underlined());
    println!();

    let profiles = vec![
        ("minimal", "Absolute minimum system", vec!["ipv6"]),
        (
            "server",
            "Server systems",
            vec!["ssl", "ipv6", "threads", "caps"],
        ),
        (
            "desktop",
            "Desktop systems",
            vec!["X", "dbus", "pulseaudio", "gtk", "ssl", "ipv6"],
        ),
        (
            "developer",
            "Development environment",
            vec!["debug", "doc", "test", "ssl", "ipv6"],
        ),
        (
            "hardened",
            "Security-focused systems",
            vec!["hardened", "pie", "ssp", "caps"],
        ),
        ("embedded", "Embedded systems", vec!["static", "-ipv6"]),
        (
            "container",
            "Container environments",
            vec!["static", "-pam", "-systemd"],
        ),
    ];

    for (name, description, flags) in &profiles {
        println!("  {} - {}", style(name).green().bold(), description);
        println!("    USE: {}", flags.join(" "));
        println!();
    }

    Ok(())
}

/// Show profile information
async fn cmd_profile_show(profile: &str) -> buckos_package::Result<()> {
    let profiles: HashMap<&str, (&str, Vec<&str>)> = vec![
        ("minimal", ("Absolute minimum system", vec!["ipv6"])),
        (
            "server",
            ("Server systems", vec!["ssl", "ipv6", "threads", "caps"]),
        ),
        (
            "desktop",
            (
                "Desktop systems",
                vec!["X", "dbus", "pulseaudio", "gtk", "ssl", "ipv6"],
            ),
        ),
        (
            "developer",
            (
                "Development environment",
                vec!["debug", "doc", "test", "ssl", "ipv6"],
            ),
        ),
        (
            "hardened",
            (
                "Security-focused systems",
                vec!["hardened", "pie", "ssp", "caps"],
            ),
        ),
        ("embedded", ("Embedded systems", vec!["static", "-ipv6"])),
        (
            "container",
            ("Container environments", vec!["static", "-pam", "-systemd"]),
        ),
    ]
    .into_iter()
    .collect();

    if let Some((description, flags)) = profiles.get(profile) {
        println!(
            "{}",
            style(format!("Profile: {}", profile)).bold().underlined()
        );
        println!();
        println!("  {}: {}", style("Description").bold(), description);
        println!("  {}: {}", style("USE flags").bold(), flags.join(" "));

        // Show package set for this profile
        let set_name = match profile {
            "minimal" | "embedded" | "container" => "minimal",
            "server" | "hardened" => "server",
            "desktop" | "developer" => "desktop",
            _ => "server",
        };

        let packages = get_set_packages(set_name);
        println!();
        println!("  {}:", style("Base packages").bold());
        for pkg in packages.iter().take(5) {
            println!("    {}", pkg);
        }
        if packages.len() > 5 {
            println!("    ... and {} more", packages.len() - 5);
        }
    } else {
        println!(
            "{} Unknown profile: {}",
            style(">>>").yellow().bold(),
            profile
        );
    }

    Ok(())
}

/// Set the active profile
async fn cmd_profile_set(profile: &str) -> buckos_package::Result<()> {
    let valid_profiles = [
        "minimal",
        "server",
        "desktop",
        "developer",
        "hardened",
        "embedded",
        "container",
    ];

    if !valid_profiles.contains(&profile) {
        println!(
            "{} Unknown profile: {}",
            style(">>>").yellow().bold(),
            profile
        );
        println!("Valid profiles: {}", valid_profiles.join(", "));
        return Ok(());
    }

    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/etc/buckos"))
        .join("buckos")
        .join("profile");

    // Create config directory
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Write profile
    fs::write(&config_path, profile)?;

    println!(
        "{} Profile set to: {}",
        style(">>>").green().bold(),
        profile
    );
    println!();
    println!("Run 'buckos update @world' to apply profile changes");

    Ok(())
}

/// Show current profile
async fn cmd_profile_current() -> buckos_package::Result<()> {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/etc/buckos"))
        .join("buckos")
        .join("profile");

    let profile = if config_path.exists() {
        fs::read_to_string(&config_path).unwrap_or_else(|_| "default".to_string())
    } else {
        "default".to_string()
    };

    println!(
        "{}: {}",
        style("Current profile").bold(),
        style(profile.trim()).green()
    );

    Ok(())
}

/// Export configuration in various formats
async fn cmd_export(args: ExportArgs) -> buckos_package::Result<()> {
    let config = Config::default();

    // Build configuration structure
    let use_flags: Vec<String> = vec![
        "ssl".to_string(),
        "http2".to_string(),
        "ipv6".to_string(),
        "zstd".to_string(),
        "dbus".to_string(),
    ];

    let output = match args.format.as_str() {
        "json" => {
            let mut export = serde_json::json!({
                "profile": "default",
                "arch": config.arch,
                "use_flags": {
                    "global": use_flags,
                    "package": {}
                },
                "env": {
                    "CFLAGS": config.cflags,
                    "CXXFLAGS": config.cxxflags,
                    "LDFLAGS": config.ldflags,
                    "MAKEOPTS": config.makeopts
                },
                "accept_keywords": [],
                "package_mask": [],
                "package_unmask": []
            });

            if args.with_packages {
                // Would include installed packages here
                export["packages"] = serde_json::json!({
                    "installed": []
                });
            }

            serde_json::to_string_pretty(&export).unwrap_or_default()
        }
        "toml" => {
            let mut output = String::new();
            output.push_str("# BuckOS Configuration Export\n\n");
            output.push_str("[profile]\n");
            output.push_str("name = \"default\"\n");
            output.push_str(&format!("arch = \"{}\"\n\n", config.arch));

            output.push_str("[use_flags]\n");
            output.push_str(&format!("global = {:?}\n\n", use_flags));

            output.push_str("[env]\n");
            output.push_str(&format!("CFLAGS = \"{}\"\n", config.cflags));
            output.push_str(&format!("CXXFLAGS = \"{}\"\n", config.cxxflags));
            output.push_str(&format!("LDFLAGS = \"{}\"\n", config.ldflags));
            output.push_str(&format!("MAKEOPTS = \"{}\"\n", config.makeopts));

            output
        }
        "shell" => {
            let mut output = String::new();
            output.push_str("#!/bin/bash\n");
            output.push_str("# BuckOS Configuration Export\n\n");

            output.push_str("export PROFILE=\"default\"\n");
            output.push_str(&format!("export ARCH=\"{}\"\n\n", config.arch));

            output.push_str("# USE flags\n");
            output.push_str(&format!("export USE=\"{}\"\n\n", use_flags.join(" ")));

            output.push_str("# Environment\n");
            output.push_str(&format!("export CFLAGS=\"{}\"\n", config.cflags));
            output.push_str(&format!("export CXXFLAGS=\"{}\"\n", config.cxxflags));
            output.push_str(&format!("export LDFLAGS=\"{}\"\n", config.ldflags));
            output.push_str(&format!("export MAKEOPTS=\"{}\"\n", config.makeopts));

            output
        }
        "buck" => {
            let mut output = String::new();
            output.push_str("# BuckOS Configuration Export\n\n");

            output.push_str("PROFILE = \"default\"\n");
            output.push_str(&format!("ARCH = \"{}\"\n\n", config.arch));

            output.push_str(&format!("USE_FLAGS = {:?}\n\n", use_flags));

            output.push_str("PACKAGE_USE = {}\n\n");

            output.push_str("ENV = {\n");
            output.push_str(&format!("    \"CFLAGS\": \"{}\",\n", config.cflags));
            output.push_str(&format!("    \"CXXFLAGS\": \"{}\",\n", config.cxxflags));
            output.push_str(&format!("    \"LDFLAGS\": \"{}\",\n", config.ldflags));
            output.push_str(&format!("    \"MAKEOPTS\": \"{}\",\n", config.makeopts));
            output.push_str("}\n");

            output
        }
        _ => {
            return Err(buckos_package::Error::ConfigError(format!(
                "Unknown format: {}. Use json, toml, shell, or buck",
                args.format
            )));
        }
    };

    // Write to file or stdout
    if let Some(path) = args.output {
        fs::write(&path, &output)?;
        println!(
            "{} Configuration exported to: {}",
            style(">>>").green().bold(),
            path
        );
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Rebuild packages with broken library dependencies
async fn cmd_revdep(
    pm: &PackageManager,
    args: RevdepArgs,
    emerge_opts: &EmergeOptions,
) -> buckos_package::Result<()> {
    println!(
        "{} Checking for packages with broken library dependencies...",
        style(">>>").blue().bold()
    );

    // Find packages with broken dependencies
    let broken = pm
        .find_broken_deps(args.library.as_deref(), &args.packages)
        .await?;

    // Filter out ignored packages
    let to_rebuild: Vec<_> = broken
        .into_iter()
        .filter(|pkg| {
            !args.ignore.contains(&pkg.name) && !args.ignore.contains(&pkg.id.full_name())
        })
        .collect();

    if to_rebuild.is_empty() {
        println!(
            "\n{} No packages with broken dependencies found",
            style(">>>").green().bold()
        );
        return Ok(());
    }

    // Display packages to rebuild
    println!(
        "\n{} Found {} package(s) with broken dependencies:\n",
        style(">>>").yellow().bold(),
        to_rebuild.len()
    );

    for pkg in &to_rebuild {
        println!(
            "  {} {}/{}",
            style("R").yellow().bold(),
            style(&pkg.id.category).cyan(),
            style(format!("{}-{}", &pkg.name, &pkg.version)).yellow()
        );

        // Show broken libraries
        if !pkg.broken_libs.is_empty() {
            for lib in &pkg.broken_libs {
                println!(
                    "      {} Missing library: {}",
                    style("->").dim(),
                    style(lib).red()
                );
            }
        }
    }

    println!(
        "\n>>> Rebuilding {} package(s)...",
        style(to_rebuild.len()).bold()
    );

    // Pretend mode
    if args.pretend || emerge_opts.pretend {
        return Ok(());
    }

    // Ask mode
    if emerge_opts.ask {
        if !Confirm::new()
            .with_prompt("Would you like to rebuild these packages?")
            .default(true)
            .interact()?
        {
            println!("{}", style(">>> Exiting.").yellow().bold());
            return Ok(());
        }
        println!();
    }

    // Actually rebuild
    pm.rebuild_packages(&to_rebuild).await?;

    println!(
        "\n{} {} packages rebuilt successfully",
        style(">>>").green().bold(),
        to_rebuild.len()
    );

    Ok(())
}

/// Package signing management
async fn cmd_sign(args: SignArgs) -> buckos_package::Result<()> {
    use buckos_package::security::signing::{
        format_key, format_verification, SigningManager, TrustLevel,
    };

    let mut manager = SigningManager::new()?;

    // Check if GPG is available
    if !manager.is_gpg_available() {
        return Err(buckos_package::Error::Signing(
            "GPG is not available. Please install gnupg.".to_string(),
        ));
    }

    match args.subcommand {
        SignCommand::ListKeys { secret } => {
            println!("{}", style("Available Signing Keys").bold().underlined());
            println!();

            let keys = manager.list_keys(secret)?;

            if keys.is_empty() {
                println!("  No keys found");
                if secret {
                    println!("\n  To create a new key: gpg --gen-key");
                }
            } else {
                for key in keys {
                    let key_type = if key.is_secret { "sec" } else { "pub" };
                    println!(
                        "  {} {}/{} {}",
                        style(key_type).dim(),
                        key.algorithm,
                        key.key_size,
                        key.created
                    );
                    println!("        {} {}", style("Key ID:").bold(), key.key_id);
                    println!("        {} {}", style("User:").bold(), key.user_id);
                    println!("        {} {}", style("Trust:").bold(), key.trust);
                    if let Some(ref expires) = key.expires {
                        println!("        {} {}", style("Expires:").bold(), expires);
                    }
                    println!();
                }
            }
        }

        SignCommand::ImportKey { source } => {
            println!(
                "{} Importing key from {}...",
                style(">>>").blue().bold(),
                source
            );

            let result = manager.import_key(&source)?;
            println!("{}", result);

            println!("{} Key imported successfully", style(">>>").green().bold());
        }

        SignCommand::ExportKey {
            key_id,
            output,
            armor,
        } => {
            println!(
                "{} Exporting key {} to {}...",
                style(">>>").blue().bold(),
                key_id,
                output
            );

            manager.export_key(&key_id, std::path::Path::new(&output), armor)?;

            println!("{} Key exported successfully", style(">>>").green().bold());
        }

        SignCommand::SignManifest { package_dir, key } => {
            println!(
                "{} Signing manifest in {}...",
                style(">>>").blue().bold(),
                package_dir
            );

            let path = std::path::Path::new(&package_dir);
            if !path.exists() {
                return Err(buckos_package::Error::Signing(format!(
                    "Directory not found: {}",
                    package_dir
                )));
            }

            // Extract package ID from directory name
            let pkg_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let category = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let package_id = buckos_package::PackageId::new(category, pkg_name);

            // Generate manifest
            let mut manifest = manager.generate_manifest(path, &package_id)?;

            // Sign it
            manager.sign_manifest(&mut manifest, key.as_deref())?;

            // Write to file
            let manifest_path = path.join("Manifest");
            manager.write_manifest(&manifest, &manifest_path)?;

            println!(
                "{} Manifest signed and written to {}",
                style(">>>").green().bold(),
                manifest_path.display()
            );
        }

        SignCommand::VerifyManifest { manifest } => {
            println!(
                "{} Verifying manifest {}...",
                style(">>>").blue().bold(),
                manifest
            );

            let path = std::path::Path::new(&manifest);
            let pkg_manifest = manager.read_manifest(path)?;

            let verification = manager.verify_manifest(&pkg_manifest)?;
            println!("\n{}", format_verification(&verification));

            if verification.valid {
                // Also verify files
                if let Some(parent) = path.parent() {
                    let file_results = manager.verify_manifest_files(&pkg_manifest, parent)?;
                    let failed: Vec<_> = file_results
                        .iter()
                        .filter(|r| {
                            r.status != buckos_package::security::signing::ManifestVerifyStatus::Ok
                        })
                        .collect();

                    if failed.is_empty() {
                        println!(
                            "{} All {} files verified",
                            style(">>>").green().bold(),
                            file_results.len()
                        );
                    } else {
                        println!(
                            "{} {} file(s) failed verification:",
                            style(">>>").red().bold(),
                            failed.len()
                        );
                        for result in failed {
                            println!("  {} {}: {}", style("!").red(), result.path, result.message);
                        }
                    }
                }
            } else {
                println!(
                    "{} Signature verification failed",
                    style(">>>").red().bold()
                );
            }
        }

        SignCommand::SignRepo { repo_dir, key } => {
            println!(
                "{} Signing repository {}...",
                style(">>>").blue().bold(),
                repo_dir
            );

            let path = std::path::Path::new(&repo_dir);
            manager.sign_repository(path, key.as_deref())?;

            println!(
                "{} Repository signed successfully",
                style(">>>").green().bold()
            );
        }

        SignCommand::VerifyRepo { repo_dir } => {
            println!(
                "{} Verifying repository {}...",
                style(">>>").blue().bold(),
                repo_dir
            );

            let path = std::path::Path::new(&repo_dir);
            let verification = manager.verify_repository(path)?;

            println!("\n{}", format_verification(&verification));

            if verification.valid {
                println!(
                    "{} Repository signature verified",
                    style(">>>").green().bold()
                );
            } else {
                println!(
                    "{} Repository signature verification failed",
                    style(">>>").red().bold()
                );
            }
        }

        SignCommand::SignFile { file, key } => {
            println!("{} Signing file {}...", style(">>>").blue().bold(), file);

            let path = std::path::Path::new(&file);
            let sig_path = manager.sign_file(path, key.as_deref())?;

            println!(
                "{} File signed: {}",
                style(">>>").green().bold(),
                sig_path.display()
            );
        }

        SignCommand::VerifyFile { file, signature } => {
            println!("{} Verifying file {}...", style(">>>").blue().bold(), file);

            let path = std::path::Path::new(&file);
            let sig_path = signature.map(|s| std::path::PathBuf::from(s));

            let verification = manager.verify_file(path, sig_path.as_deref())?;
            println!("\n{}", format_verification(&verification));

            if verification.valid {
                println!("{} Signature verified", style(">>>").green().bold());
            } else {
                println!(
                    "{} Signature verification failed",
                    style(">>>").red().bold()
                );
            }
        }

        SignCommand::KeyInfo { key_id } => match manager.get_key(&key_id)? {
            Some(key) => {
                println!("{}", style("Key Information").bold().underlined());
                println!();
                print!("{}", format_key(&key));
            }
            None => {
                println!("{} Key not found: {}", style(">>>").yellow().bold(), key_id);
            }
        },

        SignCommand::SetTrust { key_id, trust } => {
            let trust_level = match trust.to_lowercase().as_str() {
                "unknown" => TrustLevel::Unknown,
                "never" => TrustLevel::Never,
                "marginal" => TrustLevel::Marginal,
                "full" => TrustLevel::Full,
                "ultimate" => TrustLevel::Ultimate,
                _ => {
                    return Err(buckos_package::Error::Signing(format!(
                        "Invalid trust level: {}. Use: unknown, never, marginal, full, ultimate",
                        trust
                    )));
                }
            };

            println!(
                "{} Setting trust level for {} to {}...",
                style(">>>").blue().bold(),
                key_id,
                trust
            );

            manager.set_key_trust(&key_id, trust_level)?;

            println!("{} Trust level updated", style(">>>").green().bold());
        }
    }

    Ok(())
}

/// Handle overlay commands
async fn cmd_overlay(args: OverlayArgs) -> buckos_package::Result<()> {
    let config = OverlayConfig::default();
    let mut manager = OverlayManager::new(config)?;

    match args.subcommand {
        OverlayCommand::List { enabled, all } => {
            let overlays = if enabled {
                manager.list_enabled()
            } else if all {
                manager.list_all()
            } else {
                // Default: show all overlays with status
                manager.list_all()
            };

            if overlays.is_empty() {
                println!("{} No overlays configured", style(">>>").yellow().bold());
                return Ok(());
            }

            println!("{}", style("Configured Overlays").bold().underlined());
            println!();

            for overlay in overlays {
                let status = if overlay.enabled {
                    style("*").green().bold()
                } else {
                    style(" ").dim()
                };

                let quality = match overlay.quality {
                    OverlayQuality::Official => style("[official]").green(),
                    OverlayQuality::Community => style("[community]").blue(),
                    OverlayQuality::Experimental => style("[experimental]").yellow(),
                    OverlayQuality::Local => style("[local]").cyan(),
                };

                println!(
                    " {} {} {} (priority: {}) {}",
                    status,
                    style(&overlay.name).bold(),
                    quality,
                    overlay.priority,
                    if overlay.enabled { "" } else { "(disabled)" }
                );

                if !overlay.description.is_empty() {
                    println!("     {}", style(&overlay.description).dim());
                }
            }

            println!();
            println!("{} * = enabled overlay", style("Legend:").dim());
        }

        OverlayCommand::Add {
            name,
            uri,
            sync_type,
            priority,
            location,
        } => {
            let sync_type_enum = match sync_type.to_lowercase().as_str() {
                "git" => SyncType::Git,
                "rsync" => SyncType::Rsync,
                "http" | "https" => SyncType::Http,
                "local" => SyncType::Local,
                _ => {
                    return Err(buckos_package::Error::InvalidOverlayConfig(format!(
                        "Unknown sync type: {}. Use: git, rsync, http, local",
                        sync_type
                    )));
                }
            };

            if sync_type_enum == SyncType::Local {
                let path = location.ok_or_else(|| {
                    buckos_package::Error::InvalidOverlayConfig(
                        "Local overlays require --location".to_string(),
                    )
                })?;

                println!(
                    "{} Adding local overlay {}...",
                    style(">>>").blue().bold(),
                    name
                );

                manager.add_local(&name, std::path::Path::new(&path), priority)?;
            } else {
                let uri = uri.ok_or_else(|| {
                    buckos_package::Error::InvalidOverlayConfig(
                        "Remote overlays require --uri".to_string(),
                    )
                })?;

                println!(
                    "{} Adding overlay {} from {}...",
                    style(">>>").blue().bold(),
                    name,
                    uri
                );

                manager.add_remote(&name, &uri, sync_type_enum, priority)?;
            }

            println!(
                "{} Overlay {} added successfully",
                style(">>>").green().bold(),
                name
            );
            println!("  Use 'buckos overlay enable {}' to enable it", name);
        }

        OverlayCommand::Remove { name, delete } => {
            println!(
                "{} Removing overlay {}...",
                style(">>>").blue().bold(),
                name
            );

            manager.remove(&name, delete)?;

            println!(
                "{} Overlay {} removed",
                style(">>>").green().bold(),
                name
            );
        }

        OverlayCommand::Enable { name } => {
            println!(
                "{} Enabling overlay {}...",
                style(">>>").blue().bold(),
                name
            );

            manager.enable(&name)?;

            println!(
                "{} Overlay {} enabled",
                style(">>>").green().bold(),
                name
            );
        }

        OverlayCommand::Disable { name } => {
            println!(
                "{} Disabling overlay {}...",
                style(">>>").blue().bold(),
                name
            );

            manager.disable(&name)?;

            println!(
                "{} Overlay {} disabled",
                style(">>>").green().bold(),
                name
            );
        }

        OverlayCommand::Sync { name } => {
            if let Some(name) = name {
                println!(
                    "{} Syncing overlay {}...",
                    style(">>>").blue().bold(),
                    name
                );

                manager.sync(&name).await?;

                println!(
                    "{} Overlay {} synced",
                    style(">>>").green().bold(),
                    name
                );
            } else {
                println!(
                    "{} Syncing all enabled overlays...",
                    style(">>>").blue().bold()
                );

                manager.sync_all().await?;

                println!(
                    "{} All overlays synced",
                    style(">>>").green().bold()
                );
            }
        }

        OverlayCommand::Info { name } => {
            match manager.get_info(&name) {
                Some(overlay) => {
                    println!("{}", style("Overlay Information").bold().underlined());
                    println!();
                    println!("  {} {}", style("Name:").bold(), overlay.name);
                    println!("  {} {}", style("Description:").bold(), overlay.description);
                    println!("  {} {:?}", style("Sync Type:").bold(), overlay.sync_type);
                    println!("  {} {}", style("Sync URI:").bold(), overlay.sync_uri);
                    println!("  {} {}", style("Location:").bold(), overlay.location.display());
                    println!("  {} {}", style("Priority:").bold(), overlay.priority);
                    println!("  {} {}", style("Quality:").bold(), overlay.quality);
                    println!(
                        "  {} {}",
                        style("Status:").bold(),
                        if overlay.enabled { "enabled" } else { "disabled" }
                    );
                    println!(
                        "  {} {}",
                        style("Auto-sync:").bold(),
                        if overlay.auto_sync { "yes" } else { "no" }
                    );
                    if let Some(owner) = &overlay.owner {
                        println!("  {} {}", style("Owner:").bold(), owner);
                    }
                    if let Some(homepage) = &overlay.homepage {
                        println!("  {} {}", style("Homepage:").bold(), homepage);
                    }
                    if !overlay.masters.is_empty() {
                        println!("  {} {}", style("Masters:").bold(), overlay.masters.join(", "));
                    }
                    if let Some(last_sync) = overlay.last_sync {
                        let datetime = chrono::DateTime::from_timestamp(last_sync as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        println!("  {} {}", style("Last Sync:").bold(), datetime);
                    }
                }
                None => {
                    println!(
                        "{} Overlay not found: {}",
                        style(">>>").yellow().bold(),
                        name
                    );
                }
            }
        }

        OverlayCommand::Priority { name, priority } => {
            println!(
                "{} Setting priority for {} to {}...",
                style(">>>").blue().bold(),
                name,
                priority
            );

            manager.set_priority(&name, priority)?;

            println!(
                "{} Priority updated",
                style(">>>").green().bold()
            );
        }

        OverlayCommand::Search { query } => {
            let results = manager.search(&query);

            if results.is_empty() {
                println!(
                    "{} No overlays found matching '{}'",
                    style(">>>").yellow().bold(),
                    query
                );
                return Ok(());
            }

            println!(
                "{} Found {} overlay(s) matching '{}'",
                style(">>>").green().bold(),
                results.len(),
                query
            );
            println!();

            for overlay in results {
                let quality = match overlay.quality {
                    OverlayQuality::Official => style("[official]").green(),
                    OverlayQuality::Community => style("[community]").blue(),
                    OverlayQuality::Experimental => style("[experimental]").yellow(),
                    OverlayQuality::Local => style("[local]").cyan(),
                };

                println!("  {} {}", style(&overlay.name).bold(), quality);
                if !overlay.description.is_empty() {
                    println!("    {}", style(&overlay.description).dim());
                }
            }
        }
    }

    Ok(())
}
