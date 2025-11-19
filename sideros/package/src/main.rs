//! Sideros Package Manager CLI
//!
//! Command-line interface for the Sideros package manager.

use clap::{Args, Parser, Subcommand};
use console::style;
use sideros_package::{
    BuildOptions, CleanOptions, Config, InstallOptions, PackageManager, RemoveOptions,
    UpdateOptions,
};
use std::process::ExitCode;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "sideros-pkg",
    about = "Sideros Package Manager - A scalable Buck-based package manager",
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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages
    Install(InstallArgs),

    /// Remove packages
    Remove(RemoveArgs),

    /// Update packages
    Update(UpdateArgs),

    /// Sync package repositories
    Sync,

    /// Search for packages
    Search(SearchArgs),

    /// Show package information
    Info(InfoArgs),

    /// List installed packages
    List(ListArgs),

    /// Build a package from source
    Build(BuildArgs),

    /// Clean cache
    Clean(CleanArgs),

    /// Verify installed packages
    Verify,

    /// Query package database
    Query(QueryArgs),

    /// Show package that owns a file
    Owner(OwnerArgs),

    /// Show dependency tree
    Depgraph(DepgraphArgs),

    /// Show configuration
    Config,
}

#[derive(Args)]
struct InstallArgs {
    /// Packages to install
    #[arg(required = true)]
    packages: Vec<String>,

    /// Force reinstall even if already installed
    #[arg(short, long)]
    force: bool,

    /// Don't install dependencies
    #[arg(long)]
    no_deps: bool,

    /// Build from source
    #[arg(short, long)]
    build: bool,

    /// USE flags to enable
    #[arg(long, value_delimiter = ',')]
    use_flags: Vec<String>,
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
    /// Packages to update (all if not specified)
    packages: Vec<String>,

    /// Don't sync repositories first
    #[arg(long)]
    no_sync: bool,

    /// Only check for updates
    #[arg(short, long)]
    check: bool,
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

    // Execute command
    let result = match cli.command {
        Commands::Install(args) => cmd_install(&pkg_manager, args).await,
        Commands::Remove(args) => cmd_remove(&pkg_manager, args).await,
        Commands::Update(args) => cmd_update(&pkg_manager, args).await,
        Commands::Sync => cmd_sync(&pkg_manager).await,
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
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{}", e);
            ExitCode::FAILURE
        }
    }
}

async fn cmd_install(pm: &PackageManager, args: InstallArgs) -> sideros_package::Result<()> {
    println!(
        "{} Installing {} package(s)...",
        style(">>>").green().bold(),
        args.packages.len()
    );

    let opts = InstallOptions {
        force: args.force,
        no_deps: args.no_deps,
        build: args.build,
        use_flags: args.use_flags,
    };

    pm.install(&args.packages, opts).await?;

    println!("{} Installation complete", style(">>>").green().bold());

    Ok(())
}

async fn cmd_remove(pm: &PackageManager, args: RemoveArgs) -> sideros_package::Result<()> {
    println!(
        "{} Removing {} package(s)...",
        style(">>>").yellow().bold(),
        args.packages.len()
    );

    let opts = RemoveOptions {
        force: args.force,
        recursive: args.recursive,
    };

    pm.remove(&args.packages, opts).await?;

    println!("{} Removal complete", style(">>>").green().bold());

    Ok(())
}

async fn cmd_update(pm: &PackageManager, args: UpdateArgs) -> sideros_package::Result<()> {
    println!("{} Checking for updates...", style(">>>").blue().bold());

    let opts = UpdateOptions {
        sync: !args.no_sync,
        check_only: args.check,
    };

    let packages = if args.packages.is_empty() {
        None
    } else {
        Some(args.packages.as_slice())
    };

    pm.update(packages, opts).await?;

    Ok(())
}

async fn cmd_sync(pm: &PackageManager) -> sideros_package::Result<()> {
    println!("{} Syncing repositories...", style(">>>").blue().bold());
    pm.sync().await?;
    println!("{} Sync complete", style(">>>").green().bold());
    Ok(())
}

async fn cmd_search(pm: &PackageManager, args: SearchArgs) -> sideros_package::Result<()> {
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

async fn cmd_info(pm: &PackageManager, args: InfoArgs) -> sideros_package::Result<()> {
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

async fn cmd_list(pm: &PackageManager, args: ListArgs) -> sideros_package::Result<()> {
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

async fn cmd_build(pm: &PackageManager, args: BuildArgs) -> sideros_package::Result<()> {
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

async fn cmd_clean(pm: &PackageManager, args: CleanArgs) -> sideros_package::Result<()> {
    let opts = CleanOptions {
        all: args.all,
        downloads: args.downloads,
        builds: args.builds,
    };

    pm.clean(opts).await?;
    println!("{} Cache cleaned", style(">>>").green().bold());

    Ok(())
}

async fn cmd_verify(pm: &PackageManager) -> sideros_package::Result<()> {
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

async fn cmd_query(pm: &PackageManager, args: QueryArgs) -> sideros_package::Result<()> {
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

async fn cmd_owner(_pm: &PackageManager, args: OwnerArgs) -> sideros_package::Result<()> {
    println!("Searching for owner of: {}", args.path);
    println!("File owner query not yet implemented");
    Ok(())
}

async fn cmd_depgraph(pm: &PackageManager, args: DepgraphArgs) -> sideros_package::Result<()> {
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

async fn cmd_config() -> sideros_package::Result<()> {
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
