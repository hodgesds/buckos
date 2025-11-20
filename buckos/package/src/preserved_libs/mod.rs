//! Preserved libraries management
//!
//! Tracks shared libraries that are still in use after package upgrades
//! and handles their preservation until dependents are rebuilt.

use crate::{Error, PackageId, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A preserved library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservedLib {
    /// Path to the library
    pub path: PathBuf,
    /// Original package that provided this library
    pub original_package: PackageId,
    /// Version of the original package
    pub original_version: semver::Version,
    /// Soname of the library
    pub soname: String,
    /// Packages that still depend on this library
    pub consumers: HashSet<PackageId>,
    /// When this library was preserved
    pub preserved_at: chrono::DateTime<chrono::Utc>,
}

/// Manager for preserved libraries
pub struct PreservedLibsManager {
    /// Preserved libraries
    libs: HashMap<PathBuf, PreservedLib>,
    /// Database file path
    db_path: PathBuf,
}

impl PreservedLibsManager {
    /// Create a new preserved libs manager
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            libs: HashMap::new(),
            db_path,
        }
    }

    /// Load preserved libs from database
    pub fn load(&mut self) -> Result<()> {
        if !self.db_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.db_path)?;
        self.libs = serde_json::from_str(&content)
            .map_err(|e| Error::ParseError(e.to_string()))?;

        Ok(())
    }

    /// Save preserved libs to database
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.libs)
            .map_err(|e| Error::ParseError(e.to_string()))?;
        std::fs::write(&self.db_path, content)?;

        Ok(())
    }

    /// Preserve a library
    pub fn preserve(
        &mut self,
        path: PathBuf,
        original_package: PackageId,
        original_version: semver::Version,
        soname: String,
    ) -> Result<()> {
        // Find consumers of this library
        let consumers = self.find_consumers(&path)?;

        if consumers.is_empty() {
            // No consumers, no need to preserve
            return Ok(());
        }

        let lib = PreservedLib {
            path: path.clone(),
            original_package,
            original_version,
            soname,
            consumers,
            preserved_at: chrono::Utc::now(),
        };

        // Move library to preserved location
        let preserved_path = self.get_preserved_path(&path);
        if let Some(parent) = preserved_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(&path, &preserved_path)?;

        // Create symlink from original location
        std::os::unix::fs::symlink(&preserved_path, &path)?;

        self.libs.insert(path, lib);

        Ok(())
    }

    /// Get the preserved location for a library
    fn get_preserved_path(&self, original: &Path) -> PathBuf {
        let filename = original.file_name().unwrap_or_default();
        PathBuf::from("/var/cache/preserved-libs")
            .join(filename)
    }

    /// Find packages that use a library
    fn find_consumers(&self, lib_path: &Path) -> Result<HashSet<PackageId>> {
        let mut consumers = HashSet::new();

        // Use lsof or /proc to find processes using this library
        let output = std::process::Command::new("lsof")
            .arg(lib_path)
            .output();

        if let Ok(output) = output {
            // Parse lsof output to find package names
            // This is a simplified version - real implementation would
            // trace back to packages
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                // Extract process info and map to package
                // For now, we'll just note that something is using it
                if !line.is_empty() {
                    consumers.insert(PackageId::new("unknown", "process"));
                }
            }
        }

        // Also check ELF dependencies of installed packages
        // This would involve scanning VDB for packages with this soname

        Ok(consumers)
    }

    /// Check if a library is still needed
    pub fn is_needed(&self, path: &Path) -> bool {
        if let Some(lib) = self.libs.get(path) {
            !lib.consumers.is_empty()
        } else {
            false
        }
    }

    /// Remove consumer from a library
    pub fn remove_consumer(&mut self, lib_path: &Path, consumer: &PackageId) -> Result<()> {
        if let Some(lib) = self.libs.get_mut(lib_path) {
            lib.consumers.remove(consumer);
        }
        Ok(())
    }

    /// Clean up libraries that are no longer needed
    pub fn cleanup(&mut self) -> Result<Vec<PathBuf>> {
        let mut cleaned = Vec::new();

        let paths_to_remove: Vec<_> = self.libs
            .iter()
            .filter(|(_, lib)| lib.consumers.is_empty())
            .map(|(path, _)| path.clone())
            .collect();

        for path in paths_to_remove {
            if let Some(lib) = self.libs.remove(&path) {
                // Remove the symlink
                if path.is_symlink() {
                    std::fs::remove_file(&path)?;
                }

                // Remove the preserved library
                let preserved_path = self.get_preserved_path(&path);
                if preserved_path.exists() {
                    std::fs::remove_file(&preserved_path)?;
                }

                cleaned.push(path);
            }
        }

        Ok(cleaned)
    }

    /// Get all preserved libraries
    pub fn list(&self) -> Vec<&PreservedLib> {
        self.libs.values().collect()
    }

    /// Get packages that need rebuilding
    pub fn get_rebuild_list(&self) -> Vec<PackageId> {
        let mut packages = HashSet::new();

        for lib in self.libs.values() {
            packages.extend(lib.consumers.clone());
        }

        packages.into_iter().collect()
    }

    /// Check if any preserved libraries exist
    pub fn has_preserved_libs(&self) -> bool {
        !self.libs.is_empty()
    }

    /// Get library by soname
    pub fn get_by_soname(&self, soname: &str) -> Option<&PreservedLib> {
        self.libs.values().find(|lib| lib.soname == soname)
    }

    /// Register that a package was rebuilt
    pub fn package_rebuilt(&mut self, package: &PackageId) -> Result<()> {
        for lib in self.libs.values_mut() {
            lib.consumers.remove(package);
        }
        Ok(())
    }
}

