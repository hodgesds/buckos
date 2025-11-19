//! Buckos Package Manager CLI
//!
//! Command-line interface for the Buckos package manager.
//! Designed to be compatible with Gentoo's emerge command.

use clap::{Args, Parser, Subcommand};
use console::style;
use dialoguer::Confirm;
use buckos_package::{
    BuildOptions, CleanOptions, Config, DepcleanOptions, EmergeOptions, InstallOptions,
    PackageManager, RemoveOptions, Resolution, SyncOptions, UpdateOptions,
};
use std::collections::HashSet;
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
        Some(path) => {
            match Config::load_from(std::path::Path::new(&path)) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to load config: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }
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
        println!(
            "{} No packages to unmerge",
            style(">>>").yellow().bold()
        );
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
            println!(
                "\n{} @world set is up-to-date",
                style(">>>").green().bold()
            );
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
        QueryType::Rdeps { package: _ } => {
            println!("Reverse dependencies not yet implemented");
        }
    }

    Ok(())
}

async fn cmd_owner(_pm: &PackageManager, args: OwnerArgs) -> buckos_package::Result<()> {
    println!("Searching for owner of: {}", args.path);
    println!("File owner query not yet implemented");
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
            style("R").yellow().bold()  // Rebuild
        } else if pkg.is_upgrade {
            style("U").blue().bold()  // Update
        } else {
            style("N").green().bold()  // New
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
    println!(
        "{} Calculating dependencies...",
        style(">>>").blue().bold()
    );

    let opts = DepcleanOptions {
        packages: args.packages.clone(),
        pretend: args.pretend || emerge_opts.pretend,
    };

    let to_remove = pm.calculate_depclean(&opts).await?;

    if to_remove.is_empty() {
        println!(
            "{} No packages to depclean",
            style(">>>").green().bold()
        );
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
    println!(
        ">>> Space freed: {}",
        style(format_size(total_size)).cyan()
    );

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
    let pkg_names: Vec<String> = to_rebuild
        .iter()
        .map(|p| p.id.full_name())
        .collect();

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
