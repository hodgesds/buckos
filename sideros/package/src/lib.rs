//! Sideros Package Manager
//!
//! A scalable, Buck-based package manager for the Sideros Linux distribution.
//!
//! # Architecture
//!
//! The package manager is built around several core components:
//!
//! - **Database**: SQLite-based local package database for tracking installed packages
//! - **Buck Integration**: Interfaces with Buck2 build system for package builds
//! - **Resolver**: SAT solver-based dependency resolution
//! - **Executor**: Parallel execution engine for scalable operations
//! - **Transaction**: Atomic package operations with rollback support
//! - **Cache**: Download and build artifact caching
//! - **Repository**: Package repository management

pub mod buck;
pub mod cache;
pub mod catalog;
pub mod config;
pub mod db;
pub mod error;
pub mod executor;
pub mod repository;
pub mod resolver;
pub mod transaction;
pub mod types;
pub mod validation;

pub use config::Config;
pub use error::{Error, Result};
pub use types::*;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Main package manager instance
pub struct PackageManager {
    /// Configuration
    config: config::Config,
    /// Package database
    db: Arc<RwLock<db::PackageDb>>,
    /// Build cache
    cache: Arc<cache::PackageCache>,
    /// Repository manager
    repos: Arc<repository::RepositoryManager>,
    /// Buck integration
    buck: Arc<buck::BuckIntegration>,
    /// Parallel executor
    executor: Arc<executor::ParallelExecutor>,
}

impl PackageManager {
    /// Create a new package manager instance
    pub async fn new(config: config::Config) -> Result<Self> {
        info!("Initializing Sideros package manager");

        // Initialize database
        let db_path = config.db_path.clone();
        let db = db::PackageDb::open(&db_path)?;
        let db = Arc::new(RwLock::new(db));

        // Initialize cache
        let cache = cache::PackageCache::new(&config.cache_dir)?;
        let cache = Arc::new(cache);

        // Initialize repository manager
        let repos = repository::RepositoryManager::new(&config)?;
        let repos = Arc::new(repos);

        // Initialize Buck integration
        let buck = buck::BuckIntegration::new(&config)?;
        let buck = Arc::new(buck);

        // Initialize parallel executor
        let executor = executor::ParallelExecutor::new(config.parallelism);
        let executor = Arc::new(executor);

        Ok(Self {
            config,
            db,
            cache,
            repos,
            buck,
            executor,
        })
    }

    /// Install packages
    pub async fn install(&self, packages: &[String], opts: InstallOptions) -> Result<()> {
        info!("Installing packages: {:?}", packages);

        // Resolve dependencies
        let resolver = resolver::DependencyResolver::new(
            self.db.clone(),
            self.repos.clone(),
        );

        let resolution = resolver.resolve(packages, &opts).await?;

        if resolution.packages.is_empty() {
            info!("All packages are already installed");
            return Ok(());
        }

        // Create transaction
        let mut transaction = transaction::Transaction::new(
            self.db.clone(),
            self.cache.clone(),
            self.buck.clone(),
        );

        // Add install operations
        for pkg in &resolution.packages {
            transaction.add_install(pkg.clone());
        }

        // Execute transaction
        transaction.execute(&self.executor).await?;

        info!("Successfully installed {} packages", resolution.packages.len());
        Ok(())
    }

    /// Remove packages
    pub async fn remove(&self, packages: &[String], opts: RemoveOptions) -> Result<()> {
        info!("Removing packages: {:?}", packages);

        let db = self.db.read().await;

        // Check that all packages are installed
        let mut to_remove = Vec::new();
        for pkg_name in packages {
            if let Some(pkg) = db.get_installed(pkg_name)? {
                to_remove.push(pkg);
            } else {
                return Err(Error::PackageNotInstalled(pkg_name.clone()));
            }
        }
        drop(db);

        // Check for reverse dependencies if not forced
        if !opts.force {
            let db = self.db.read().await;
            for pkg in &to_remove {
                let rdeps = db.get_reverse_dependencies(&pkg.name)?;
                if !rdeps.is_empty() {
                    return Err(Error::HasDependents {
                        package: pkg.name.clone(),
                        dependents: rdeps,
                    });
                }
            }
        }

        // Create transaction
        let mut transaction = transaction::Transaction::new(
            self.db.clone(),
            self.cache.clone(),
            self.buck.clone(),
        );

        // Add remove operations
        for pkg in to_remove {
            transaction.add_remove(pkg);
        }

        // Execute transaction
        transaction.execute(&self.executor).await?;

        info!("Successfully removed {} packages", packages.len());
        Ok(())
    }

