//! Configuration file protection
//!
//! Protects user-modified configuration files during package upgrades.
//! Similar to Portage's CONFIG_PROTECT and dispatch-conf/etc-update.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Configuration protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectConfig {
    /// Paths to protect
    pub protected_paths: Vec<PathBuf>,
    /// Paths to exempt from protection (inside protected paths)
    pub mask_paths: Vec<PathBuf>,
}

impl Default for ProtectConfig {
    fn default() -> Self {
        Self {
            protected_paths: vec![
                PathBuf::from("/etc"),
            ],
            mask_paths: vec![
                PathBuf::from("/etc/env.d"),
                PathBuf::from("/etc/gconf"),
                PathBuf::from("/etc/sandbox.d"),
                PathBuf::from("/etc/terminfo"),
                PathBuf::from("/etc/texmf"),
                PathBuf::from("/etc/udev/hwdb.d"),
            ],
        }
    }
}

/// A protected configuration file update
#[derive(Debug, Clone)]
pub struct ConfigUpdate {
    /// Original file path
    pub path: PathBuf,
    /// Temporary file with new content
    pub temp_path: PathBuf,
    /// Package that provides this update
    pub package: String,
    /// Whether files differ
    pub differs: bool,
}

/// Action for handling a config update
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Keep the existing file
    Keep,
    /// Replace with new file
    Replace,
    /// Merge changes
    Merge,
    /// Delete the update file
    Delete,
}

/// Result of comparing two config files
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    /// Lines only in the original
    pub removed: Vec<String>,
    /// Lines only in the new file
    pub added: Vec<String>,
    /// Lines that changed
    pub changed: Vec<(String, String)>,
    /// Whether the files are identical
    pub identical: bool,
}

/// Configuration protection manager
pub struct ConfigProtect {
    /// Configuration
    config: ProtectConfig,
    /// Pending updates
    pending_updates: Vec<ConfigUpdate>,
}

impl ConfigProtect {
    /// Create a new config protection manager
    pub fn new(config: ProtectConfig) -> Self {
        Self {
            config,
            pending_updates: Vec::new(),
        }
    }

    /// Check if a path is protected
    pub fn is_protected(&self, path: &Path) -> bool {
        // First check if it's in a protected path
        let in_protected = self.config.protected_paths
            .iter()
            .any(|p| path.starts_with(p));

        if !in_protected {
            return false;
        }

        // Check if it's masked (exempted)
        !self.config.mask_paths
            .iter()
            .any(|p| path.starts_with(p))
    }

    /// Protect a configuration file
    ///
    /// If the file exists and differs from the new content,
    /// create a ._cfg0000_filename file instead of overwriting.
    pub fn protect_file(
        &mut self,
        path: &Path,
        new_content: &[u8],
        package: &str,
    ) -> Result<ProtectResult> {
        // Check if protection applies
        if !self.is_protected(path) {
            return Ok(ProtectResult::NotProtected);
        }

        // Check if original file exists
        if !path.exists() {
            return Ok(ProtectResult::NotProtected);
        }

        // Read existing content
        let existing_content = std::fs::read(path)?;

        // Compare contents
        if existing_content == new_content {
            return Ok(ProtectResult::Identical);
        }

        // Create protected file
        let temp_path = self.create_protected_path(path)?;
        std::fs::write(&temp_path, new_content)?;

        // Record the update
        self.pending_updates.push(ConfigUpdate {
            path: path.to_path_buf(),
            temp_path: temp_path.clone(),
            package: package.to_string(),
            differs: true,
        });

        Ok(ProtectResult::Protected { temp_path })
    }

    /// Create a protected file path (._cfg0000_filename)
    fn create_protected_path(&self, path: &Path) -> Result<PathBuf> {
        let parent = path.parent().unwrap_or(Path::new("/"));
        let filename = path.file_name()
            .ok_or_else(|| Error::InvalidPath(path.to_string_lossy().to_string()))?;

        // Find next available config number
        let mut num = 0;
        loop {
            let temp_name = format!("._cfg{:04}_{}", num, filename.to_string_lossy());
            let temp_path = parent.join(temp_name);
            if !temp_path.exists() {
                return Ok(temp_path);
            }
            num += 1;
            if num > 9999 {
                return Err(Error::TooManyConfigFiles(path.to_path_buf()));
            }
        }
    }

    /// Find all pending configuration updates
    pub fn find_pending_updates(&mut self) -> Result<Vec<ConfigUpdate>> {
        self.pending_updates.clear();

        for protected_path in &self.config.protected_paths {
            if !protected_path.exists() {
                continue;
            }

            self.scan_directory(protected_path)?;
        }

        Ok(self.pending_updates.clone())
    }