impl Default for PreservedLibsManager {
    fn default() -> Self {
        Self::new(PathBuf::from("/var/lib/buckos/preserved-libs.json"))
    }
}

/// Find shared libraries in a set of files
pub fn find_shared_libs(files: &[PathBuf]) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|p| {
            let name = p.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            name.contains(".so") || p.extension().map(|e| e == "so").unwrap_or(false)
        })
        .cloned()
        .collect()
}

/// Extract soname from a shared library
pub fn get_soname(lib_path: &Path) -> Option<String> {
    let output = std::process::Command::new("objdump")
        .args(["-p", lib_path.to_str()?])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("SONAME") {
            return line.split_whitespace().last().map(|s| s.to_string());
        }
    }

    // Fall back to filename
    lib_path.file_name().map(|n| n.to_string_lossy().to_string())
}

/// Format preserved libs report
pub fn format_preserved_libs_report(libs: &[&PreservedLib]) -> String {
    if libs.is_empty() {
        return "No preserved libraries.".to_string();
    }

    let mut report = String::new();
    report.push_str(&format!("Preserved libraries: {}\n\n", libs.len()));

    for lib in libs {
        report.push_str(&format!(
            "  {} ({})\n    From: {}-{}\n    Consumers: {}\n",
            lib.path.display(),
            lib.soname,
            lib.original_package,
            lib.original_version,
            lib.consumers.len()
        ));
    }

    if libs.iter().any(|l| !l.consumers.is_empty()) {
        report.push_str("\nRun 'emerge @preserved-rebuild' to rebuild consumers.\n");
    }

    report
}

/// Check if revdep-rebuild is needed
pub fn needs_revdep_rebuild(manager: &PreservedLibsManager) -> bool {
    manager.has_preserved_libs() &&
        manager.list().iter().any(|lib| !lib.consumers.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_shared_libs() {
        let files = vec![
            PathBuf::from("/usr/lib/libfoo.so.1"),
            PathBuf::from("/usr/lib/libbar.so"),
            PathBuf::from("/usr/bin/program"),
            PathBuf::from("/usr/lib/libfoo.so.1.2.3"),
        ];

        let libs = find_shared_libs(&files);
        assert_eq!(libs.len(), 3);
    }

    #[test]
    fn test_manager_default() {
        let manager = PreservedLibsManager::default();
        assert!(!manager.has_preserved_libs());
    }

    #[test]
    fn test_get_preserved_path() {
        let manager = PreservedLibsManager::default();
        let original = PathBuf::from("/usr/lib/libfoo.so.1");
        let preserved = manager.get_preserved_path(&original);
        assert!(preserved.to_string_lossy().contains("preserved-libs"));
    }
}