    /// Update packages
    pub async fn update(&self, packages: Option<&[String]>, opts: UpdateOptions) -> Result<()> {
        // Sync repositories first
        if opts.sync {
            self.sync().await?;
        }

        let db = self.db.read().await;

        // Get packages to update
        let to_check: Vec<InstalledPackage> = match packages {
            Some(names) => {
                let mut pkgs = Vec::new();
                for name in names {
                    if let Some(pkg) = db.get_installed(name)? {
                        pkgs.push(pkg);
                    }
                }
                pkgs
            }
            None => db.get_all_installed()?,
        };
        drop(db);

        // Find available updates
        let mut updates = Vec::new();
        for pkg in to_check {
            if let Some(available) = self.repos.get_latest(&pkg.name).await? {
                if available.version > pkg.version {
                    updates.push((pkg, available));
                }
            }
        }

        if updates.is_empty() {
            info!("All packages are up to date");
            return Ok(());
        }

        info!("Found {} updates", updates.len());

        // Create transaction
        let mut transaction = transaction::Transaction::new(
            self.db.clone(),
            self.cache.clone(),
            self.buck.clone(),
        );

        // Add upgrade operations
        for (old, new) in updates {
            transaction.add_upgrade(old, new);
        }

        // Execute transaction
        transaction.execute(&self.executor).await?;

        Ok(())
    }

    /// Sync package repositories
    pub async fn sync(&self) -> Result<()> {
        info!("Syncing package repositories");
        self.repos.sync_all().await?;
        Ok(())
    }

    /// Search for packages
    pub async fn search(&self, query: &str) -> Result<Vec<PackageInfo>> {
        self.repos.search(query).await
    }

    /// Get package information
    pub async fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        self.repos.get_info(package).await
    }

    /// List installed packages
    pub async fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let db = self.db.read().await;
        db.get_all_installed()
    }

    /// Build a package from source using Buck
    pub async fn build(&self, target: &str, opts: BuildOptions) -> Result<BuildResult> {
        info!("Building target: {}", target);
        self.buck.build(target, &opts).await
    }

    /// Clean build cache
    pub async fn clean(&self, opts: CleanOptions) -> Result<()> {
        if opts.all {
            self.cache.clean_all()?;
            self.buck.clean().await?;
        } else if opts.downloads {
            self.cache.clean_downloads()?;
        } else if opts.builds {
            self.buck.clean().await?;
        }
        Ok(())
    }

    /// Verify installed packages
    pub async fn verify(&self) -> Result<Vec<VerifyResult>> {
        let db = self.db.read().await;
        let installed = db.get_all_installed()?;
        drop(db);

        let mut results = Vec::new();
        for pkg in installed {
            let result = self.verify_package(&pkg).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn verify_package(&self, pkg: &InstalledPackage) -> Result<VerifyResult> {
        let db = self.db.read().await;
        let files = db.get_package_files(&pkg.name)?;
        drop(db);

        let mut missing = Vec::new();
        let mut modified = Vec::new();

        for file in files {
            let path = PathBuf::from(&file.path);
            if !path.exists() {
                missing.push(file.path.clone());
            } else if let Some(expected_hash) = &file.blake3_hash {
                let actual_hash = cache::compute_blake3(&path)?;
                if &actual_hash != expected_hash {
                    modified.push(file.path.clone());
                }
            }
        }

        let ok = missing.is_empty() && modified.is_empty();
        Ok(VerifyResult {
            package: pkg.name.clone(),
            missing,
            modified,
            ok,
        })
    }
}

/// Options for install command
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Force reinstall even if already installed
    pub force: bool,
    /// Don't install dependencies
    pub no_deps: bool,
    /// Build from source instead of using binary
    pub build: bool,
    /// Use flags
    pub use_flags: Vec<String>,
}

/// Options for remove command
#[derive(Debug, Clone, Default)]
pub struct RemoveOptions {
    /// Force removal even with dependents
    pub force: bool,
    /// Also remove dependencies that are no longer needed
    pub recursive: bool,
}

/// Options for update command
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// Sync repositories before updating
    pub sync: bool,
    /// Only check for updates, don't install
    pub check_only: bool,
}

/// Options for build command
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    /// Number of parallel jobs
    pub jobs: Option<usize>,
    /// Build in release mode
    pub release: bool,
    /// Additional Buck arguments
    pub buck_args: Vec<String>,
}

/// Options for clean command
#[derive(Debug, Clone, Default)]
pub struct CleanOptions {
    /// Clean everything
    pub all: bool,
    /// Clean only downloads
    pub downloads: bool,
    /// Clean only builds
    pub builds: bool,
}

/// Result of package verification
#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub package: String,
    pub missing: Vec<String>,
    pub modified: Vec<String>,
    pub ok: bool,
}
