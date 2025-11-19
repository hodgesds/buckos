//! FEATURES configuration
//!
//! Implements Gentoo-style FEATURES variable:
//! - Build features (sandbox, parallel, etc.)
//! - Testing features
//! - Binary package features

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// Enabled features
    pub enabled: HashSet<String>,
    /// Disabled features (prefixed with -)
    pub disabled: HashSet<String>,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            enabled: default_features(),
            disabled: HashSet::new(),
        }
    }
}

impl FeaturesConfig {
    /// Create a new features configuration
    pub fn new() -> Self {
        Self {
            enabled: HashSet::new(),
            disabled: HashSet::new(),
        }
    }

    /// Create with default features
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// Enable a feature
    pub fn enable(&mut self, feature: impl Into<String>) {
        let feature = feature.into();
        self.disabled.remove(&feature);
        self.enabled.insert(feature);
    }

    /// Disable a feature
    pub fn disable(&mut self, feature: impl Into<String>) {
        let feature = feature.into();
        self.enabled.remove(&feature);
        self.disabled.insert(feature);
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.enabled.contains(feature) && !self.disabled.contains(feature)
    }

    /// Check if a feature is explicitly disabled
    pub fn is_disabled(&self, feature: &str) -> bool {
        self.disabled.contains(feature)
    }

    /// Parse a FEATURES string
    pub fn parse(s: &str) -> Self {
        let mut config = Self::new();

        for part in s.split_whitespace() {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part.starts_with('-') {
                config.disable(&part[1..]);
            } else {
                config.enable(part);
            }
        }

        config
    }

    /// Format as FEATURES string
    pub fn format(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Add enabled features
        let mut enabled: Vec<_> = self.enabled.iter().collect();
        enabled.sort();
        parts.extend(enabled.into_iter().cloned());

        // Add disabled features
        let mut disabled: Vec<_> = self.disabled.iter().collect();
        disabled.sort();
        parts.extend(disabled.into_iter().map(|f| format!("-{}", f)));

        parts.join(" ")
    }

    /// Merge with another features configuration
    pub fn merge(&mut self, other: &FeaturesConfig) {
        for feature in &other.enabled {
            self.enable(feature.clone());
        }
        for feature in &other.disabled {
            self.disable(feature.clone());
        }
    }

    /// Get all effective features
    pub fn effective(&self) -> HashSet<&str> {
        self.enabled
            .iter()
            .filter(|f| !self.disabled.contains(*f))
            .map(|s| s.as_str())
            .collect()
    }
}

