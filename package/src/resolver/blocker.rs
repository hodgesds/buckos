//! Package blocker resolution
//!
//! Handles hard blockers (!!atom) and soft blockers (!atom) between packages.

use crate::{Error, PackageId, PackageInfo, Result, VersionSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Type of package blocker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockerType {
    /// Hard blocker (!!atom) - packages cannot coexist at all
    Hard,
    /// Soft blocker (!atom) - packages can coexist during upgrade
    Soft,
}

/// A package blocker definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocker {
    /// Package that declares the blocker
    pub package: PackageId,
    /// Version of the package declaring the blocker
    pub version: semver::Version,
    /// Package being blocked
    pub blocked: PackageId,
    /// Version constraint for the blocked package
    pub blocked_version: VersionSpec,
    /// Type of blocker
    pub blocker_type: BlockerType,
    /// Reason for the blocker
    pub reason: Option<String>,
}

/// Result of blocker resolution
#[derive(Debug, Clone)]
pub struct BlockerResolution {
    /// Blockers that can be resolved
    pub resolved: Vec<ResolvedBlocker>,
    /// Blockers that cannot be resolved
    pub unresolved: Vec<UnresolvedBlocker>,
}

/// A resolved blocker with action
#[derive(Debug, Clone)]
pub struct ResolvedBlocker {
    /// The original blocker
    pub blocker: Blocker,
    /// Action to resolve the blocker
    pub action: BlockerAction,
}

/// An unresolved blocker that requires user intervention
#[derive(Debug, Clone)]
pub struct UnresolvedBlocker {
    /// The original blocker
    pub blocker: Blocker,
    /// Why it cannot be resolved
    pub reason: String,
}

/// Action to take to resolve a blocker
#[derive(Debug, Clone)]
pub enum BlockerAction {
    /// Remove the blocking package
    Remove(PackageId),
    /// Upgrade the blocking package to a non-conflicting version
    Upgrade {
        package: PackageId,
        to_version: semver::Version,
    },
    /// Downgrade the package to avoid the blocker
    Downgrade {
        package: PackageId,
        to_version: semver::Version,
    },
    /// Install packages in a specific order to avoid conflict
    OrderedInstall { first: PackageId, second: PackageId },
}

/// Blocker resolver
pub struct BlockerResolver {
    /// Known blockers
    blockers: Vec<Blocker>,
}

impl BlockerResolver {
    /// Create a new blocker resolver
    pub fn new() -> Self {
        Self {
            blockers: Vec::new(),
        }
    }

    /// Add a blocker
    pub fn add_blocker(&mut self, blocker: Blocker) {
        self.blockers.push(blocker);
    }

    /// Parse a blocker string (e.g., "!!sys-apps/systemd" or "!<sys-apps/openrc-0.45")
    pub fn parse_blocker(
        s: &str,
        source_pkg: &PackageId,
        source_version: &semver::Version,
    ) -> Result<Blocker> {
        let s = s.trim();

        let (blocker_type, rest) = if s.starts_with("!!") {
            (BlockerType::Hard, &s[2..])
        } else if s.starts_with('!') {
            (BlockerType::Soft, &s[1..])
        } else {
            return Err(Error::InvalidBlocker(s.to_string()));
        };

        // Parse version operator if present
        let (version_spec, pkg_str) = if let Some(pkg_str) = rest.strip_prefix(">=") {
            (Self::parse_versioned_atom(pkg_str, ">=")?, pkg_str)
        } else if let Some(pkg_str) = rest.strip_prefix("<=") {
            (Self::parse_versioned_atom(pkg_str, "<=")?, pkg_str)
        } else if let Some(pkg_str) = rest.strip_prefix('>') {
            (Self::parse_versioned_atom(pkg_str, ">")?, pkg_str)
        } else if let Some(pkg_str) = rest.strip_prefix('<') {
            (Self::parse_versioned_atom(pkg_str, "<")?, pkg_str)
        } else if let Some(pkg_str) = rest.strip_prefix('=') {
            (Self::parse_versioned_atom(pkg_str, "=")?, pkg_str)
        } else {
            (VersionSpec::Any, rest)
        };

        // Parse package ID
        let blocked = PackageId::parse(
            pkg_str
                .split('-')
                .take(2)
                .collect::<Vec<_>>()
                .join("/")
                .as_str(),
        )
        .ok_or_else(|| Error::InvalidBlocker(s.to_string()))?;

        Ok(Blocker {
            package: source_pkg.clone(),
            version: source_version.clone(),
            blocked,
            blocked_version: version_spec,
            blocker_type,
            reason: None,
        })
    }

