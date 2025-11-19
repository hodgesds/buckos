//! Package catalog with system package definitions
//!
//! This module provides definitions for common system packages including
//! core libraries, toolchains, build systems, utilities, and services.

pub mod categories;
pub mod core;
pub mod toolchain;
pub mod build;
pub mod utils;
pub mod services;
pub mod network;
pub mod compression;

use crate::types::{PackageId, PackageInfo, Dependency, UseFlag, VersionSpec, UseCondition};
use std::collections::HashMap;
use semver::Version;

/// Package catalog containing all known package definitions
pub struct PackageCatalog {
    packages: HashMap<PackageId, Vec<PackageInfo>>,
}

impl PackageCatalog {
    /// Create a new package catalog with built-in system packages
    pub fn new() -> Self {
        let mut catalog = Self {
            packages: HashMap::new(),
        };

        // Load all built-in packages
        catalog.load_core_packages();
        catalog.load_toolchain_packages();
        catalog.load_build_packages();
        catalog.load_utils_packages();
        catalog.load_services_packages();
        catalog.load_network_packages();
        catalog.load_compression_packages();

        catalog
    }

    /// Add a package to the catalog
    pub fn add_package(&mut self, info: PackageInfo) {
        self.packages
            .entry(info.id.clone())
            .or_insert_with(Vec::new)
            .push(info);
    }

    /// Get all versions of a package
    pub fn get_package(&self, id: &PackageId) -> Option<&Vec<PackageInfo>> {
        self.packages.get(id)
    }

    /// Get the latest version of a package
    pub fn get_latest(&self, id: &PackageId) -> Option<&PackageInfo> {
        self.packages.get(id).and_then(|versions| {
            versions.iter().max_by(|a, b| a.version.cmp(&b.version))
        })
    }

    /// Get a specific version of a package
    pub fn get_version(&self, id: &PackageId, version: &Version) -> Option<&PackageInfo> {
        self.packages.get(id).and_then(|versions| {
            versions.iter().find(|p| &p.version == version)
        })
    }

    /// Search packages by name or description
    pub fn search(&self, query: &str) -> Vec<&PackageInfo> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&PackageInfo> = self.packages
            .values()
            .flatten()
            .filter(|p| {
                p.id.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .collect();

        // Sort by relevance (exact name match first)
        results.sort_by(|a, b| {
            let a_exact = a.id.name.to_lowercase() == query_lower;
            let b_exact = b.id.name.to_lowercase() == query_lower;
            b_exact.cmp(&a_exact)
        });

        results
    }

    /// Get all packages in a category
    pub fn get_category(&self, category: &str) -> Vec<&PackageInfo> {
        self.packages
            .iter()
            .filter(|(id, _)| id.category == category)
            .flat_map(|(_, versions)| versions.iter())
            .collect()
    }

    /// Get total number of packages
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Check if catalog is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Get all package IDs
    pub fn package_ids(&self) -> impl Iterator<Item = &PackageId> {
        self.packages.keys()
    }

    // Load functions for each package category
    fn load_core_packages(&mut self) {
        for pkg in core::get_packages() {
            self.add_package(pkg);
        }
    }

    fn load_toolchain_packages(&mut self) {
        for pkg in toolchain::get_packages() {
            self.add_package(pkg);
        }
    }

    fn load_build_packages(&mut self) {
        for pkg in build::get_packages() {
            self.add_package(pkg);
        }
    }

    fn load_utils_packages(&mut self) {
        for pkg in utils::get_packages() {
            self.add_package(pkg);
        }
    }

    fn load_services_packages(&mut self) {
        for pkg in services::get_packages() {
            self.add_package(pkg);
        }
    }

    fn load_network_packages(&mut self) {
        for pkg in network::get_packages() {
            self.add_package(pkg);
        }
    }

    fn load_compression_packages(&mut self) {
        for pkg in compression::get_packages() {
            self.add_package(pkg);
        }
    }
}

impl Default for PackageCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a basic dependency
pub fn dep(category: &str, name: &str) -> Dependency {
    Dependency::new(PackageId::new(category, name))
}

/// Helper function to create a dependency with version requirement
pub fn dep_version(category: &str, name: &str, version_spec: VersionSpec) -> Dependency {
    let mut d = Dependency::new(PackageId::new(category, name));
    d.version = version_spec;
    d
}

/// Helper function to create a build-time only dependency
pub fn dep_build(category: &str, name: &str) -> Dependency {
    let mut d = Dependency::new(PackageId::new(category, name));
    d.build_time = true;
    d.run_time = false;
    d
}

/// Helper function to create a runtime only dependency
pub fn dep_runtime(category: &str, name: &str) -> Dependency {
    let mut d = Dependency::new(PackageId::new(category, name));
    d.build_time = false;
    d.run_time = true;
    d
}

/// Helper function to create an optional dependency
pub fn dep_optional(category: &str, name: &str) -> Dependency {
    let mut d = Dependency::new(PackageId::new(category, name));
    d.optional = true;
    d
}

/// Helper function to create a USE-flag conditional dependency
pub fn dep_use(category: &str, name: &str, flag: &str) -> Dependency {
    let mut d = Dependency::new(PackageId::new(category, name));
    d.use_flags = UseCondition::IfEnabled(flag.to_string());
    d
}

/// Helper function to create a USE flag definition
pub fn use_flag(name: &str, description: &str, default: bool) -> UseFlag {
    UseFlag {
        name: name.to_string(),
        description: description.to_string(),
        default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_creation() {
        let catalog = PackageCatalog::new();
        assert!(!catalog.is_empty());
    }

    #[test]
    fn test_search() {
        let catalog = PackageCatalog::new();
        let results = catalog.search("glibc");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_get_category() {
        let catalog = PackageCatalog::new();
        let sys_libs = catalog.get_category("sys-libs");
        assert!(!sys_libs.is_empty());
    }
}