/// Default features for Buckos
pub fn default_features() -> HashSet<String> {
    [
        // Safety features
        "sandbox",
        "usersandbox",
        "network-sandbox",
        "ipc-sandbox",
        "pid-sandbox",
        // Build features
        "parallel-fetch",
        "parallel-install",
        "distlocks",
        "ebuild-locks",
        // Package handling
        "buildpkg",
        "binpkg-logs",
        "binpkg-multi-instance",
        // Protection
        "config-protect-if-modified",
        "protect-owned",
        "preserve-libs",
        // Logging
        "clean-logs",
        "split-elog",
        // Quality
        "strict",
        "unknown-features-warn",
        "qa-unresolved-soname-deps",
        // Misc
        "merge-sync",
        "multilib-strict",
        "unmerge-orphans",
        "fixlafiles",
        "sfperms",
        "news",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// All available features
pub fn all_features() -> Vec<FeatureInfo> {
    vec![
        // Sandbox features
        FeatureInfo {
            name: "sandbox".to_string(),
            category: FeatureCategory::Security,
            description: "Enable sandbox for builds".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "usersandbox".to_string(),
            category: FeatureCategory::Security,
            description: "Enable sandbox when running as non-root".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "network-sandbox".to_string(),
            category: FeatureCategory::Security,
            description: "Disable network access during build".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "ipc-sandbox".to_string(),
            category: FeatureCategory::Security,
            description: "Enable IPC namespace sandbox".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "pid-sandbox".to_string(),
            category: FeatureCategory::Security,
            description: "Enable PID namespace sandbox".to_string(),
            default: true,
        },
        // Build features
        FeatureInfo {
            name: "parallel-fetch".to_string(),
            category: FeatureCategory::Performance,
            description: "Fetch files in parallel".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "parallel-install".to_string(),
            category: FeatureCategory::Performance,
            description: "Install packages in parallel".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "distlocks".to_string(),
            category: FeatureCategory::Build,
            description: "Use locks for distfile access".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "ebuild-locks".to_string(),
            category: FeatureCategory::Build,
            description: "Use locks for ebuild access".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "keepwork".to_string(),
            category: FeatureCategory::Build,
            description: "Keep work directory after build".to_string(),
            default: false,
        },
        FeatureInfo {
            name: "keeptemp".to_string(),
            category: FeatureCategory::Build,
            description: "Keep temp directory after build".to_string(),
            default: false,
        },
        FeatureInfo {
            name: "nostrip".to_string(),
            category: FeatureCategory::Build,
            description: "Do not strip binaries".to_string(),
            default: false,
        },
        FeatureInfo {
            name: "splitdebug".to_string(),
            category: FeatureCategory::Build,
            description: "Split debug symbols into separate files".to_string(),
            default: false,
        },
        FeatureInfo {
            name: "compressdebug".to_string(),
            category: FeatureCategory::Build,
            description: "Compress debug sections".to_string(),
            default: false,
        },
        // Binary packages
        FeatureInfo {
            name: "buildpkg".to_string(),
            category: FeatureCategory::BinaryPkg,
            description: "Build binary packages".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "binpkg-logs".to_string(),
            category: FeatureCategory::BinaryPkg,
            description: "Include build logs in binary packages".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "binpkg-multi-instance".to_string(),
            category: FeatureCategory::BinaryPkg,
            description: "Allow multiple binary package instances".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "getbinpkg".to_string(),
            category: FeatureCategory::BinaryPkg,
            description: "Fetch binary packages from remote".to_string(),
            default: false,
        },
        // Testing
        FeatureInfo {
            name: "test".to_string(),
            category: FeatureCategory::Testing,
            description: "Run package tests".to_string(),
            default: false,
        },
        FeatureInfo {
            name: "test-fail-continue".to_string(),
            category: FeatureCategory::Testing,
            description: "Continue on test failures".to_string(),
            default: false,
        },
        // Protection
        FeatureInfo {
            name: "protect-owned".to_string(),
            category: FeatureCategory::Protection,
            description: "Protect files owned by other packages".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "preserve-libs".to_string(),
            category: FeatureCategory::Protection,
            description: "Preserve old libraries during upgrade".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "config-protect-if-modified".to_string(),
            category: FeatureCategory::Protection,
            description: "Only protect modified config files".to_string(),
            default: true,
        },
        // Logging
        FeatureInfo {
            name: "clean-logs".to_string(),
            category: FeatureCategory::Logging,
            description: "Clean old log files".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "split-elog".to_string(),
            category: FeatureCategory::Logging,
            description: "Split elog by category".to_string(),
            default: true,
        },
        // Caching
        FeatureInfo {
            name: "ccache".to_string(),
            category: FeatureCategory::Caching,
            description: "Use ccache for compilation".to_string(),
            default: false,
        },
        FeatureInfo {
            name: "sccache".to_string(),
            category: FeatureCategory::Caching,
            description: "Use sccache for compilation".to_string(),
            default: false,
        },
        // Quality
        FeatureInfo {
            name: "strict".to_string(),
            category: FeatureCategory::Quality,
            description: "Enable strict mode".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "qa-unresolved-soname-deps".to_string(),
            category: FeatureCategory::Quality,
            description: "Warn about unresolved soname deps".to_string(),
            default: true,
        },
        FeatureInfo {
            name: "unknown-features-warn".to_string(),
            category: FeatureCategory::Quality,
            description: "Warn about unknown features".to_string(),
            default: true,
        },
        // Misc
        FeatureInfo {
            name: "candy".to_string(),
            category: FeatureCategory::Misc,
            description: "Enable visual candy in output".to_string(),
            default: false,
        },
        FeatureInfo {
            name: "news".to_string(),
            category: FeatureCategory::Misc,
            description: "Show news items".to_string(),
            default: true,
        },
    ]
}

/// Information about a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    /// Feature name
    pub name: String,
    /// Feature category
    pub category: FeatureCategory,
    /// Description
    pub description: String,
    /// Default enabled
    pub default: bool,
}

/// Feature categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureCategory {
    /// Security features
    Security,
    /// Performance features
    Performance,
    /// Build control
    Build,
    /// Binary packages
    BinaryPkg,
    /// Testing
    Testing,
    /// Protection
    Protection,
    /// Logging
    Logging,
    /// Caching
    Caching,
    /// Quality assurance
    Quality,
    /// Miscellaneous
    Misc,
}

impl std::fmt::Display for FeatureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureCategory::Security => write!(f, "Security"),
            FeatureCategory::Performance => write!(f, "Performance"),
            FeatureCategory::Build => write!(f, "Build"),
            FeatureCategory::BinaryPkg => write!(f, "Binary Packages"),
            FeatureCategory::Testing => write!(f, "Testing"),
            FeatureCategory::Protection => write!(f, "Protection"),
            FeatureCategory::Logging => write!(f, "Logging"),
            FeatureCategory::Caching => write!(f, "Caching"),
            FeatureCategory::Quality => write!(f, "Quality"),
            FeatureCategory::Misc => write!(f, "Miscellaneous"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_features_config() {
        let mut config = FeaturesConfig::with_defaults();
        assert!(config.is_enabled("sandbox"));

        config.disable("sandbox");
        assert!(!config.is_enabled("sandbox"));
        assert!(config.is_disabled("sandbox"));
    }

    #[test]
    fn test_parse_features() {
        let config = FeaturesConfig::parse("sandbox parallel-fetch -test ccache");
        assert!(config.is_enabled("sandbox"));
        assert!(config.is_enabled("parallel-fetch"));
        assert!(config.is_disabled("test"));
        assert!(config.is_enabled("ccache"));
    }

    #[test]
    fn test_format_features() {
        let mut config = FeaturesConfig::new();
        config.enable("sandbox");
        config.enable("ccache");
        config.disable("test");

        let formatted = config.format();
        assert!(formatted.contains("sandbox"));
        assert!(formatted.contains("ccache"));
        assert!(formatted.contains("-test"));
    }

    #[test]
    fn test_default_features() {
        let features = default_features();
        assert!(features.contains("sandbox"));
        assert!(features.contains("parallel-fetch"));
        assert!(!features.contains("test")); // test is not default
    }
}