    fn parse_versioned_atom(s: &str, op: &str) -> Result<VersionSpec> {
        // Find version part (after last hyphen followed by digit)
        let mut last_dash = None;
        for (i, c) in s.char_indices() {
            if c == '-'
                && s[i + 1..]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                last_dash = Some(i);
            }
        }

        if let Some(idx) = last_dash {
            let version_str = &s[idx + 1..];
            let version = Self::parse_version(version_str)?;

            Ok(match op {
                "=" => VersionSpec::Exact(version),
                ">" => VersionSpec::GreaterThan(version),
                ">=" => VersionSpec::GreaterThanOrEqual(version),
                "<" => VersionSpec::LessThan(version),
                "<=" => VersionSpec::LessThanOrEqual(version),
                _ => VersionSpec::Any,
            })
        } else {
            Ok(VersionSpec::Any)
        }
    }

    fn parse_version(s: &str) -> Result<semver::Version> {
        semver::Version::parse(s).or_else(|_| {
            // Try parsing simple versions like "250" or "250.4"
            let parts: Vec<&str> = s.split('.').collect();
            let version_str = match parts.len() {
                1 => format!("{}.0.0", parts[0]),
                2 => format!("{}.{}.0", parts[0], parts[1]),
                _ => s.to_string(),
            };
            version_str
                .parse()
                .map_err(|_| Error::InvalidVersion(s.to_string()))
        })
    }

    /// Check for blockers in a set of packages to install
    pub fn check_blockers(
        &self,
        to_install: &[PackageInfo],
        installed: &[PackageInfo],
    ) -> Vec<Blocker> {
        let mut active_blockers = Vec::new();
        let install_ids: HashSet<_> = to_install.iter().map(|p| &p.id).collect();
        let installed_ids: HashSet<_> = installed.iter().map(|p| &p.id).collect();

        for blocker in &self.blockers {
            // Check if the blocking package is being installed or is installed
            let blocker_active =
                install_ids.contains(&blocker.package) || installed_ids.contains(&blocker.package);

            if !blocker_active {
                continue;
            }

            // Check if the blocked package is being installed or is installed
            let blocked_present = to_install
                .iter()
                .any(|p| p.id == blocker.blocked && blocker.blocked_version.matches(&p.version))
                || installed
                    .iter()
                    .find(|p| {
                        p.id == blocker.blocked && blocker.blocked_version.matches(&p.version)
                    })
                    .is_some();

            if blocked_present {
                active_blockers.push(blocker.clone());
            }
        }

        active_blockers
    }

    /// Resolve blockers automatically where possible
    pub fn resolve_blockers(
        &self,
        blockers: &[Blocker],
        to_install: &[PackageInfo],
        installed: &[PackageInfo],
        available: &[PackageInfo],
    ) -> BlockerResolution {
        let mut resolved = Vec::new();
        let mut unresolved = Vec::new();

        for blocker in blockers {
            match self.try_resolve_blocker(blocker, to_install, installed, available) {
                Some(action) => {
                    resolved.push(ResolvedBlocker {
                        blocker: blocker.clone(),
                        action,
                    });
                }
                None => {
                    let reason = match blocker.blocker_type {
                        BlockerType::Hard => {
                            format!(
                                "Hard blocker: {} cannot coexist with {}",
                                blocker.package, blocker.blocked
                            )
                        }
                        BlockerType::Soft => {
                            format!(
                                "Soft blocker: {} conflicts with {} during installation",
                                blocker.package, blocker.blocked
                            )
                        }
                    };
                    unresolved.push(UnresolvedBlocker {
                        blocker: blocker.clone(),
                        reason,
                    });
                }
            }
        }

        BlockerResolution {
            resolved,
            unresolved,
        }
    }

    fn try_resolve_blocker(
        &self,
        blocker: &Blocker,
        to_install: &[PackageInfo],
        installed: &[PackageInfo],
        available: &[PackageInfo],
    ) -> Option<BlockerAction> {
        // For soft blockers, try ordered installation
        if blocker.blocker_type == BlockerType::Soft {
            // Check if both packages are being installed
            let pkg_installing = to_install.iter().any(|p| p.id == blocker.package);
            let blocked_installing = to_install.iter().any(|p| p.id == blocker.blocked);

            if pkg_installing && blocked_installing {
                return Some(BlockerAction::OrderedInstall {
                    first: blocker.package.clone(),
                    second: blocker.blocked.clone(),
                });
            }
        }

        // Try to find a non-conflicting version
        let non_conflicting = available
            .iter()
            .filter(|p| p.id == blocker.blocked)
            .find(|p| !blocker.blocked_version.matches(&p.version));

        if let Some(pkg) = non_conflicting {
            let current = installed
                .iter()
                .find(|p| p.id == blocker.blocked)
                .or_else(|| to_install.iter().find(|p| p.id == blocker.blocked));

            if let Some(current_pkg) = current {
                if pkg.version > current_pkg.version {
                    return Some(BlockerAction::Upgrade {
                        package: pkg.id.clone(),
                        to_version: pkg.version.clone(),
                    });
                } else {
                    return Some(BlockerAction::Downgrade {
                        package: pkg.id.clone(),
                        to_version: pkg.version.clone(),
                    });
                }
            }
        }

        // For hard blockers where blocked package is installed, suggest removal
        if blocker.blocker_type == BlockerType::Hard {
            let is_installed = installed
                .iter()
                .any(|p| p.id == blocker.blocked && blocker.blocked_version.matches(&p.version));

            if is_installed {
                return Some(BlockerAction::Remove(blocker.blocked.clone()));
            }
        }

        None
    }

    /// Extract blockers from a package's dependencies
    ///
    /// This parses blocker strings (e.g., `!sys-apps/openrc`, `!!sys-apps/sysvinit`)
    /// from the package's blockers field and registers them with this resolver.
    pub fn extract_blockers_from_package(&mut self, pkg: &PackageInfo) {
        for blocker_str in &pkg.blockers {
            if let Ok(blocker) = Self::parse_blocker(blocker_str, &pkg.id, &pkg.version) {
                self.add_blocker(blocker);
            }
        }
    }
}

