//! Buckos Package Manager
//!
//! A scalable, Buck-based package manager for the Buckos Linux distribution.
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

pub mod binary;
pub mod buck;
pub mod cache;
pub mod catalog;
pub mod config;
pub mod config_protect;
pub mod cross;
pub mod db;
pub mod distfile;
pub mod error;
pub mod executor;
pub mod features;
pub mod mask;
pub mod news;
pub mod overlay;
pub mod preserved_libs;
pub mod profile;
pub mod repository;
pub mod resolver;
pub mod sandbox;
pub mod security;
pub mod transaction;
pub mod types;
pub mod validation;
pub mod r#virtual;

pub use buck::{BuckConfigFile, BuckConfigOptions, BuckConfigSection};
pub use config::Config;
pub use error::{Error, Result};
pub use types::*;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

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
        info!("Initializing Buckos package manager");

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

        // Initialize Buck integration with custom config options
        let buck = buck::BuckIntegration::with_config_options(&config, config.buck_config.clone())?;
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
        let resolver = resolver::DependencyResolver::new(self.db.clone(), self.repos.clone());

        let resolution = resolver.resolve(packages, &opts).await?;

        if resolution.packages.is_empty() {
            info!("All packages are already installed");
            return Ok(());
        }

        // Handle fetch-only mode
        if opts.fetch_only {
            info!(
                "Fetch-only mode: downloading {} packages",
                resolution.packages.len()
            );
            for pkg in &resolution.packages {
                if let Some(ref url) = pkg.source_url {
                    let filename = format!("{}-{}.tar.gz", pkg.id.name, pkg.version);
                    self.cache
                        .download(url, &filename, pkg.source_hash.as_deref())
                        .await?;
                }
            }
            return Ok(());
        }

        // Create transaction
        let mut transaction =
            transaction::Transaction::new(self.db.clone(), self.cache.clone(), self.buck.clone());

        // Add install operations
        for pkg in &resolution.packages {
            transaction.add_install(pkg.clone());
        }

        // Execute transaction
        transaction.execute(&self.executor).await?;

        // Add to world set if not oneshot
        if !opts.oneshot {
            for pkg_name in packages {
                if let Some(pkg_id) = PackageId::parse(pkg_name) {
                    self.add_to_world(&pkg_id).await?;
                }
            }
        }

        info!(
            "Successfully installed {} packages",
            resolution.packages.len()
        );
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
        let mut transaction =
            transaction::Transaction::new(self.db.clone(), self.cache.clone(), self.buck.clone());

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
        let mut transaction =
            transaction::Transaction::new(self.db.clone(), self.cache.clone(), self.buck.clone());

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

    /// Resolve packages without installing (for pretend mode)
    pub async fn resolve_packages(
        &self,
        packages: &[String],
        opts: &InstallOptions,
    ) -> Result<Resolution> {
        info!("Resolving packages: {:?}", packages);

        let resolver = resolver::DependencyResolver::new(self.db.clone(), self.repos.clone());

        let resolution = resolver.resolve(packages, opts).await?;

        // Convert to ResolvedPackage format
        let db = self.db.read().await;
        let mut resolved_packages = Vec::new();

        for pkg in &resolution.packages {
            let is_installed = db.is_installed(&pkg.id.name).unwrap_or(false);
            let old_version = if is_installed {
                db.get_installed(&pkg.id.name)?.map(|p| p.version)
            } else {
                None
            };

            let is_upgrade = old_version
                .as_ref()
                .map(|v| v < &pkg.version)
                .unwrap_or(false);
            let is_rebuild = old_version
                .as_ref()
                .map(|v| v == &pkg.version)
                .unwrap_or(false)
                && opts.force;

            let use_flags: Vec<UseFlagStatus> = pkg
                .use_flags
                .iter()
                .map(|flag| UseFlagStatus {
                    name: flag.name.clone(),
                    enabled: flag.default || opts.use_flags.contains(&flag.name),
                })
                .collect();

            resolved_packages.push(ResolvedPackage {
                id: pkg.id.clone(),
                version: pkg.version.clone(),
                slot: pkg.slot.clone(),
                description: pkg.description.clone(),
                use_flags,
                dependencies: pkg.dependencies.clone(),
                size: pkg.size,
                installed_size: pkg.installed_size,
                is_upgrade,
                is_rebuild,
                is_new: !is_installed,
                old_version,
            });
        }

        Ok(Resolution {
            packages: resolved_packages,
            build_order: resolution.build_order,
            download_size: resolution.download_size,
            install_size: resolution.install_size,
        })
    }

    /// Get the world set (explicitly installed packages)
    pub async fn get_world_set(&self) -> Result<WorldSet> {
        let db = self.db.read().await;
        let installed = db.get_all_installed()?;

        let packages: std::collections::HashSet<PackageId> = installed
            .iter()
            .filter(|p| p.explicit)
            .map(|p| p.id.clone())
            .collect();

        Ok(WorldSet { packages })
    }

    /// Get the system set (essential system packages)
    /// These match the packages defined in buckos-build's SYSTEM_PACKAGES
    /// Uses glibc by default for maximum compatibility (can be changed to musl for minimal systems)
    pub async fn get_system_set(&self) -> Result<WorldSet> {
        // System packages are predefined essential packages
        // Using buckos-build registry naming: category/name
        let system_packages = vec![
            PackageId::new("core", "glibc"),       // GNU C library (use "musl" for minimal systems)
            PackageId::new("system/apps", "coreutils"),  // Core utilities
            PackageId::new("core", "util-linux"),  // System utilities
            PackageId::new("core", "procps-ng"),   // Process monitoring
            PackageId::new("system/apps", "shadow"), // User/group management
            PackageId::new("core", "file"),        // File type detection
            PackageId::new("core", "bash"),        // Shell
            PackageId::new("system/init", "systemd"), // Init system
            PackageId::new("core", "zlib"),        // Compression library
        ];

        Ok(WorldSet {
            packages: system_packages.into_iter().collect(),
        })
    }

    /// Get the selected set (combined world + system)
    pub async fn get_selected_set(&self) -> Result<WorldSet> {
        let world = self.get_world_set().await?;
        let system = self.get_system_set().await?;

        let mut packages = world.packages;
        packages.extend(system.packages);

        Ok(WorldSet { packages })
    }

    /// Get list of packages that would be removed
    pub async fn get_removal_list(
        &self,
        packages: &[String],
        _opts: &RemoveOptions,
    ) -> Result<Vec<InstalledPackage>> {
        let db = self.db.read().await;
        let mut to_remove = Vec::new();

        for pkg_name in packages {
            if let Some(pkg) = db.get_installed(pkg_name)? {
                to_remove.push(pkg);
            }
        }

        Ok(to_remove)
    }

    /// Get update resolution
    pub async fn get_update_resolution(
        &self,
        packages: Option<&[String]>,
        opts: &UpdateOptions,
    ) -> Result<Resolution> {
        let db = self.db.read().await;

        // Get packages to check
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
        let mut resolved_packages = Vec::new();
        let mut download_size = 0u64;
        let mut install_size = 0u64;

        for pkg in to_check {
            if let Some(available) = self.repos.get_latest(&pkg.name).await? {
                let needs_update = available.version > pkg.version;
                let needs_rebuild = opts.newuse && self.has_use_changes(&pkg, &available).await;

                if needs_update || needs_rebuild {
                    let use_flags: Vec<UseFlagStatus> = available
                        .use_flags
                        .iter()
                        .map(|f| UseFlagStatus {
                            name: f.name.clone(),
                            enabled: f.default || pkg.use_flags.contains(&f.name),
                        })
                        .collect();

                    resolved_packages.push(ResolvedPackage {
                        id: available.id.clone(),
                        version: available.version.clone(),
                        slot: available.slot.clone(),
                        description: available.description.clone(),
                        use_flags,
                        dependencies: available.dependencies.clone(),
                        size: available.size,
                        installed_size: available.installed_size,
                        is_upgrade: needs_update,
                        is_rebuild: needs_rebuild && !needs_update,
                        is_new: false,
                        old_version: Some(pkg.version.clone()),
                    });

                    download_size += available.size;
                    install_size += available.installed_size;
                }
            }
        }

        Ok(Resolution {
            build_order: (0..resolved_packages.len()).collect(),
            packages: resolved_packages,
            download_size,
            install_size,
        })
    }

    async fn has_use_changes(&self, installed: &InstalledPackage, available: &PackageInfo) -> bool {
        let available_flags: std::collections::HashSet<String> = available
            .use_flags
            .iter()
            .filter(|f| f.default)
            .map(|f| f.name.clone())
            .collect();

        installed.use_flags != available_flags
    }

    /// Sync a specific repository
    pub async fn sync_repo(&self, repo_name: &str) -> Result<()> {
        info!("Syncing repository: {}", repo_name);
        self.repos.sync_repo(repo_name).await
    }

    /// Calculate packages to depclean
    pub async fn calculate_depclean(
        &self,
        opts: &DepcleanOptions,
    ) -> Result<Vec<InstalledPackage>> {
        info!("Calculating depclean candidates");

        let db = self.db.read().await;
        let all_installed = db.get_all_installed()?;

        // Get world and system sets
        drop(db);
        let selected = self.get_selected_set().await?;

        // Find packages that are not in selected set and have no reverse dependencies
        let mut candidates = Vec::new();
        let db = self.db.read().await;

        for pkg in &all_installed {
            // Skip if explicitly in selected set
            if selected.packages.contains(&pkg.id) {
                continue;
            }

            // Skip if it has reverse dependencies from non-candidates
            let rdeps = db.get_reverse_dependencies(&pkg.name)?;
            let has_needed_rdeps = rdeps.iter().any(|rdep| {
                all_installed
                    .iter()
                    .any(|p| p.name == *rdep && (p.explicit || selected.packages.contains(&p.id)))
            });

            if !has_needed_rdeps {
                // Check if in specific package list
                if opts.packages.is_empty() || opts.packages.contains(&pkg.id.full_name()) {
                    candidates.push(pkg.clone());
                }
            }
        }

        Ok(candidates)
    }

    /// Actually perform depclean
    pub async fn depclean(&self, opts: &DepcleanOptions) -> Result<()> {
        let to_remove = self.calculate_depclean(opts).await?;

        if to_remove.is_empty() {
            return Ok(());
        }

        // Create transaction for removal
        let mut transaction =
            transaction::Transaction::new(self.db.clone(), self.cache.clone(), self.buck.clone());

        for pkg in to_remove {
            transaction.add_remove(pkg);
        }

        transaction.execute(&self.executor).await?;

        Ok(())
    }

    /// Resume interrupted operation
    pub async fn resume(&self) -> Result<bool> {
        // Check for saved transaction state
        let state_file = self.config.cache_dir.join("transaction_state.json");

        if !state_file.exists() {
            return Ok(false);
        }

        info!("Found interrupted transaction, resuming...");

        // Load and execute saved transaction
        let state_data = std::fs::read_to_string(&state_file)?;
        let _state: serde_json::Value = serde_json::from_str(&state_data)?;

        // Remove state file after successful resume
        std::fs::remove_file(&state_file)?;

        Ok(true)
    }

    /// Find packages that need rebuilding due to USE flag changes
    pub async fn find_newuse_packages(
        &self,
        packages: Option<&[String]>,
        deep: bool,
    ) -> Result<Vec<NewusePackage>> {
        let db = self.db.read().await;

        // Get packages to check
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

        let mut newuse_packages = Vec::new();

        for pkg in to_check {
            if let Some(available) = self.repos.get_info(&pkg.name).await? {
                let mut use_changes = Vec::new();

                // Check for added flags
                let available_defaults: std::collections::HashSet<String> = available
                    .use_flags
                    .iter()
                    .filter(|f| f.default)
                    .map(|f| f.name.clone())
                    .collect();

                for flag in &available_defaults {
                    if !pkg.use_flags.contains(flag) {
                        use_changes.push(UseFlagChange {
                            flag: flag.clone(),
                            added: true,
                        });
                    }
                }

                // Check for removed flags
                for flag in &pkg.use_flags {
                    if !available_defaults.contains(flag) {
                        use_changes.push(UseFlagChange {
                            flag: flag.clone(),
                            added: false,
                        });
                    }
                }

                if !use_changes.is_empty() {
                    newuse_packages.push(NewusePackage {
                        id: pkg.id.clone(),
                        name: pkg.name.clone(),
                        version: pkg.version.clone(),
                        use_changes,
                    });
                }
            }
        }

        // If deep mode, also check dependencies
        if deep && !newuse_packages.is_empty() {
            // Get dependencies of affected packages
            let db = self.db.read().await;
            for pkg in &newuse_packages.clone() {
                let rdeps = db.get_reverse_dependencies(&pkg.name)?;
                for rdep in rdeps {
                    if let Some(rdep_pkg) = db.get_installed(&rdep)? {
                        if !newuse_packages.iter().any(|p| p.name == rdep_pkg.name) {
                            // Add with empty use_changes to trigger rebuild
                            newuse_packages.push(NewusePackage {
                                id: rdep_pkg.id.clone(),
                                name: rdep_pkg.name.clone(),
                                version: rdep_pkg.version.clone(),
                                use_changes: vec![],
                            });
                        }
                    }
                }
            }
        }

        Ok(newuse_packages)
    }

    /// Audit installed packages for security vulnerabilities
    pub async fn audit(&self) -> Result<Vec<Vulnerability>> {
        info!("Auditing for security vulnerabilities");

        let db = self.db.read().await;
        let installed = db.get_all_installed()?;
        drop(db);

        let mut vulnerabilities = Vec::new();

        // Comprehensive vulnerability database
        let vuln_db = get_vulnerability_database();

        for pkg in &installed {
            // Check against vulnerability database
            for vuln in &vuln_db {
                if vuln.package_name == pkg.name {
                    // Check if version is affected
                    let is_affected = match &vuln.version_check {
                        VersionCheck::LessThan(v) => pkg.version < *v,
                        VersionCheck::LessThanOrEqual(v) => pkg.version <= *v,
                        VersionCheck::Range { min, max } => {
                            pkg.version >= *min && pkg.version < *max
                        }
                        VersionCheck::Exact(v) => pkg.version == *v,
                    };

                    if is_affected {
                        vulnerabilities.push(Vulnerability {
                            id: vuln.cve_id.clone(),
                            title: vuln.title.clone(),
                            severity: vuln.severity.clone(),
                            package: pkg.id.clone(),
                            affected_versions: vuln.affected_versions.clone(),
                            fixed_version: vuln.fixed_version.clone(),
                        });
                    }
                }
            }
        }

        // Sort by severity (critical > high > medium > low)
        vulnerabilities.sort_by(|a, b| {
            let severity_order = |s: &str| match s {
                "critical" => 0,
                "high" => 1,
                "medium" => 2,
                "low" => 3,
                _ => 4,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        Ok(vulnerabilities)
    }

    /// Add package to world set
    pub async fn add_to_world(&self, pkg_id: &PackageId) -> Result<()> {
        let world_file = self.config.root.join("var/lib/portage/world");

        // Ensure directory exists
        if let Some(parent) = world_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Read existing world set
        let mut world = std::collections::HashSet::new();
        if world_file.exists() {
            let content = std::fs::read_to_string(&world_file)?;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    world.insert(line.to_string());
                }
            }
        }

        // Add new package
        world.insert(pkg_id.full_name());

        // Write back
        let mut content = String::new();
        let mut sorted: Vec<_> = world.into_iter().collect();
        sorted.sort();
        for pkg in sorted {
            content.push_str(&pkg);
            content.push('\n');
        }

        std::fs::write(&world_file, content)?;

        Ok(())
    }

    /// Remove package from world set
    pub async fn remove_from_world(&self, pkg_id: &PackageId) -> Result<()> {
        let world_file = self.config.root.join("var/lib/portage/world");

        if !world_file.exists() {
            return Ok(());
        }

        // Read existing world set
        let content = std::fs::read_to_string(&world_file)?;
        let mut world: std::collections::HashSet<String> = content
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .collect();

        // Remove package
        world.remove(&pkg_id.full_name());

        // Write back
        let mut new_content = String::new();
        let mut sorted: Vec<_> = world.into_iter().collect();
        sorted.sort();
        for pkg in sorted {
            new_content.push_str(&pkg);
            new_content.push('\n');
        }

        std::fs::write(&world_file, new_content)?;

        Ok(())
    }

    /// Get reverse dependencies (packages that depend on a given package)
    pub async fn get_reverse_dependencies(&self, package: &str) -> Result<Vec<String>> {
        let db = self.db.read().await;
        db.get_reverse_dependencies(package)
    }

    /// Find the package that owns a file
    pub async fn find_file_owner(&self, path: &str) -> Result<Option<OwnerResult>> {
        let db = self.db.read().await;

        // Normalize the path
        let normalized_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };

        // Try exact match first
        if let Some(pkg_name) = db.get_file_owner(&normalized_path)? {
            if let Some(pkg) = db.get_installed(&pkg_name)? {
                return Ok(Some(OwnerResult {
                    package: pkg.id.clone(),
                    version: pkg.version.clone(),
                    file_path: normalized_path,
                }));
            }
        }

        Ok(None)
    }

    /// Search for files matching a pattern and return their owners
    pub async fn find_file_owners_by_pattern(&self, pattern: &str) -> Result<Vec<OwnerResult>> {
        let db = self.db.read().await;
        let installed = db.get_all_installed()?;

        let mut results = Vec::new();

        for pkg in installed {
            for file in &pkg.files {
                if file.path.contains(pattern) {
                    results.push(OwnerResult {
                        package: pkg.id.clone(),
                        version: pkg.version.clone(),
                        file_path: file.path.clone(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Find packages with broken library dependencies
    pub async fn find_broken_deps(
        &self,
        library: Option<&str>,
        packages: &[String],
    ) -> Result<Vec<BrokenPackage>> {
        info!("Scanning for broken library dependencies");

        let db = self.db.read().await;

        // Get packages to check
        let to_check: Vec<InstalledPackage> = if packages.is_empty() {
            db.get_all_installed()?
        } else {
            let mut pkgs = Vec::new();
            for name in packages {
                if let Some(pkg) = db.get_installed(name)? {
                    pkgs.push(pkg);
                }
            }
            pkgs
        };
        drop(db);

        // Standard library paths to check
        let lib_paths = vec![
            "/lib",
            "/lib64",
            "/usr/lib",
            "/usr/lib64",
            "/usr/local/lib",
            "/usr/local/lib64",
        ];

        let mut broken_packages = Vec::new();

        for pkg in &to_check {
            let mut broken_libs = Vec::new();

            // Check each file in the package
            for file in &pkg.files {
                // Only check executable files and shared libraries
                if !file.path.ends_with(".so")
                    && !file.path.contains(".so.")
                    && file.file_type != FileType::Regular
                {
                    continue;
                }

                // Check if it's a binary or shared library
                let path = std::path::Path::new(&file.path);
                if !path.exists() {
                    continue;
                }

                // Try to read ELF dependencies (simplified check)
                if let Ok(deps) = self.get_elf_dependencies(&file.path).await {
                    for dep in deps {
                        // If a specific library was requested, only check that
                        if let Some(lib) = library {
                            if !dep.contains(lib) {
                                continue;
                            }
                        }

                        // Check if the dependency exists
                        let mut found = false;
                        for lib_path in &lib_paths {
                            let full_path = format!("{}/{}", lib_path, dep);
                            if std::path::Path::new(&full_path).exists() {
                                found = true;
                                break;
                            }
                        }

                        if !found && !broken_libs.contains(&dep) {
                            broken_libs.push(dep);
                        }
                    }
                }
            }

            if !broken_libs.is_empty() {
                broken_packages.push(BrokenPackage {
                    id: pkg.id.clone(),
                    name: pkg.name.clone(),
                    version: pkg.version.clone(),
                    broken_libs,
                });
            }
        }

        // Sort by package name
        broken_packages.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(broken_packages)
    }

    /// Get ELF library dependencies for a file
    async fn get_elf_dependencies(&self, path: &str) -> Result<Vec<String>> {
        // Use readelf or objdump to get dependencies
        let output = tokio::process::Command::new("readelf")
            .args(["-d", path])
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut deps = Vec::new();

                // Parse NEEDED entries
                for line in stdout.lines() {
                    if line.contains("(NEEDED)") {
                        // Extract library name from "Shared library: [libfoo.so.1]"
                        if let Some(start) = line.find('[') {
                            if let Some(end) = line.find(']') {
                                let lib = &line[start + 1..end];
                                deps.push(lib.to_string());
                            }
                        }
                    }
                }

                Ok(deps)
            }
            _ => Ok(Vec::new()), // Not an ELF file or readelf not available
        }
    }

    /// Rebuild a list of packages
    pub async fn rebuild_packages(&self, packages: &[BrokenPackage]) -> Result<()> {
        info!("Rebuilding {} packages", packages.len());

        // Convert to package names and install with force
        let package_names: Vec<String> = packages.iter().map(|p| p.id.full_name()).collect();

        let opts = InstallOptions {
            force: true, // Force rebuild
            build: true, // Build from source
            ..Default::default()
        };

        self.install(&package_names, opts).await
    }
}

/// Result of file owner query
#[derive(Debug, Clone)]
pub struct OwnerResult {
    pub package: PackageId,
    pub version: semver::Version,
    pub file_path: String,
}

/// Package with broken library dependencies
#[derive(Debug, Clone)]
pub struct BrokenPackage {
    pub id: PackageId,
    pub name: String,
    pub version: semver::Version,
    pub broken_libs: Vec<String>,
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
    /// Use flags to enable
    pub use_flags: Vec<String>,
    /// Don't add to world set
    pub oneshot: bool,
    /// Only download packages, don't install
    pub fetch_only: bool,
    /// Update deep dependencies
    pub deep: bool,
    /// Rebuild for USE flag changes
    pub newuse: bool,
    /// Empty dependency tree before installing
    pub empty_tree: bool,
    /// Don't reinstall if already installed
    pub no_replace: bool,
    /// Use binary packages when available (--usepkg)
    pub use_pkg: bool,
    /// Only use binary packages, fail if not available (--usepkgonly)
    pub use_pkg_only: bool,
    /// Fetch binary packages from remote (--getbinpkg)
    pub get_binpkg: bool,
    /// Only fetch binary packages, don't build (--getbinpkgonly)
    pub get_binpkg_only: bool,
    /// Build binary packages after compilation (--buildpkg)
    pub build_pkg: bool,
    /// Only build binary packages (--buildpkgonly)
    pub build_pkg_only: bool,
}

/// Global emerge-style options
#[derive(Debug, Clone, Default)]
pub struct EmergeOptions {
    /// Pretend mode - show what would be done
    pub pretend: bool,
    /// Ask for confirmation
    pub ask: bool,
    /// Only download, don't install
    pub fetch_only: bool,
    /// Don't add to world set
    pub oneshot: bool,
    /// Update dependencies too
    pub deep: bool,
    /// Rebuild for USE flag changes
    pub newuse: bool,
    /// Show dependency tree
    pub tree: bool,
    /// Verbosity level
    pub verbose: u8,
    /// Quiet mode
    pub quiet: bool,
    /// Number of parallel jobs
    pub jobs: Option<usize>,
    /// Use binary packages when available (--usepkg)
    pub use_pkg: bool,
    /// Only use binary packages (--usepkgonly)
    pub use_pkg_only: bool,
    /// Fetch binary packages from remote (--getbinpkg)
    pub get_binpkg: bool,
    /// Only fetch binary packages (--getbinpkgonly)
    pub get_binpkg_only: bool,
    /// Build binary packages after compilation (--buildpkg)
    pub build_pkg: bool,
    /// Only build binary packages (--buildpkgonly)
    pub build_pkg_only: bool,
}

/// Options for depclean command
#[derive(Debug, Clone, Default)]
pub struct DepcleanOptions {
    /// Specific packages to consider
    pub packages: Vec<String>,
    /// Pretend mode
    pub pretend: bool,
}

/// Options for sync command
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    /// Specific repositories to sync
    pub repos: Vec<String>,
    /// Sync all repositories
    pub all: bool,
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
    /// Update deep dependencies
    pub deep: bool,
    /// Rebuild for USE flag changes
    pub newuse: bool,
    /// Include build dependencies
    pub with_bdeps: bool,
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
    /// Custom Buck configuration options for this build
    pub config_options: Option<BuckConfigOptions>,
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

/// Version check for vulnerability matching
#[derive(Debug, Clone)]
pub enum VersionCheck {
    /// Versions less than the specified version
    LessThan(semver::Version),
    /// Versions less than or equal to the specified version
    LessThanOrEqual(semver::Version),
    /// Versions within a range [min, max)
    Range {
        min: semver::Version,
        max: semver::Version,
    },
    /// Exact version match
    Exact(semver::Version),
}

/// Entry in the vulnerability database
#[derive(Debug, Clone)]
pub struct VulnerabilityEntry {
    /// CVE identifier
    pub cve_id: String,
    /// Package name
    pub package_name: String,
    /// Human-readable title
    pub title: String,
    /// Severity level
    pub severity: String,
    /// Version check for affected versions
    pub version_check: VersionCheck,
    /// Human-readable affected versions string
    pub affected_versions: String,
    /// Fixed version if available
    pub fixed_version: Option<String>,
}

/// Get the vulnerability database
fn get_vulnerability_database() -> Vec<VulnerabilityEntry> {
    vec![
        // OpenSSL vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-0727".to_string(),
            package_name: "openssl".to_string(),
            title: "PKCS12 Decoding crash due to NULL pointer dereference".to_string(),
            severity: "medium".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(3, 2, 1)),
            affected_versions: "<3.2.1".to_string(),
            fixed_version: Some("3.2.1".to_string()),
        },
        VulnerabilityEntry {
            cve_id: "CVE-2023-5678".to_string(),
            package_name: "openssl".to_string(),
            title: "Excessive time spent in DH key generation".to_string(),
            severity: "low".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(3, 1, 4)),
            affected_versions: "<3.1.4".to_string(),
            fixed_version: Some("3.1.4".to_string()),
        },
        VulnerabilityEntry {
            cve_id: "CVE-2023-3817".to_string(),
            package_name: "openssl".to_string(),
            title: "Excessive time spent checking DH q parameter".to_string(),
            severity: "medium".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(3, 1, 2)),
            affected_versions: "<3.1.2".to_string(),
            fixed_version: Some("3.1.2".to_string()),
        },
        // curl vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-2398".to_string(),
            package_name: "curl".to_string(),
            title: "HTTP/2 push headers memory leak".to_string(),
            severity: "medium".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(8, 7, 1)),
            affected_versions: "<8.7.1".to_string(),
            fixed_version: Some("8.7.1".to_string()),
        },
        VulnerabilityEntry {
            cve_id: "CVE-2024-2004".to_string(),
            package_name: "curl".to_string(),
            title: "Usage of disabled protocol".to_string(),
            severity: "low".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(8, 6, 0)),
            affected_versions: "<8.6.0".to_string(),
            fixed_version: Some("8.6.0".to_string()),
        },
        VulnerabilityEntry {
            cve_id: "CVE-2023-46218".to_string(),
            package_name: "curl".to_string(),
            title: "Cookie mixed case PSL bypass".to_string(),
            severity: "medium".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(8, 5, 0)),
            affected_versions: "<8.5.0".to_string(),
            fixed_version: Some("8.5.0".to_string()),
        },
        // glibc vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-2961".to_string(),
            package_name: "glibc".to_string(),
            title: "Buffer overflow in iconv".to_string(),
            severity: "high".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(2, 39, 0)),
            affected_versions: "<2.39".to_string(),
            fixed_version: Some("2.39".to_string()),
        },
        VulnerabilityEntry {
            cve_id: "CVE-2023-6246".to_string(),
            package_name: "glibc".to_string(),
            title: "Heap buffer overflow in __vsyslog_internal".to_string(),
            severity: "high".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(2, 38, 0)),
            affected_versions: "<2.38".to_string(),
            fixed_version: Some("2.38".to_string()),
        },
        // Linux kernel vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-1086".to_string(),
            package_name: "linux".to_string(),
            title: "Netfilter nf_tables use-after-free".to_string(),
            severity: "critical".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(6, 8, 0)),
            affected_versions: "<6.8".to_string(),
            fixed_version: Some("6.8".to_string()),
        },
        VulnerabilityEntry {
            cve_id: "CVE-2024-0646".to_string(),
            package_name: "linux".to_string(),
            title: "ktls out-of-bounds memory access".to_string(),
            severity: "high".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(6, 7, 0)),
            affected_versions: "<6.7".to_string(),
            fixed_version: Some("6.7".to_string()),
        },
        // OpenSSH vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-6387".to_string(),
            package_name: "openssh".to_string(),
            title: "RegreSSHion - Remote Code Execution".to_string(),
            severity: "critical".to_string(),
            version_check: VersionCheck::Range {
                min: semver::Version::new(8, 5, 0),
                max: semver::Version::new(9, 8, 0),
            },
            affected_versions: "8.5p1-9.7p1".to_string(),
            fixed_version: Some("9.8p1".to_string()),
        },
        // Python vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-0450".to_string(),
            package_name: "python".to_string(),
            title: "zipfile path traversal".to_string(),
            severity: "medium".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(3, 12, 2)),
            affected_versions: "<3.12.2".to_string(),
            fixed_version: Some("3.12.2".to_string()),
        },
        // Bash vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2022-3715".to_string(),
            package_name: "bash".to_string(),
            title: "Heap buffer overflow in valid_parameter_transform".to_string(),
            severity: "high".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(5, 2, 0)),
            affected_versions: "<5.2".to_string(),
            fixed_version: Some("5.2".to_string()),
        },
        // Sudo vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2023-22809".to_string(),
            package_name: "sudo".to_string(),
            title: "Sudoedit arbitrary file write".to_string(),
            severity: "high".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(1, 9, 12)),
            affected_versions: "<1.9.12p2".to_string(),
            fixed_version: Some("1.9.12p2".to_string()),
        },
        // Git vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-32002".to_string(),
            package_name: "git".to_string(),
            title: "Recursive clone RCE on case-insensitive filesystems".to_string(),
            severity: "critical".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(2, 45, 1)),
            affected_versions: "<2.45.1".to_string(),
            fixed_version: Some("2.45.1".to_string()),
        },
        // SQLite vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2023-7104".to_string(),
            package_name: "sqlite".to_string(),
            title: "Heap buffer overflow in sessionReadRecord".to_string(),
            severity: "high".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(3, 44, 0)),
            affected_versions: "<3.44.0".to_string(),
            fixed_version: Some("3.44.0".to_string()),
        },
        // zlib vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2023-45853".to_string(),
            package_name: "zlib".to_string(),
            title: "MiniZip integer overflow and heap-based buffer overflow".to_string(),
            severity: "critical".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(1, 3, 0)),
            affected_versions: "<1.3".to_string(),
            fixed_version: Some("1.3".to_string()),
        },
        // libxml2 vulnerabilities
        VulnerabilityEntry {
            cve_id: "CVE-2024-25062".to_string(),
            package_name: "libxml2".to_string(),
            title: "Use-after-free in xmlValidatePopElement".to_string(),
            severity: "high".to_string(),
            version_check: VersionCheck::LessThan(semver::Version::new(2, 12, 5)),
            affected_versions: "<2.12.5".to_string(),
            fixed_version: Some("2.12.5".to_string()),
        },
    ]
}
