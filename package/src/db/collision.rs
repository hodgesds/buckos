//! File collision detection
//!
//! Prevents packages from overwriting files owned by other packages.

use crate::{Error, PackageId, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Configuration for collision detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionConfig {
    /// Patterns to ignore for collision detection
    pub ignore_patterns: Vec<String>,
    /// Whether to protect /etc files
    pub protect_etc: bool,
    /// Whether to allow collisions in /usr/share/doc
    pub allow_doc_collisions: bool,
    /// Whether to allow collisions in /usr/share/man
    pub allow_man_collisions: bool,
}

impl Default for CollisionConfig {
    fn default() -> Self {
        Self {
            ignore_patterns: vec![
                "/usr/share/info/dir".to_string(),
                "*.pyc".to_string(),
                "*.pyo".to_string(),
                "__pycache__/*".to_string(),
            ],
            protect_etc: true,
            allow_doc_collisions: true,
            allow_man_collisions: true,
        }
    }
}

/// A file collision
#[derive(Debug, Clone)]
pub struct Collision {
    /// Path of the colliding file
    pub path: PathBuf,
    /// Package being installed
    pub installing: PackageId,
    /// Package that owns the existing file
    pub owner: PackageId,
    /// Type of collision
    pub collision_type: CollisionType,
}

/// Type of collision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionType {
    /// File exists and is owned by another package
    OwnedByOther,
    /// File exists but is orphaned (no owner)
    Orphaned,
    /// Directory vs file conflict
    TypeMismatch,
    /// Symlink target differs
    SymlinkDiffers,
}

/// Result of collision detection
#[derive(Debug, Clone)]
pub struct CollisionResult {
    /// Detected collisions
    pub collisions: Vec<Collision>,
    /// Files that are safe to install
    pub safe_files: Vec<PathBuf>,
    /// Whether the installation can proceed
    pub can_proceed: bool,
}

/// File collision detector
pub struct CollisionDetector {
    /// Configuration
    config: CollisionConfig,
    /// File ownership database (path -> package)
    file_owners: HashMap<PathBuf, PackageId>,
}

impl CollisionDetector {
    /// Create a new collision detector
    pub fn new(config: CollisionConfig) -> Self {
        Self {
            config,
            file_owners: HashMap::new(),
        }
    }

    /// Load file ownership from database
    pub fn load_from_db(&mut self, files: Vec<(PathBuf, PackageId)>) {
        self.file_owners = files.into_iter().collect();
    }

    /// Check for collisions when installing files
    pub fn check_collisions(&self, pkg_id: &PackageId, files: &[PathBuf]) -> CollisionResult {
        let mut collisions = Vec::new();
        let mut safe_files = Vec::new();

        for file in files {
            // Check if this file should be ignored
            if self.should_ignore(file) {
                safe_files.push(file.clone());
                continue;
            }

            // Check for existing owner
            if let Some(owner) = self.file_owners.get(file) {
                if owner != pkg_id {
                    collisions.push(Collision {
                        path: file.clone(),
                        installing: pkg_id.clone(),
                        owner: owner.clone(),
                        collision_type: CollisionType::OwnedByOther,
                    });
                    continue;
                }
            }

            // Check if file exists on filesystem without owner (orphaned)
            if file.exists() && !self.file_owners.contains_key(file) {
                // Check if it's a type mismatch
                let collision_type = if file.is_dir() {
                    CollisionType::TypeMismatch
                } else {
                    CollisionType::Orphaned
                };

                collisions.push(Collision {
                    path: file.clone(),
                    installing: pkg_id.clone(),
                    owner: PackageId::new("unknown", "orphaned"),
                    collision_type,
                });
                continue;
            }

            safe_files.push(file.clone());
        }

        let can_proceed =
            collisions.is_empty() || collisions.iter().all(|c| self.is_acceptable_collision(c));

        CollisionResult {
            collisions,
            safe_files,
            can_proceed,
        }
    }

    /// Check if a path should be ignored for collision detection
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check allowed collision paths
        if self.config.allow_doc_collisions && path_str.contains("/usr/share/doc/") {
            return true;
        }
        if self.config.allow_man_collisions && path_str.contains("/usr/share/man/") {
            return true;
        }

