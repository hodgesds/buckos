//! Package validation and integrity checking
//!
//! This module provides functions for validating packages including:
//! - Source hash verification
//! - Dependency validation
//! - Package metadata validation
//! - Circular dependency detection
//! - Version constraint checking

use crate::catalog::PackageCatalog;
use crate::error::{Error, Result};
use crate::types::{Dependency, PackageId, PackageInfo, VersionSpec};
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// Package validator for checking package integrity and consistency
pub struct PackageValidator {
    catalog: PackageCatalog,
}

impl PackageValidator {
    /// Create a new package validator with the given catalog
    pub fn new(catalog: PackageCatalog) -> Self {
        Self { catalog }
    }

    /// Validate all packages in the catalog
    pub fn validate_all(&self) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for pkg_id in self.catalog.package_ids() {
            if let Some(versions) = self.catalog.get_package(pkg_id) {
                for pkg in versions {
                    results.push(self.validate_package(pkg));
                }
            }
        }

        results
    }

    /// Validate a single package
    pub fn validate_package(&self, pkg: &PackageInfo) -> ValidationResult {
        let mut issues = Vec::new();

        // Validate package ID
        if let Err(e) = self.validate_package_id(&pkg.id) {
            issues.push(ValidationIssue::Error(e.to_string()));
        }

        // Validate version
        if let Err(e) = self.validate_version(pkg) {
            issues.push(ValidationIssue::Error(e.to_string()));
        }

        // Validate dependencies
        for issue in self.validate_dependencies(pkg) {
            issues.push(issue);
        }

        // Validate source URL and hash
        if let Err(e) = self.validate_source(pkg) {
            issues.push(ValidationIssue::Warning(e));
        }

        // Validate buck target
        if let Err(e) = self.validate_buck_target(pkg) {
            issues.push(ValidationIssue::Warning(e));
        }

        // Validate license
        if let Err(e) = self.validate_license(pkg) {
            issues.push(ValidationIssue::Warning(e));
        }

        // Validate keywords
        for issue in self.validate_keywords(pkg) {
            issues.push(issue);
        }

        ValidationResult {
            package: pkg.id.clone(),
            version: pkg.version.clone(),
            issues,
        }
    }

    /// Validate package ID format
    fn validate_package_id(&self, id: &PackageId) -> Result<()> {
        if id.category.is_empty() {
            return Err(Error::InvalidPackageSpec("Empty category".to_string()));
        }

        if id.name.is_empty() {
            return Err(Error::InvalidPackageSpec("Empty package name".to_string()));
        }

        // Check for valid characters
        let valid_chars = |s: &str| {
            s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '+')
        };

        if !valid_chars(&id.category) {
            return Err(Error::InvalidPackageSpec(format!(
                "Invalid characters in category: {}",
                id.category
            )));
        }

        if !valid_chars(&id.name) {
            return Err(Error::InvalidPackageSpec(format!(
                "Invalid characters in package name: {}",
                id.name
            )));
        }

        Ok(())
    }

    /// Validate package version
    fn validate_version(&self, pkg: &PackageInfo) -> Result<()> {
        // Check that version is not 0.0.0
        if pkg.version.major == 0 && pkg.version.minor == 0 && pkg.version.patch == 0 {
            return Err(Error::InvalidVersion("Version cannot be 0.0.0".to_string()));
        }

        Ok(())
    }

    /// Validate package dependencies
    fn validate_dependencies(&self, pkg: &PackageInfo) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check all dependency types
        let all_deps: Vec<&Dependency> = pkg
            .dependencies
            .iter()
            .chain(pkg.build_dependencies.iter())
            .chain(pkg.runtime_dependencies.iter())
            .collect();

        for dep in all_deps {
            // Check if dependency exists in catalog
            if self.catalog.get_package(&dep.package).is_none() {
                issues.push(ValidationIssue::Warning(format!(
                    "Dependency not found in catalog: {}",
                    dep.package
                )));
            }

            // Check for self-dependency
            if dep.package == pkg.id {
                issues.push(ValidationIssue::Error(format!(
                    "Package depends on itself: {}",
                    pkg.id
                )));
            }
        }

        issues
    }

    /// Validate source URL and hash
    fn validate_source(&self, pkg: &PackageInfo) -> std::result::Result<(), String> {
        // If source URL is provided, hash should also be provided
        if pkg.source_url.is_some() && pkg.source_hash.is_none() {
            return Err(format!("Package {} has source URL but no hash", pkg.id));
        }

        // Validate hash format (should be hex)
        if let Some(hash) = &pkg.source_hash {
            if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!("Invalid hash format for {}: {}", pkg.id, hash));
            }

            // SHA256 hash should be 64 characters
            if hash.len() != 64 {
                return Err(format!(
                    "Hash length should be 64 for SHA256, got {} for {}",
                    hash.len(),
                    pkg.id
                ));
            }
        }

        Ok(())
    }

    /// Validate buck target format
    fn validate_buck_target(&self, pkg: &PackageInfo) -> std::result::Result<(), String> {
        if pkg.buck_target.is_empty() {
            return Err(format!("Empty buck target for {}", pkg.id));
        }

        // Buck target should start with //
        if !pkg.buck_target.starts_with("//") {
            return Err(format!(
                "Buck target should start with //: {}",
                pkg.buck_target
            ));
        }

        // Should contain :
        if !pkg.buck_target.contains(':') {
            return Err(format!(
                "Buck target should contain target name after ':'): {}",
                pkg.buck_target
            ));
        }

        Ok(())
    }

    /// Validate license field
    fn validate_license(&self, pkg: &PackageInfo) -> std::result::Result<(), String> {
        if pkg.license.is_empty() {
            return Err(format!("Empty license for {}", pkg.id));
        }

        Ok(())
    }

    /// Validate keywords
    fn validate_keywords(&self, pkg: &PackageInfo) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if pkg.keywords.is_empty() {
            issues.push(ValidationIssue::Warning(format!(
                "No keywords defined for {}",
                pkg.id
            )));
        }

        // Valid architectures
        let valid_arches = ["amd64", "arm64", "x86", "arm", "ppc64", "riscv64", "s390x"];

        for keyword in &pkg.keywords {
            let arch = keyword.trim_start_matches('~');
            if !valid_arches.contains(&arch) {
                issues.push(ValidationIssue::Warning(format!(
                    "Unknown architecture keyword '{}' for {}",
                    keyword, pkg.id
                )));
            }
        }

        issues
    }

    /// Check for circular dependencies in the entire catalog
    pub fn check_circular_dependencies(&self) -> Vec<Vec<PackageId>> {
        let mut graph: DiGraph<PackageId, ()> = DiGraph::new();
        let mut node_map: HashMap<PackageId, NodeIndex> = HashMap::new();

        // Build graph
        for pkg_id in self.catalog.package_ids() {
            if let Some(versions) = self.catalog.get_package(pkg_id) {
                for pkg in versions {
                    let node = *node_map
                        .entry(pkg.id.clone())
                        .or_insert_with(|| graph.add_node(pkg.id.clone()));

                    for dep in &pkg.dependencies {
                        let dep_node = *node_map
                            .entry(dep.package.clone())
                            .or_insert_with(|| graph.add_node(dep.package.clone()));
                        graph.add_edge(node, dep_node, ());
                    }
                }
            }
        }

        // Find cycles
        if is_cyclic_directed(&graph) {
            // Return the cycles found
            // For simplicity, we just report that cycles exist
            // A more sophisticated implementation would identify specific cycles
            vec![vec![]] // Placeholder indicating cycles exist
        } else {
            vec![]
        }
    }

    /// Validate version constraints between packages
    pub fn validate_version_constraints(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for pkg_id in self.catalog.package_ids() {
            if let Some(versions) = self.catalog.get_package(pkg_id) {
                for pkg in versions {
                    for dep in &pkg.dependencies {
                        if let VersionSpec::Exact(required_version) = &dep.version {
                            // Check if the required version exists
                            if let Some(dep_versions) = self.catalog.get_package(&dep.package) {
                                let exists =
                                    dep_versions.iter().any(|v| &v.version == required_version);

                                if !exists {
                                    issues.push(ValidationIssue::Error(format!(
                                        "{} requires {} version {} which doesn't exist",
                                        pkg.id, dep.package, required_version
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        issues
    }
}

/// Verify a file's SHA256 hash
pub async fn verify_sha256(path: &Path, expected_hash: &str) -> Result<bool> {
    let contents = fs::read(path).await?;

    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    let actual_hash = hex::encode(result);

    Ok(actual_hash == expected_hash.to_lowercase())
}

/// Verify a file's BLAKE3 hash
pub fn verify_blake3(path: &Path, expected_hash: &str) -> Result<bool> {
    let contents = std::fs::read(path)?;

    let actual_hash = blake3::hash(&contents).to_hex().to_string();
    Ok(actual_hash == expected_hash.to_lowercase())
}

/// Compute SHA256 hash of a file
pub async fn compute_sha256(path: &Path) -> Result<String> {
    let contents = fs::read(path).await?;

    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Compute BLAKE3 hash of a file
pub fn compute_blake3(path: &Path) -> Result<String> {
    let contents = std::fs::read(path)?;

    Ok(blake3::hash(&contents).to_hex().to_string())
}

/// Result of package validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub package: PackageId,
    pub version: semver::Version,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Check if validation passed (no errors)
    pub fn is_ok(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| matches!(i, ValidationIssue::Error(_)))
    }

    /// Check if there are warnings
    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| matches!(i, ValidationIssue::Warning(_)))
    }

    /// Get all errors
    pub fn errors(&self) -> Vec<&str> {
        self.issues
            .iter()
            .filter_map(|i| match i {
                ValidationIssue::Error(e) => Some(e.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Get all warnings
    pub fn warnings(&self) -> Vec<&str> {
        self.issues
            .iter()
            .filter_map(|i| match i {
                ValidationIssue::Warning(w) => Some(w.as_str()),
                _ => None,
            })
            .collect()
    }
}

/// Validation issue types
#[derive(Debug, Clone)]
pub enum ValidationIssue {
    Error(String),
    Warning(String),
}

/// Validate package dependencies can be resolved
pub fn validate_dependency_resolution(
    packages: &[PackageInfo],
    catalog: &PackageCatalog,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for pkg in packages {
        for dep in &pkg.dependencies {
            if catalog.get_package(&dep.package).is_none() {
                issues.push(ValidationIssue::Error(format!(
                    "Cannot resolve dependency {} for {}",
                    dep.package, pkg.id
                )));
            }
        }
    }

    issues
}

/// Check for conflicting packages
pub fn check_conflicts(packages: &[PackageInfo]) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut seen: HashMap<PackageId, &PackageInfo> = HashMap::new();

    for pkg in packages {
        if let Some(existing) = seen.get(&pkg.id) {
            if existing.slot == pkg.slot {
                issues.push(ValidationIssue::Error(format!(
                    "Slot conflict: {} version {} and {} cannot both be installed in slot {}",
                    pkg.id, pkg.version, existing.version, pkg.slot
                )));
            }
        } else {
            seen.insert(pkg.id.clone(), pkg);
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let catalog = PackageCatalog::new();
        let _validator = PackageValidator::new(catalog);
    }

    #[test]
    fn test_validate_package_id() {
        let catalog = PackageCatalog::new();
        let validator = PackageValidator::new(catalog);

        // Valid ID
        let valid_id = PackageId::new("sys-libs", "glibc");
        assert!(validator.validate_package_id(&valid_id).is_ok());

        // Empty category
        let invalid_id = PackageId::new("", "glibc");
        assert!(validator.validate_package_id(&invalid_id).is_err());

        // Empty name
        let invalid_id = PackageId::new("sys-libs", "");
        assert!(validator.validate_package_id(&invalid_id).is_err());
    }

    #[tokio::test]
    async fn test_hash_computation() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "test content").unwrap();

        let hash = compute_sha256(file.path()).await.unwrap();
        assert_eq!(hash.len(), 64);
    }
}