    /// Scan a directory for ._cfg files
    fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip masked directories
                if self.config.mask_paths.iter().any(|m| path.starts_with(m)) {
                    continue;
                }
                self.scan_directory(&path)?;
            } else if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("._cfg") {
                    // Parse the original filename
                    if let Some(original_name) = filename_str.get(10..) {
                        let original_path = path.parent().unwrap().join(original_name);
                        self.pending_updates.push(ConfigUpdate {
                            path: original_path,
                            temp_path: path,
                            package: "unknown".to_string(),
                            differs: true,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply an update action
    pub fn apply_action(&self, update: &ConfigUpdate, action: UpdateAction) -> Result<()> {
        match action {
            UpdateAction::Keep => {
                // Delete the temp file
                std::fs::remove_file(&update.temp_path)?;
            }
            UpdateAction::Replace => {
                // Backup original if it exists
                if update.path.exists() {
                    let backup_path = self.create_backup_path(&update.path)?;
                    std::fs::rename(&update.path, &backup_path)?;
                }
                // Move temp file to original location
                std::fs::rename(&update.temp_path, &update.path)?;
            }
            UpdateAction::Merge => {
                // Three-way merge would go here
                // For now, just keep original
                std::fs::remove_file(&update.temp_path)?;
            }
            UpdateAction::Delete => {
                // Delete both files
                if update.path.exists() {
                    std::fs::remove_file(&update.path)?;
                }
                std::fs::remove_file(&update.temp_path)?;
            }
        }

        Ok(())
    }

    /// Create a backup path for a file
    fn create_backup_path(&self, path: &Path) -> Result<PathBuf> {
        let mut backup = path.to_path_buf();
        let filename = path.file_name()
            .ok_or_else(|| Error::InvalidPath(path.to_string_lossy().to_string()))?;
        backup.set_file_name(format!("{}.bak", filename.to_string_lossy()));

        // If backup already exists, add a number
        if backup.exists() {
            let mut num = 1;
            loop {
                let numbered = path.with_extension(format!("bak.{}", num));
                if !numbered.exists() {
                    return Ok(numbered);
                }
                num += 1;
            }
        }

        Ok(backup)
    }

    /// Compute diff between two files
    pub fn diff_files(&self, original: &Path, new: &Path) -> Result<ConfigDiff> {
        let original_content = if original.exists() {
            std::fs::read_to_string(original)?
        } else {
            String::new()
        };

        let new_content = std::fs::read_to_string(new)?;

        let original_lines: HashSet<_> = original_content.lines().collect();
        let new_lines: HashSet<_> = new_content.lines().collect();

        let removed: Vec<String> = original_lines
            .difference(&new_lines)
            .map(|s| s.to_string())
            .collect();

        let added: Vec<String> = new_lines
            .difference(&original_lines)
            .map(|s| s.to_string())
            .collect();

        let identical = removed.is_empty() && added.is_empty();

        Ok(ConfigDiff {
            removed,
            added,
            changed: Vec::new(), // Would need more sophisticated diff for this
            identical,
        })
    }

    /// Auto-merge trivial changes
    pub fn auto_merge(&self, update: &ConfigUpdate) -> Result<bool> {
        let diff = self.diff_files(&update.path, &update.temp_path)?;

        // Only auto-merge if there are only additions (no removals or changes)
        if !diff.removed.is_empty() || !diff.changed.is_empty() {
            return Ok(false);
        }

        // Read original and new files
        let original = std::fs::read_to_string(&update.path)?;
        let new = std::fs::read_to_string(&update.temp_path)?;

        // Simple merge: append new lines
        let mut merged = original;
        for line in &diff.added {
            if !merged.ends_with('\n') {
                merged.push('\n');
            }
            merged.push_str(line);
            merged.push('\n');
        }

        // Write merged content
        std::fs::write(&update.path, merged)?;
        std::fs::remove_file(&update.temp_path)?;

        Ok(true)
    }

    /// Get pending updates count
    pub fn pending_count(&self) -> usize {
        self.pending_updates.len()
    }

    /// Clear pending updates
    pub fn clear_pending(&mut self) {
        self.pending_updates.clear();
    }
}

impl Default for ConfigProtect {
    fn default() -> Self {
        Self::new(ProtectConfig::default())
    }
}

/// Result of protecting a file
#[derive(Debug, Clone)]
pub enum ProtectResult {
    /// File was not in a protected path
    NotProtected,
    /// New content identical to existing
    Identical,
    /// File was protected, temp file created
    Protected { temp_path: PathBuf },
}

/// Format pending updates report
pub fn format_updates_report(updates: &[ConfigUpdate]) -> String {
    if updates.is_empty() {
        return "No configuration file updates pending.".to_string();
    }

    let mut report = String::new();
    report.push_str(&format!(
        "Configuration files needing updating: {}\n\n",
        updates.len()
    ));

    for update in updates {
        report.push_str(&format!(
            "  {} -> {}\n",
            update.temp_path.display(),
            update.path.display()
        ));
    }

    report.push_str("\nUse 'buckos etc-update' or 'dispatch-conf' to merge updates.\n");

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ProtectConfig::default();
        assert!(config.protected_paths.contains(&PathBuf::from("/etc")));
    }

    #[test]
    fn test_is_protected() {
        let protect = ConfigProtect::default();

        // /etc files should be protected
        assert!(protect.is_protected(Path::new("/etc/passwd")));

        // /etc/env.d should be masked
        assert!(!protect.is_protected(Path::new("/etc/env.d/00basic")));

        // /usr files should not be protected
        assert!(!protect.is_protected(Path::new("/usr/bin/foo")));
    }

    #[test]
    fn test_create_protected_path() {
        let protect = ConfigProtect::default();
        let path = Path::new("/etc/test.conf");
        let protected = protect.create_protected_path(path).unwrap();

        let filename = protected.file_name().unwrap().to_string_lossy();
        assert!(filename.starts_with("._cfg"));
        assert!(filename.ends_with("test.conf"));
    }
}