        // Check ignore patterns
        for pattern in &self.config.ignore_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a path matches a glob pattern
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let starts = parts[0].is_empty() || path.starts_with(parts[0]);
                let ends = parts[1].is_empty() || path.ends_with(parts[1]);
                return starts && ends;
            }
        }
        path == pattern || path.ends_with(pattern)
    }

    /// Check if a collision is acceptable
    fn is_acceptable_collision(&self, collision: &Collision) -> bool {
        match collision.collision_type {
            CollisionType::OwnedByOther => false,
            CollisionType::Orphaned => true, // Can overwrite orphaned files
            CollisionType::TypeMismatch => false,
            CollisionType::SymlinkDiffers => false,
        }
    }

    /// Register files for a package
    pub fn register_files(&mut self, pkg_id: &PackageId, files: &[PathBuf]) {
        for file in files {
            self.file_owners.insert(file.clone(), pkg_id.clone());
        }
    }

    /// Unregister files for a package
    pub fn unregister_files(&mut self, pkg_id: &PackageId) {
        self.file_owners.retain(|_, owner| owner != pkg_id);
    }

    /// Get the owner of a file
    pub fn get_owner(&self, path: &Path) -> Option<&PackageId> {
        self.file_owners.get(path)
    }

    /// Find all files owned by a package
    pub fn get_package_files(&self, pkg_id: &PackageId) -> Vec<&PathBuf> {
        self.file_owners
            .iter()
            .filter(|(_, owner)| *owner == pkg_id)
            .map(|(path, _)| path)
            .collect()
    }

    /// Resolve collisions by determining which files to keep/replace
    pub fn resolve_collisions(
        &self,
        collisions: &[Collision],
        force: bool,
    ) -> Vec<CollisionResolution> {
        collisions
            .iter()
            .map(|collision| {
                let action = if force {
                    CollisionAction::Replace
                } else {
                    match collision.collision_type {
                        CollisionType::OwnedByOther => CollisionAction::Skip,
                        CollisionType::Orphaned => CollisionAction::Replace,
                        CollisionType::TypeMismatch => CollisionAction::Skip,
                        CollisionType::SymlinkDiffers => CollisionAction::Skip,
                    }
                };

                CollisionResolution {
                    collision: collision.clone(),
                    action,
                }
            })
            .collect()
    }
}

impl Default for CollisionDetector {
    fn default() -> Self {
        Self::new(CollisionConfig::default())
    }
}

/// Resolution for a collision
#[derive(Debug, Clone)]
pub struct CollisionResolution {
    /// The collision
    pub collision: Collision,
    /// Action to take
    pub action: CollisionAction,
}

/// Action to take for a collision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionAction {
    /// Skip installing this file
    Skip,
    /// Replace the existing file
    Replace,
    /// Backup existing and install new
    Backup,
}

/// Extend configuration to support COLLISION_IGNORE
pub fn parse_collision_ignore(ignore_str: &str) -> Vec<String> {
    ignore_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Format collision report
pub fn format_collision_report(result: &CollisionResult) -> String {
    if result.collisions.is_empty() {
        return "No file collisions detected.".to_string();
    }

    let mut report = String::new();
    report.push_str(&format!(
        "Detected {} file collision(s):\n\n",
        result.collisions.len()
    ));

    for collision in &result.collisions {
        let type_str = match collision.collision_type {
            CollisionType::OwnedByOther => "owned by another package",
            CollisionType::Orphaned => "orphaned file",
            CollisionType::TypeMismatch => "type mismatch",
            CollisionType::SymlinkDiffers => "symlink differs",
        };

        report.push_str(&format!(
            "  {} -> {} ({})\n    Owner: {}\n",
            collision.path.display(),
            collision.installing,
            type_str,
            collision.owner
        ));
    }

    if result.can_proceed {
        report.push_str("\nInstallation can proceed with some collisions.\n");
    } else {
        report.push_str("\nInstallation blocked due to collisions.\n");
        report.push_str("Use --force to override or resolve conflicts manually.\n");
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = CollisionConfig::default();
        assert!(config.protect_etc);
        assert!(config.allow_doc_collisions);
    }

    #[test]
    fn test_should_ignore() {
        let detector = CollisionDetector::default();

        // Should ignore info dir
        assert!(detector.should_ignore(Path::new("/usr/share/info/dir")));

        // Should ignore doc files
        assert!(detector.should_ignore(Path::new("/usr/share/doc/foo/README")));

        // Should not ignore regular files
        assert!(!detector.should_ignore(Path::new("/usr/bin/foo")));
    }

    #[test]
    fn test_collision_detection() {
        let mut detector = CollisionDetector::default();

        let pkg_a = PackageId::new("sys-apps", "foo");
        let pkg_b = PackageId::new("sys-apps", "bar");

        // Register file for pkg_a
        detector.register_files(&pkg_a, &[PathBuf::from("/usr/bin/shared")]);

        // Try to install same file for pkg_b
        let result = detector.check_collisions(&pkg_b, &[PathBuf::from("/usr/bin/shared")]);

        assert_eq!(result.collisions.len(), 1);
        assert!(!result.can_proceed);
    }

    #[test]
    fn test_parse_collision_ignore() {
        let patterns = parse_collision_ignore("/usr/share/info/dir /usr/lib/debug/*");
        assert_eq!(patterns.len(), 2);
    }
}
