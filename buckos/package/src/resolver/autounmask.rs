//! Automatic unmasking for dependency resolution
//!
//! When packages are masked or have keyword restrictions, this module
//! can automatically suggest or apply unmasking to satisfy dependencies.

use crate::{Error, PackageId, PackageInfo, Result, VersionSpec};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Configuration for autounmask behavior
#[derive(Debug, Clone)]
pub struct AutounmaskConfig {
    /// Whether to automatically write changes
    pub auto_write: bool,
    /// Path to package.accept_keywords
    pub keywords_path: PathBuf,
    /// Path to package.unmask
    pub unmask_path: PathBuf,
    /// Path to package.use
    pub use_path: PathBuf,
    /// Maximum instability level to accept
    pub max_instability: InstabilityLevel,
}

impl Default for AutounmaskConfig {
    fn default() -> Self {
        Self {
            auto_write: false,
            keywords_path: PathBuf::from("/etc/portage/package.accept_keywords"),
            unmask_path: PathBuf::from("/etc/portage/package.unmask"),
            use_path: PathBuf::from("/etc/portage/package.use"),
            max_instability: InstabilityLevel::Testing,
        }
    }
}

/// Level of package instability
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstabilityLevel {
    /// Stable packages
    Stable,
    /// Testing (~arch) packages
    Testing,
    /// Masked packages
    Masked,
    /// Live/9999 packages
    Live,
}

/// A suggested change for unmasking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutounmaskChange {
    /// Accept keywords for a package
    AcceptKeywords {
        package: PackageId,
        version: Option<semver::Version>,
        keywords: Vec<String>,
    },
    /// Unmask a package
    Unmask {
        package: PackageId,
        version: Option<semver::Version>,
    },
    /// Change USE flags for a package
    UseChange {
        package: PackageId,
        enable: Vec<String>,
        disable: Vec<String>,
    },
    /// Accept a license
    AcceptLicense {
        package: PackageId,
        license: String,
    },
}

/// Result of autounmask analysis
#[derive(Debug, Clone)]
pub struct AutounmaskResult {
    /// Suggested changes
    pub changes: Vec<AutounmaskChange>,
    /// Packages that would be installed after changes
    pub packages: Vec<PackageId>,
    /// Whether all dependencies can be satisfied
    pub satisfiable: bool,
    /// Human-readable explanation
    pub explanation: String,
}

/// Autounmask resolver
pub struct AutounmaskResolver {
    /// Configuration
    config: AutounmaskConfig,
    /// Current keywords
    current_keywords: HashMap<PackageId, HashSet<String>>,
    /// Currently masked packages
    masked_packages: HashSet<PackageId>,
    /// Current USE flags
    current_use: HashMap<PackageId, HashSet<String>>,
}

impl AutounmaskResolver {
    /// Create a new autounmask resolver
    pub fn new(config: AutounmaskConfig) -> Self {
        Self {
            config,
            current_keywords: HashMap::new(),
            masked_packages: HashSet::new(),
            current_use: HashMap::new(),
        }
    }

    /// Load current configuration from files
    pub fn load_current_config(&mut self) -> Result<()> {
        // Load package.accept_keywords
        if self.config.keywords_path.exists() {
            let content = std::fs::read_to_string(&self.config.keywords_path)?;
            self.parse_keywords_file(&content)?;
        }

        // Load package.unmask (to know what's already unmasked)
        if self.config.unmask_path.exists() {
            let content = std::fs::read_to_string(&self.config.unmask_path)?;
            self.parse_unmask_file(&content)?;
        }

        // Load package.use
        if self.config.use_path.exists() {
            let content = std::fs::read_to_string(&self.config.use_path)?;
            self.parse_use_file(&content)?;
        }

        Ok(())
    }

