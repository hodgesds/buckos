//! License acceptance configuration
//!
//! Implements Gentoo-style ACCEPT_LICENSE handling:
//! - License groups (@FREE, @OSI-APPROVED, etc.)
//! - Per-package license acceptance
//! - License file parsing

use crate::PackageAtom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// License configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    /// Global ACCEPT_LICENSE pattern
    pub accept_license: String,
    /// Expanded set of accepted licenses
    pub accepted: HashSet<String>,
    /// Per-package license acceptance
    pub package: Vec<PackageLicenseEntry>,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        let mut accepted = HashSet::new();
        // Default to free licenses
        for license in free_licenses() {
            accepted.insert(license);
        }

        Self {
            accept_license: "@FREE".to_string(),
            accepted,
            package: Vec::new(),
        }
    }
}

impl LicenseConfig {
    /// Create a new license configuration
    pub fn new(accept_license: impl Into<String>) -> Self {
        let accept_license = accept_license.into();
        let accepted = Self::expand_license_pattern(&accept_license);

        Self {
            accept_license,
            accepted,
            package: Vec::new(),
        }
    }

    /// Accept all licenses
    pub fn accept_all() -> Self {
        Self::new("*")
    }

    /// Accept free licenses only
    pub fn accept_free() -> Self {
        Self::new("@FREE")
    }

    /// Add a license to accept
    pub fn add_license(&mut self, license: impl Into<String>) {
        self.accepted.insert(license.into());
    }

    /// Remove a license
    pub fn remove_license(&mut self, license: &str) {
        self.accepted.remove(license);
    }

    /// Add per-package license acceptance
    pub fn add_package_license(&mut self, atom: PackageAtom, licenses: Vec<String>) {
        self.package.push(PackageLicenseEntry { atom, licenses });
    }

    /// Check if a license is accepted globally
    pub fn is_accepted(&self, license: &str) -> bool {
        if self.accept_license == "*" {
            return true;
        }

        self.accepted.contains(license)
    }

    /// Check if a license is accepted for a specific package
    pub fn is_accepted_for(&self, category: &str, name: &str, license: &str) -> bool {
        // Check per-package overrides first
        for entry in &self.package {
            if entry.atom.matches_cpn(category, name) {
                if entry.licenses.contains(&"*".to_string()) {
                    return true;
                }
                if entry.licenses.contains(&license.to_string()) {
                    return true;
                }
            }
        }

        // Fall back to global acceptance
        self.is_accepted(license)
    }

    /// Expand a license pattern into a set of licenses
    pub fn expand_license_pattern(pattern: &str) -> HashSet<String> {
        let mut result = HashSet::new();

        for part in pattern.split_whitespace() {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let negate = part.starts_with('-');
            let part = if negate { &part[1..] } else { part };

            let licenses: HashSet<String> = if part == "*" {
                // All licenses (we can't enumerate all, so this is a special case)
                HashSet::new()
            } else if part.starts_with('@') {
                // License group
                match part {
                    "@FREE" => free_licenses().into_iter().collect(),
                    "@FREE-SOFTWARE" => free_software_licenses().into_iter().collect(),
                    "@FREE-DOCUMENTS" => free_document_licenses().into_iter().collect(),
                    "@OSI-APPROVED" => osi_approved_licenses().into_iter().collect(),
                    "@FSF-APPROVED" => fsf_approved_licenses().into_iter().collect(),
                    "@GPL-COMPATIBLE" => gpl_compatible_licenses().into_iter().collect(),
                    "@BINARY-REDISTRIBUTABLE" => {
                        binary_redistributable_licenses().into_iter().collect()
                    }
                    "@EULA" => eula_licenses().into_iter().collect(),
                    _ => HashSet::new(),
                }
            } else {
                // Single license
                let mut set = HashSet::new();
                set.insert(part.to_string());
                set
            };

            if negate {
                for license in licenses {
                    result.remove(&license);
                }
            } else {
                result.extend(licenses);
            }
        }

        result
    }

