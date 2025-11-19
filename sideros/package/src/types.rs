//! Core type definitions for the package manager

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// Package identifier with category and name
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PackageId {
    pub category: String,
    pub name: String,
}

impl PackageId {
    pub fn new(category: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
        }
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.category, self.name)
    }

    /// Parse a package identifier from string (e.g., "sys-apps/systemd")
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.category, self.name)
    }
}

/// Version specification with comparison operator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionSpec {
    Any,
    Exact(semver::Version),
    GreaterThan(semver::Version),
    GreaterThanOrEqual(semver::Version),
    LessThan(semver::Version),
    LessThanOrEqual(semver::Version),
    Range {
        min: Option<semver::Version>,
        max: Option<semver::Version>,
    },
}

impl VersionSpec {
    pub fn matches(&self, version: &semver::Version) -> bool {
        match self {
            VersionSpec::Any => true,
            VersionSpec::Exact(v) => version == v,
            VersionSpec::GreaterThan(v) => version > v,
            VersionSpec::GreaterThanOrEqual(v) => version >= v,
            VersionSpec::LessThan(v) => version < v,
            VersionSpec::LessThanOrEqual(v) => version <= v,
            VersionSpec::Range { min, max } => {
                let min_ok = min.as_ref().map(|m| version >= m).unwrap_or(true);
                let max_ok = max.as_ref().map(|m| version <= m).unwrap_or(true);
                min_ok && max_ok
            }
        }
    }
}

impl Default for VersionSpec {
    fn default() -> Self {
        VersionSpec::Any
    }
}

/// Package dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub package: PackageId,
    pub version: VersionSpec,
    pub slot: Option<String>,
    pub use_flags: UseCondition,
    pub optional: bool,
    pub build_time: bool,
    pub run_time: bool,
}

impl Dependency {
    pub fn new(package: PackageId) -> Self {
        Self {
            package,
            version: VersionSpec::Any,
            slot: None,
            use_flags: UseCondition::Always,
            optional: false,
            build_time: true,
            run_time: true,
        }
    }
}

/// Conditional dependency based on USE flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UseCondition {
    Always,
    IfEnabled(String),
    IfDisabled(String),
    And(Vec<UseCondition>),
    Or(Vec<UseCondition>),
}

impl UseCondition {
    pub fn evaluate(&self, enabled_flags: &HashSet<String>) -> bool {
        match self {
            UseCondition::Always => true,
            UseCondition::IfEnabled(flag) => enabled_flags.contains(flag),
            UseCondition::IfDisabled(flag) => !enabled_flags.contains(flag),
            UseCondition::And(conditions) => {
                conditions.iter().all(|c| c.evaluate(enabled_flags))
            }
            UseCondition::Or(conditions) => {
                conditions.iter().any(|c| c.evaluate(enabled_flags))
            }
        }
    }
}

/// Package information from repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub id: PackageId,
    pub version: semver::Version,
    pub slot: String,
    pub description: String,
    pub homepage: Option<String>,
    pub license: String,
    pub keywords: Vec<String>,
    pub use_flags: Vec<UseFlag>,
    pub dependencies: Vec<Dependency>,
    pub build_dependencies: Vec<Dependency>,
    pub runtime_dependencies: Vec<Dependency>,
    pub source_url: Option<String>,
    pub source_hash: Option<String>,
    pub buck_target: String,
    pub size: u64,
    pub installed_size: u64,
}

/// USE flag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseFlag {
    pub name: String,
    pub description: String,
    pub default: bool,
}

/// Installed package record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub id: PackageId,
    pub name: String,
    pub version: semver::Version,
    pub slot: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub use_flags: HashSet<String>,
    pub files: Vec<InstalledFile>,
    pub size: u64,
    pub build_time: bool,
    pub explicit: bool,
}

/// Installed file record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledFile {
    pub path: String,
    pub file_type: FileType,
    pub mode: u32,
    pub size: u64,
    pub blake3_hash: Option<String>,
    pub mtime: i64,
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    Hardlink,
    Device,
    Fifo,
}

/// Result of dependency resolution
#[derive(Debug, Clone)]
pub struct Resolution {
    pub packages: Vec<PackageInfo>,
    pub build_order: Vec<usize>,
    pub download_size: u64,
    pub install_size: u64,
}