    fn parse_keywords_file(&mut self, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            if let Some(pkg_id) = PackageId::parse(parts[0]) {
                let keywords: HashSet<String> = parts[1..]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                self.current_keywords.insert(pkg_id, keywords);
            }
        }
        Ok(())
    }

    fn parse_unmask_file(&mut self, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pkg_id) = PackageId::parse(line) {
                self.masked_packages.remove(&pkg_id);
            }
        }
        Ok(())
    }

    fn parse_use_file(&mut self, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            if let Some(pkg_id) = PackageId::parse(parts[0]) {
                let flags: HashSet<String> = parts[1..]
                    .iter()
                    .filter(|s| !s.starts_with('-'))
                    .map(|s| s.to_string())
                    .collect();
                self.current_use.insert(pkg_id, flags);
            }
        }
        Ok(())
    }

    /// Analyze what needs to be unmasked to satisfy dependencies
    pub fn analyze(
        &self,
        requested: &[PackageId],
        available: &[PackageInfo],
        arch: &str,
    ) -> AutounmaskResult {
        let mut changes = Vec::new();
        let mut satisfiable = true;
        let mut resolved_packages = Vec::new();

        for pkg_id in requested {
            // Find best available version
            let candidates: Vec<_> = available
                .iter()
                .filter(|p| p.id == *pkg_id)
                .collect();

            if candidates.is_empty() {
                satisfiable = false;
                continue;
            }

            // Sort by version (newest first)
            let mut sorted_candidates = candidates.clone();
            sorted_candidates.sort_by(|a, b| b.version.cmp(&a.version));

            // Check each candidate
            for pkg in sorted_candidates {
                let instability = self.get_instability_level(pkg, arch);

                if instability > self.config.max_instability {
                    continue;
                }

                // Check if keywords need to be accepted
                if instability == InstabilityLevel::Testing {
                    let needs_keyword = !pkg.keywords.iter()
                        .any(|k| k == arch || k == &format!("~{}", arch));

                    if needs_keyword || !self.has_accepted_keywords(pkg_id, arch) {
                        changes.push(AutounmaskChange::AcceptKeywords {
                            package: pkg_id.clone(),
                            version: Some(pkg.version.clone()),
                            keywords: vec![format!("~{}", arch)],
                        });
                    }
                }

                // Check if unmasking is needed
                if instability == InstabilityLevel::Masked {
                    if self.masked_packages.contains(pkg_id) {
                        changes.push(AutounmaskChange::Unmask {
                            package: pkg_id.clone(),
                            version: Some(pkg.version.clone()),
                        });
                    }
                }

                // Check USE flag requirements
                let required_use = self.get_required_use_changes(pkg);
                if !required_use.0.is_empty() || !required_use.1.is_empty() {
                    changes.push(AutounmaskChange::UseChange {
                        package: pkg_id.clone(),
                        enable: required_use.0,
                        disable: required_use.1,
                    });
                }

                resolved_packages.push(pkg_id.clone());
                break;
            }
        }

        let explanation = if changes.is_empty() {
            "No changes required".to_string()
        } else {
            format!(
                "The following {} change(s) are required:\n{}",
                changes.len(),
                self.format_changes(&changes)
            )
        };

        AutounmaskResult {
            changes,
            packages: resolved_packages,
            satisfiable,
            explanation,
        }
    }

    fn get_instability_level(&self, pkg: &PackageInfo, arch: &str) -> InstabilityLevel {
        // Check if it's a live package
        if pkg.version.to_string().contains("9999") {
            return InstabilityLevel::Live;
        }

        // Check keywords
        let stable = pkg.keywords.iter().any(|k| k == arch);
        let testing = pkg.keywords.iter().any(|k| k == &format!("~{}", arch));

        if stable {
            InstabilityLevel::Stable
        } else if testing {
            InstabilityLevel::Testing
        } else {
            InstabilityLevel::Masked
        }
    }

    fn has_accepted_keywords(&self, pkg_id: &PackageId, arch: &str) -> bool {
        if let Some(keywords) = self.current_keywords.get(pkg_id) {
            keywords.iter().any(|k| {
                k == "**" || k == &format!("~{}", arch) || k == arch
            })
        } else {
            false
        }
    }

    fn get_required_use_changes(&self, pkg: &PackageInfo) -> (Vec<String>, Vec<String>) {
        // This would analyze REQUIRED_USE and compare with current flags
        // For now, return empty changes
        (Vec::new(), Vec::new())
    }

    fn format_changes(&self, changes: &[AutounmaskChange]) -> String {
        let mut output = String::new();

        for change in changes {
            match change {
                AutounmaskChange::AcceptKeywords { package, version, keywords } => {
                    let version_str = version
                        .as_ref()
                        .map(|v| format!("-{}", v))
                        .unwrap_or_default();
                    output.push_str(&format!(
                        "  # Accept keywords for {}\n  ={}{} {}\n",
                        package,
                        package,
                        version_str,
                        keywords.join(" ")
                    ));
                }
                AutounmaskChange::Unmask { package, version } => {
                    let version_str = version
                        .as_ref()
                        .map(|v| format!("-{}", v))
                        .unwrap_or_default();
                    output.push_str(&format!(
                        "  # Unmask {}\n  ={}{}\n",
                        package, package, version_str
                    ));
                }
                AutounmaskChange::UseChange { package, enable, disable } => {
                    let flags: Vec<String> = enable
                        .iter()
                        .map(|f| f.clone())
                        .chain(disable.iter().map(|f| format!("-{}", f)))
                        .collect();
                    output.push_str(&format!(
                        "  # USE flags for {}\n  {} {}\n",
                        package,
                        package,
                        flags.join(" ")
                    ));
                }
                AutounmaskChange::AcceptLicense { package, license } => {
                    output.push_str(&format!(
                        "  # Accept license for {}\n  {} {}\n",
                        package, package, license
                    ));
                }
            }
        }

        output
    }

    /// Write changes to configuration files
    pub fn write_changes(&self, changes: &[AutounmaskChange]) -> Result<()> {
        let mut keywords_content = String::new();
        let mut unmask_content = String::new();
        let mut use_content = String::new();

        for change in changes {
            match change {
                AutounmaskChange::AcceptKeywords { package, version, keywords } => {
                    let atom = if let Some(v) = version {
                        format!("={}-{}", package, v)
                    } else {
                        package.to_string()
                    };
                    keywords_content.push_str(&format!("{} {}\n", atom, keywords.join(" ")));
                }
                AutounmaskChange::Unmask { package, version } => {
                    let atom = if let Some(v) = version {
                        format!("={}-{}", package, v)
                    } else {
                        package.to_string()
                    };
                    unmask_content.push_str(&format!("{}\n", atom));
                }
                AutounmaskChange::UseChange { package, enable, disable } => {
                    let flags: Vec<String> = enable
                        .iter()
                        .map(|f| f.clone())
                        .chain(disable.iter().map(|f| format!("-{}", f)))
                        .collect();
                    use_content.push_str(&format!("{} {}\n", package, flags.join(" ")));
                }
                AutounmaskChange::AcceptLicense { .. } => {
                    // License acceptance would go to a different file
                }
            }
        }

        // Write files
        if !keywords_content.is_empty() {
            self.append_to_file(&self.config.keywords_path, &keywords_content)?;
        }
        if !unmask_content.is_empty() {
            self.append_to_file(&self.config.unmask_path, &unmask_content)?;
        }
        if !use_content.is_empty() {
            self.append_to_file(&self.config.use_path, &use_content)?;
        }

        Ok(())
    }

    fn append_to_file(&self, path: &PathBuf, content: &str) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(file, "\n# Added by buckos autounmask")?;
        write!(file, "{}", content)?;

        Ok(())
    }
}

impl Default for AutounmaskResolver {
    fn default() -> Self {
        Self::new(AutounmaskConfig::default())
    }
}

/// Convenience function for simple autounmask analysis
pub fn analyze_autounmask(
    requested: &[PackageId],
    available: &[PackageInfo],
    arch: &str,
) -> AutounmaskResult {
    let resolver = AutounmaskResolver::default();
    resolver.analyze(requested, available, arch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AutounmaskConfig::default();
        assert!(!config.auto_write);
        assert_eq!(config.max_instability, InstabilityLevel::Testing);
    }

    #[test]
    fn test_instability_ordering() {
        assert!(InstabilityLevel::Stable < InstabilityLevel::Testing);
        assert!(InstabilityLevel::Testing < InstabilityLevel::Masked);
        assert!(InstabilityLevel::Masked < InstabilityLevel::Live);
    }

    #[test]
    fn test_resolver_creation() {
        let resolver = AutounmaskResolver::default();
        assert!(resolver.current_keywords.is_empty());
    }
}