    /// Parse an ACCEPT_LICENSE string
    pub fn parse_license_string(s: &str) -> Vec<String> {
        s.split_whitespace()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Per-package license entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageLicenseEntry {
    /// The package atom
    pub atom: PackageAtom,
    /// Licenses to accept for this package
    pub licenses: Vec<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    /// License name/identifier
    pub name: String,
    /// License full text URL
    pub url: Option<String>,
    /// License groups this license belongs to
    pub groups: Vec<String>,
    /// Whether this is a free software license
    pub is_free: bool,
    /// Whether this is OSI approved
    pub is_osi_approved: bool,
}

/// Free software licenses (OSI-approved + FSF-approved)
pub fn free_licenses() -> Vec<String> {
    let mut licenses = free_software_licenses();
    licenses.extend(free_document_licenses());
    licenses
}

/// Free software licenses
pub fn free_software_licenses() -> Vec<String> {
    vec![
        // GPL family
        "GPL-1".to_string(),
        "GPL-1+".to_string(),
        "GPL-2".to_string(),
        "GPL-2+".to_string(),
        "GPL-3".to_string(),
        "GPL-3+".to_string(),
        "LGPL-2".to_string(),
        "LGPL-2+".to_string(),
        "LGPL-2.1".to_string(),
        "LGPL-2.1+".to_string(),
        "LGPL-3".to_string(),
        "LGPL-3+".to_string(),
        "AGPL-3".to_string(),
        "AGPL-3+".to_string(),
        // BSD family
        "BSD".to_string(),
        "BSD-2".to_string(),
        "BSD-3".to_string(),
        "BSD-4".to_string(),
        // MIT/ISC
        "MIT".to_string(),
        "ISC".to_string(),
        // Apache
        "Apache-1.0".to_string(),
        "Apache-1.1".to_string(),
        "Apache-2.0".to_string(),
        // Mozilla
        "MPL-1.0".to_string(),
        "MPL-1.1".to_string(),
        "MPL-2.0".to_string(),
        // Others
        "Artistic".to_string(),
        "Artistic-2".to_string(),
        "PSF-2".to_string(),
        "Ruby".to_string(),
        "Zlib".to_string(),
        "libpng".to_string(),
        "openssl".to_string(),
        "public-domain".to_string(),
        "Unlicense".to_string(),
        "WTFPL-2".to_string(),
        "CC0-1.0".to_string(),
        "0BSD".to_string(),
        "HPND".to_string(),
        "RSA".to_string(),
    ]
}

/// Free documentation licenses
pub fn free_document_licenses() -> Vec<String> {
    vec![
        "FDL-1.1".to_string(),
        "FDL-1.1+".to_string(),
        "FDL-1.2".to_string(),
        "FDL-1.2+".to_string(),
        "FDL-1.3".to_string(),
        "FDL-1.3+".to_string(),
        "CC-BY-1.0".to_string(),
        "CC-BY-2.0".to_string(),
        "CC-BY-2.5".to_string(),
        "CC-BY-3.0".to_string(),
        "CC-BY-4.0".to_string(),
        "CC-BY-SA-1.0".to_string(),
        "CC-BY-SA-2.0".to_string(),
        "CC-BY-SA-2.5".to_string(),
        "CC-BY-SA-3.0".to_string(),
        "CC-BY-SA-4.0".to_string(),
        "OPL".to_string(),
        "man-pages".to_string(),
    ]
}

/// OSI-approved licenses
pub fn osi_approved_licenses() -> Vec<String> {
    vec![
        "Apache-2.0".to_string(),
        "BSD-2".to_string(),
        "BSD-3".to_string(),
        "GPL-2".to_string(),
        "GPL-2+".to_string(),
        "GPL-3".to_string(),
        "GPL-3+".to_string(),
        "LGPL-2.1".to_string(),
        "LGPL-2.1+".to_string(),
        "LGPL-3".to_string(),
        "LGPL-3+".to_string(),
        "MIT".to_string(),
        "MPL-2.0".to_string(),
        "ISC".to_string(),
        "Artistic-2".to_string(),
        "CDDL".to_string(),
        "CPL-1.0".to_string(),
        "EPL-1.0".to_string(),
        "EPL-2.0".to_string(),
        "EUPL-1.1".to_string(),
        "EUPL-1.2".to_string(),
        "PostgreSQL".to_string(),
        "PSF-2".to_string(),
        "Zlib".to_string(),
        "Unlicense".to_string(),
        "0BSD".to_string(),
    ]
}

/// FSF-approved free software licenses
pub fn fsf_approved_licenses() -> Vec<String> {
    vec![
        "GPL-1".to_string(),
        "GPL-1+".to_string(),
        "GPL-2".to_string(),
        "GPL-2+".to_string(),
        "GPL-3".to_string(),
        "GPL-3+".to_string(),
        "LGPL-2".to_string(),
        "LGPL-2+".to_string(),
        "LGPL-2.1".to_string(),
        "LGPL-2.1+".to_string(),
        "LGPL-3".to_string(),
        "LGPL-3+".to_string(),
        "AGPL-3".to_string(),
        "AGPL-3+".to_string(),
        "Apache-2.0".to_string(),
        "Artistic-2".to_string(),
        "BSD-3".to_string(),
        "MIT".to_string(),
        "MPL-2.0".to_string(),
        "CC0-1.0".to_string(),
        "public-domain".to_string(),
    ]
}

/// GPL-compatible licenses
pub fn gpl_compatible_licenses() -> Vec<String> {
    vec![
        "GPL-2".to_string(),
        "GPL-2+".to_string(),
        "GPL-3".to_string(),
        "GPL-3+".to_string(),
        "LGPL-2.1".to_string(),
        "LGPL-2.1+".to_string(),
        "LGPL-3".to_string(),
        "LGPL-3+".to_string(),
        "Apache-2.0".to_string(),
        "BSD-2".to_string(),
        "BSD-3".to_string(),
        "MIT".to_string(),
        "ISC".to_string(),
        "MPL-2.0".to_string(),
        "public-domain".to_string(),
        "CC0-1.0".to_string(),
        "Zlib".to_string(),
        "Unlicense".to_string(),
        "0BSD".to_string(),
    ]
}

/// Binary redistributable licenses (free to redistribute binaries)
pub fn binary_redistributable_licenses() -> Vec<String> {
    let mut licenses = free_licenses();
    licenses.extend(vec![
        "NVIDIA".to_string(),
        "AMD-AMDGPU-PRO".to_string(),
        "intel-microcode".to_string(),
        "linux-firmware".to_string(),
        "microcode-intel".to_string(),
        "Oracle-BCLA-JavaSE".to_string(),
    ]);
    licenses
}

/// EULA/proprietary licenses
pub fn eula_licenses() -> Vec<String> {
    vec![
        "EULA".to_string(),
        "Steam".to_string(),
        "GOG-EULA".to_string(),
        "Epic-Games".to_string(),
        "Vivaldi".to_string(),
        "google-chrome".to_string(),
        "NVIDIA-CUDA".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_license() {
        let config = LicenseConfig::default();
        assert!(config.is_accepted("MIT"));
        assert!(config.is_accepted("GPL-3"));
        assert!(config.is_accepted("Apache-2.0"));
    }

    #[test]
    fn test_accept_all() {
        let config = LicenseConfig::accept_all();
        assert!(config.is_accepted("SOME-PROPRIETARY-LICENSE"));
        assert!(config.is_accepted("EULA"));
    }

    #[test]
    fn test_expand_pattern() {
        let licenses = LicenseConfig::expand_license_pattern("@FREE -GPL-3");
        assert!(licenses.contains("MIT"));
        assert!(!licenses.contains("GPL-3"));
    }

    #[test]
    fn test_per_package_license() {
        let mut config = LicenseConfig::accept_free();

        // Accept proprietary license for specific package
        let atom = PackageAtom::new("app-misc", "nvidia-drivers");
        config.add_package_license(atom, vec!["NVIDIA".to_string()]);

        assert!(config.is_accepted_for("app-misc", "nvidia-drivers", "NVIDIA"));
        assert!(!config.is_accepted_for("other", "package", "NVIDIA"));
    }
}