impl Default for BlockerResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a dependency string is a blocker
pub fn is_blocker(dep: &str) -> bool {
    dep.trim().starts_with('!')
}

/// Check if a dependency string is a hard blocker
pub fn is_hard_blocker(dep: &str) -> bool {
    dep.trim().starts_with("!!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_soft_blocker() {
        let pkg_id = PackageId::new("sys-apps", "systemd");
        let version = semver::Version::new(250, 0, 0);

        let blocker =
            BlockerResolver::parse_blocker("!sys-apps/openrc", &pkg_id, &version).unwrap();

        assert_eq!(blocker.blocker_type, BlockerType::Soft);
        assert_eq!(blocker.blocked.category, "sys-apps");
    }

    #[test]
    fn test_parse_hard_blocker() {
        let pkg_id = PackageId::new("sys-apps", "systemd");
        let version = semver::Version::new(250, 0, 0);

        let blocker =
            BlockerResolver::parse_blocker("!!sys-apps/sysvinit", &pkg_id, &version).unwrap();

        assert_eq!(blocker.blocker_type, BlockerType::Hard);
    }

    #[test]
    fn test_is_blocker() {
        assert!(is_blocker("!sys-apps/openrc"));
        assert!(is_blocker("!!sys-apps/sysvinit"));
        assert!(!is_blocker("sys-apps/systemd"));
    }

    #[test]
    fn test_is_hard_blocker() {
        assert!(!is_hard_blocker("!sys-apps/openrc"));
        assert!(is_hard_blocker("!!sys-apps/sysvinit"));
    }
}
