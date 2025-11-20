//! Transaction system for atomic package operations
//!
//! Ensures that package operations are atomic with rollback support.

use crate::buck::BuckIntegration;
use crate::cache::PackageCache;
use crate::db::PackageDb;
use crate::executor::{ParallelExecutor, Task, TaskOutput};
use crate::{
    BuildOptions, Error, FileType, InstalledFile, InstalledPackage, PackageId, PackageInfo, Result,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Package operation type
#[derive(Debug, Clone)]
pub enum Operation {
    Install(PackageInfo),
    Remove(InstalledPackage),
    Upgrade {
        old: InstalledPackage,
        new: PackageInfo,
    },
}

/// Transaction for package operations
pub struct Transaction {
    db: Arc<RwLock<PackageDb>>,
    cache: Arc<PackageCache>,
    buck: Arc<BuckIntegration>,
    operations: Vec<Operation>,
    backup_dir: PathBuf,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        db: Arc<RwLock<PackageDb>>,
        cache: Arc<PackageCache>,
        buck: Arc<BuckIntegration>,
    ) -> Self {
        let backup_dir = std::env::temp_dir().join("buckos-backup");
        Self {
            db,
            cache,
            buck,
            operations: Vec::new(),
            backup_dir,
        }
    }

    /// Add an install operation
    pub fn add_install(&mut self, pkg: PackageInfo) {
        self.operations.push(Operation::Install(pkg));
    }

    /// Add a remove operation
    pub fn add_remove(&mut self, pkg: InstalledPackage) {
        self.operations.push(Operation::Remove(pkg));
    }

    /// Add an upgrade operation
    pub fn add_upgrade(&mut self, old: InstalledPackage, new: PackageInfo) {
        self.operations.push(Operation::Upgrade { old, new });
    }

    /// Execute the transaction
    pub async fn execute(&mut self, executor: &ParallelExecutor) -> Result<()> {
        if self.operations.is_empty() {
            return Ok(());
        }

        info!(
            "Executing transaction with {} operations",
            self.operations.len()
        );

        // Create backup directory
        std::fs::create_dir_all(&self.backup_dir)?;

        // Start database transaction
        {
            let mut db = self.db.write().await;
            db.begin_transaction()?;
        }

        let result = self.execute_operations(executor).await;

        match result {
            Ok(()) => {
                // Commit database transaction
                let mut db = self.db.write().await;
                db.commit()?;
                info!("Transaction committed successfully");

                // Clean up backup
                if self.backup_dir.exists() {
                    let _ = std::fs::remove_dir_all(&self.backup_dir);
                }

                Ok(())
            }
            Err(e) => {
                error!("Transaction failed: {}", e);

                // Rollback database transaction
                let mut db = self.db.write().await;
                let _ = db.rollback();

                // Restore from backup
                if let Err(restore_err) = self.restore_backup().await {
                    error!("Failed to restore backup: {}", restore_err);
                }

                Err(Error::TransactionRolledBack(e.to_string()))
            }
        }
    }

    async fn execute_operations(&self, executor: &ParallelExecutor) -> Result<()> {
        // Group operations by type
        let mut installs = Vec::new();
        let mut removes = Vec::new();
        let mut upgrades = Vec::new();

        for op in &self.operations {
            match op {
                Operation::Install(pkg) => installs.push(pkg.clone()),
                Operation::Remove(pkg) => removes.push(pkg.clone()),
                Operation::Upgrade { old, new } => upgrades.push((old.clone(), new.clone())),
            }
        }

        // Execute removes first
        for pkg in &removes {
            self.execute_remove(pkg).await?;
        }

        // Execute upgrades (remove old, install new)
        for (old, new) in &upgrades {
            self.execute_remove(old).await?;
            self.execute_install(new).await?;
        }

        // Execute installs
        for pkg in &installs {
            self.execute_install(pkg).await?;
        }

        Ok(())
    }

    async fn execute_install(&self, pkg: &PackageInfo) -> Result<()> {
        info!("Installing {}-{}", pkg.id.name, pkg.version);

        // Build the package using Buck
        let target = &pkg.buck_target;
        let build_result = self.buck.build(target, &BuildOptions::default()).await?;

        if !build_result.success {
            return Err(Error::BuildFailed {
                package: pkg.id.name.clone(),
                message: build_result.stderr,
            });
        }

        // Get the built package
        let output_path = build_result.output_path.ok_or_else(|| Error::BuildFailed {
            package: pkg.id.name.clone(),
            message: "No output produced".to_string(),
        })?;

        // Extract and install files
        let files = self.install_files(&output_path, &pkg.id).await?;

        // Record in database
        let installed = InstalledPackage {
            id: pkg.id.clone(),
            name: pkg.id.name.clone(),
            version: pkg.version.clone(),
            slot: pkg.slot.clone(),
            installed_at: chrono::Utc::now(),
            use_flags: HashSet::new(),
            files,
            size: pkg.installed_size,
            build_time: false,
            explicit: true,
        };

        let mut db = self.db.write().await;
        db.add_package(&installed)?;

        info!("Installed {}-{}", pkg.id.name, pkg.version);
        Ok(())
    }

    async fn execute_remove(&self, pkg: &InstalledPackage) -> Result<()> {
        info!("Removing {}-{}", pkg.name, pkg.version);

        // Backup files first
        self.backup_package(pkg).await?;

        // Remove files in reverse order (files before directories)
        let mut files = pkg.files.clone();
        files.sort_by(|a, b| b.path.cmp(&a.path));

        for file in &files {
            let path = Path::new(&file.path);
            if path.exists() {
                match file.file_type {
                    FileType::Directory => {
                        // Only remove empty directories
                        let _ = std::fs::remove_dir(path);
                    }
                    _ => {
                        std::fs::remove_file(path)?;
                    }
                }
            }
        }

        // Remove from database
        let mut db = self.db.write().await;
        db.remove_package(&pkg.name)?;

        info!("Removed {}-{}", pkg.name, pkg.version);
        Ok(())
    }

    async fn install_files(
        &self,
        archive_path: &Path,
        pkg_id: &PackageId,
    ) -> Result<Vec<InstalledFile>> {
        let temp_dir = tempfile::tempdir()?;
        let extract_dir = temp_dir.path();

        // Extract archive
        crate::cache::extract_tarball(archive_path, extract_dir)?;

        // Install files to system
        let mut installed_files = Vec::new();

        for entry in walkdir::WalkDir::new(extract_dir) {
            let entry = entry?;
            let relative_path = match entry.path().strip_prefix(extract_dir) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if relative_path.as_os_str().is_empty() {
                continue;
            }

            let dest_path = Path::new("/").join(relative_path);
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
                installed_files.push(InstalledFile {
                    path: dest_path.to_string_lossy().to_string(),
                    file_type: FileType::Directory,
                    mode: 0o755,
                    size: 0,
                    blake3_hash: None,
                    mtime: metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                });
            } else if metadata.is_file() {
                // Ensure parent directory exists
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Copy file
                std::fs::copy(entry.path(), &dest_path)?;

                // Compute hash
                let hash = crate::cache::compute_blake3(&dest_path)?;

                installed_files.push(InstalledFile {
                    path: dest_path.to_string_lossy().to_string(),
                    file_type: FileType::Regular,
                    mode: 0o644,
                    size: metadata.len(),
                    blake3_hash: Some(hash),
                    mtime: metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                });
            } else if metadata.file_type().is_symlink() {
                let target = std::fs::read_link(entry.path())?;

                // Remove existing symlink if present
                if dest_path.exists() || dest_path.is_symlink() {
                    std::fs::remove_file(&dest_path)?;
                }

                std::os::unix::fs::symlink(&target, &dest_path)?;

                installed_files.push(InstalledFile {
                    path: dest_path.to_string_lossy().to_string(),
                    file_type: FileType::Symlink,
                    mode: 0o777,
                    size: 0,
                    blake3_hash: None,
                    mtime: 0,
                });
            }
        }

        Ok(installed_files)
    }

    async fn backup_package(&self, pkg: &InstalledPackage) -> Result<()> {
        let backup_path = self.backup_dir.join(&pkg.name);
        std::fs::create_dir_all(&backup_path)?;

        for file in &pkg.files {
            let src_path = Path::new(&file.path);
            if !src_path.exists() {
                continue;
            }

            let relative = src_path.strip_prefix("/").unwrap_or(src_path);
            let dest = backup_path.join(relative);

            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }

            match file.file_type {
                FileType::Regular => {
                    std::fs::copy(src_path, &dest)?;
                }
                FileType::Symlink => {
                    let target = std::fs::read_link(src_path)?;
                    std::os::unix::fs::symlink(target, &dest)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn restore_backup(&self) -> Result<()> {
        if !self.backup_dir.exists() {
            return Ok(());
        }

        info!("Restoring from backup");

        for entry in walkdir::WalkDir::new(&self.backup_dir) {
            let entry = entry?;
            let relative = match entry.path().strip_prefix(&self.backup_dir) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if relative.as_os_str().is_empty() {
                continue;
            }

            // First component is package name
            let components: Vec<_> = relative.components().collect();
            if components.len() < 2 {
                continue;
            }

            let file_path: PathBuf = components[1..].iter().collect();
            let dest_path = Path::new("/").join(&file_path);

            if entry.file_type().is_file() {
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(entry.path(), &dest_path)?;
            } else if entry.file_type().is_symlink() {
                let target = std::fs::read_link(entry.path())?;
                if dest_path.exists() {
                    std::fs::remove_file(&dest_path)?;
                }
                std::os::unix::fs::symlink(target, &dest_path)?;
            }
        }

        Ok(())
    }
}