/// Result of a build operation
#[derive(Debug, Clone)]
pub struct BuildResult {
    pub target: String,
    pub success: bool,
    pub output_path: Option<std::path::PathBuf>,
    pub duration: std::time::Duration,
    pub stdout: String,
    pub stderr: String,
}

/// Package repository definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub priority: i32,
    pub enabled: bool,
    pub sync_uri: String,
    pub buck_targets_path: String,
}

/// Package specification for user input
#[derive(Debug, Clone)]
pub struct PackageSpec {
    pub id: PackageId,
    pub version: VersionSpec,
    pub slot: Option<String>,
    pub repo: Option<String>,
}

impl PackageSpec {
    /// Parse a package specification string
    /// Examples: "sys-apps/systemd", ">=sys-apps/systemd-250", "sys-apps/systemd:0"
    pub fn parse(s: &str) -> crate::Result<Self> {
        let s = s.trim();

        // Extract version operator if present
        let (version_op, rest) = if s.starts_with(">=") {
            (Some(">="), &s[2..])
        } else if s.starts_with("<=") {
            (Some("<="), &s[2..])
        } else if s.starts_with('>') {
            (Some(">"), &s[1..])
        } else if s.starts_with('<') {
            (Some("<"), &s[1..])
        } else if s.starts_with('=') {
            (Some("="), &s[1..])
        } else if s.starts_with('~') {
            (Some("~"), &s[1..])
        } else {
            (None, s)
        };

        // Extract slot if present
        let (pkg_part, slot) = if let Some(idx) = rest.find(':') {
            (&rest[..idx], Some(rest[idx + 1..].to_string()))
        } else {
            (rest, None)
        };

        // Extract repository if present (::repo syntax)
        let (pkg_part, repo) = if let Some(idx) = pkg_part.find("::") {
            (&pkg_part[..idx], Some(pkg_part[idx + 2..].to_string()))
        } else {
            (pkg_part, None)
        };

        // Parse category/name-version
        let (id, version) = Self::parse_name_version(pkg_part, version_op)?;

        Ok(Self {
            id,
            version,
            slot,
            repo,
        })
    }

    fn parse_name_version(
        s: &str,
        version_op: Option<&str>,
    ) -> crate::Result<(PackageId, VersionSpec)> {
        // Split into category and name-version
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(crate::Error::InvalidPackageSpec(s.to_string()));
        }

        let category = parts[0].to_string();
        let name_version = parts[1];

        // Try to extract version from name (e.g., "systemd-250.4")
        if let Some(version_op) = version_op {
            // Find last dash followed by digit
            let mut last_dash = None;
            for (i, c) in name_version.char_indices() {
                if c == '-' {
                    if name_version[i + 1..].chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        last_dash = Some(i);
                    }
                }
            }

            if let Some(idx) = last_dash {
                let name = name_version[..idx].to_string();
                let version_str = &name_version[idx + 1..];
                let version = semver::Version::parse(version_str)
                    .or_else(|_| {
                        // Try parsing as simple version
                        Self::parse_simple_version(version_str)
                    })
                    .map_err(|_| crate::Error::InvalidVersion(version_str.to_string()))?;

                let version_spec = match version_op {
                    "=" | "~" => VersionSpec::Exact(version),
                    ">" => VersionSpec::GreaterThan(version),
                    ">=" => VersionSpec::GreaterThanOrEqual(version),
                    "<" => VersionSpec::LessThan(version),
                    "<=" => VersionSpec::LessThanOrEqual(version),
                    _ => VersionSpec::Any,
                };

                return Ok((PackageId::new(category, name), version_spec));
            }
        }

        // No version specified
        Ok((PackageId::new(category, name_version), VersionSpec::Any))
    }

    fn parse_simple_version(s: &str) -> Result<semver::Version, semver::Error> {
        // Handle versions like "250" or "250.4"
        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => format!("{}.0.0", parts[0]).parse(),
            2 => format!("{}.{}.0", parts[0], parts[1]).parse(),
            _ => s.parse(),
        }
    }
}

/// World set - explicitly installed packages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldSet {
    pub packages: HashSet<PackageId>,
}

/// USE flag configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UseConfig {
    pub global: HashSet<String>,
    pub package: BTreeMap<PackageId, HashSet<String>>,
}

impl UseConfig {
    pub fn get_flags(&self, pkg: &PackageId) -> HashSet<String> {
        let mut flags = self.global.clone();
        if let Some(pkg_flags) = self.package.get(pkg) {
            flags.extend(pkg_flags.clone());
        }
        flags
    }
}
