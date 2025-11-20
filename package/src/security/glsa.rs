//! GLSA (Gentoo Linux Security Advisory) support
//!
//! Checks installed packages against security advisories for known vulnerabilities.

use crate::{PackageId, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A security advisory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    /// Advisory ID (e.g., GLSA-202401-01)
    pub id: String,
    /// Title
    pub title: String,
    /// Synopsis
    pub synopsis: String,
    /// Affected packages
    pub affected: Vec<AffectedPackage>,
    /// Background information
    pub background: Option<String>,
    /// Description of the vulnerability
    pub description: String,
    /// Impact assessment
    pub impact: String,
    /// Workaround if available
    pub workaround: Option<String>,
    /// Resolution steps
    pub resolution: String,
    /// References (CVE IDs, URLs)
    pub references: Vec<Reference>,
    /// Severity level
    pub severity: Severity,
    /// Publication date
    pub announced: chrono::NaiveDate,
    /// Last revision date
    pub revised: chrono::NaiveDate,
}

/// An affected package in an advisory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedPackage {
    /// Package name (category/name)
    pub package: PackageId,
    /// Affected version ranges
    pub affected_versions: Vec<VersionRange>,
    /// Fixed in version
    pub fixed_version: Option<String>,
    /// Architecture (or "*" for all)
    pub arch: String,
}

/// Version range for affected packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRange {
    /// Range type
    pub range_type: RangeType,
    /// Version value
    pub version: String,
}

/// Type of version range
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RangeType {
    /// Less than
    Lt,
    /// Less than or equal
    Le,
    /// Equal
    Eq,
    /// Greater than or equal (rare, for excluding)
    Ge,
    /// Range: version <= x < y
    Range,
}

/// Reference to external resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// Reference type (CVE, URL, etc.)
    pub ref_type: String,
    /// Reference ID or URL
    pub id: String,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Normal,
    High,
    Critical,
}

/// Result of checking for vulnerabilities
#[derive(Debug, Clone)]
pub struct VulnCheckResult {
    /// Vulnerable packages found
    pub vulnerable: Vec<VulnerablePackage>,
    /// Advisories that apply
    pub advisories: Vec<SecurityAdvisory>,
    /// Total number of advisories checked
    pub checked: usize,
}

/// A vulnerable installed package
#[derive(Debug, Clone)]
pub struct VulnerablePackage {
    /// Package ID
    pub package: PackageId,
    /// Installed version
    pub version: semver::Version,
    /// Advisory that applies
    pub advisory: String,
    /// Severity
    pub severity: Severity,
    /// Fixed version
    pub fixed_version: Option<String>,
}

/// GLSA checker
pub struct GlsaChecker {
    /// Advisory database
    advisories: Vec<SecurityAdvisory>,
    /// Path to GLSA directory
    glsa_dir: PathBuf,
}

impl GlsaChecker {
    /// Create a new GLSA checker
    pub fn new(glsa_dir: PathBuf) -> Self {
        Self {
            advisories: Vec::new(),
            glsa_dir,
        }
    }

    /// Load advisories from directory
    pub fn load(&mut self) -> Result<()> {
        if !self.glsa_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.glsa_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "xml").unwrap_or(false) {
                if let Ok(advisory) = self.parse_advisory(&path) {
                    self.advisories.push(advisory);
                }
            }
        }

        // Sort by date (newest first)
        self.advisories
            .sort_by(|a, b| b.announced.cmp(&a.announced));

        Ok(())
    }

    /// Parse a GLSA XML file
    fn parse_advisory(&self, path: &PathBuf) -> Result<SecurityAdvisory> {
        // This is a simplified parser - real implementation would use XML parser
        let content = std::fs::read_to_string(path)?;

        // Extract GLSA ID from filename
        let id = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Create a basic advisory structure
        // In production, this would properly parse the XML
        Ok(SecurityAdvisory {
            id,
            title: "Unknown".to_string(),
            synopsis: String::new(),
            affected: Vec::new(),
            background: None,
            description: String::new(),
            impact: String::new(),
            workaround: None,
            resolution: String::new(),
            references: Vec::new(),
            severity: Severity::Normal,
            announced: chrono::Local::now().date_naive(),
            revised: chrono::Local::now().date_naive(),
        })
    }

    /// Add an advisory programmatically
    pub fn add_advisory(&mut self, advisory: SecurityAdvisory) {
        self.advisories.push(advisory);
    }

    /// Check installed packages for vulnerabilities
    pub fn check(&self, installed: &[(PackageId, semver::Version)]) -> VulnCheckResult {
        let mut vulnerable = Vec::new();
        let mut matching_advisories = Vec::new();

        for advisory in &self.advisories {
            for affected in &advisory.affected {
                // Check if package is installed
                if let Some((_, version)) = installed.iter().find(|(id, _)| id == &affected.package)
                {
                    // Check if version is affected
                    if self.is_version_affected(version, &affected.affected_versions) {
                        vulnerable.push(VulnerablePackage {
                            package: affected.package.clone(),
                            version: version.clone(),
                            advisory: advisory.id.clone(),
                            severity: advisory.severity,
                            fixed_version: affected.fixed_version.clone(),
                        });

                        if !matching_advisories
                            .iter()
                            .any(|a: &SecurityAdvisory| a.id == advisory.id)
                        {
                            matching_advisories.push(advisory.clone());
                        }
                    }
                }
            }
        }

        // Sort by severity (most severe first)
        vulnerable.sort_by(|a, b| b.severity.cmp(&a.severity));

        VulnCheckResult {
            vulnerable,
            advisories: matching_advisories,
            checked: self.advisories.len(),
        }
    }

    /// Check if a version is affected
    fn is_version_affected(&self, version: &semver::Version, ranges: &[VersionRange]) -> bool {
        for range in ranges {
            let range_version = match self.parse_version(&range.version) {
                Some(v) => v,
                None => continue,
            };

            let affected = match range.range_type {
                RangeType::Lt => version < &range_version,
                RangeType::Le => version <= &range_version,
                RangeType::Eq => version == &range_version,
                RangeType::Ge => version >= &range_version,
                RangeType::Range => {
                    // Would need additional version for upper bound
                    version < &range_version
                }
            };

            if affected {
                return true;
            }
        }

        false
    }

    /// Parse a version string
    fn parse_version(&self, s: &str) -> Option<semver::Version> {
        semver::Version::parse(s).ok().or_else(|| {
            // Try simple version format
            let parts: Vec<&str> = s.split('.').collect();
            match parts.len() {
                1 => format!("{}.0.0", parts[0]).parse().ok(),
                2 => format!("{}.{}.0", parts[0], parts[1]).parse().ok(),
                _ => s.parse().ok(),
            }
        })
    }

    /// Get advisory by ID
    pub fn get_advisory(&self, id: &str) -> Option<&SecurityAdvisory> {
        self.advisories.iter().find(|a| a.id == id)
    }

    /// List all advisories
    pub fn list_advisories(&self) -> &[SecurityAdvisory] {
        &self.advisories
    }

    /// Get advisories by severity
    pub fn by_severity(&self, severity: Severity) -> Vec<&SecurityAdvisory> {
        self.advisories
            .iter()
            .filter(|a| a.severity == severity)
            .collect()
    }

    /// Get advisories affecting a package
    pub fn for_package(&self, package: &PackageId) -> Vec<&SecurityAdvisory> {
        self.advisories
            .iter()
            .filter(|a| a.affected.iter().any(|af| &af.package == package))
            .collect()
    }

    /// Search advisories by CVE
    pub fn by_cve(&self, cve: &str) -> Vec<&SecurityAdvisory> {
        self.advisories
            .iter()
            .filter(|a| {
                a.references
                    .iter()
                    .any(|r| r.ref_type == "CVE" && r.id == cve)
            })
            .collect()
    }
}

impl Default for GlsaChecker {
    fn default() -> Self {
        Self::new(PathBuf::from("/var/db/repos/gentoo/metadata/glsa"))
    }
}

/// Format vulnerability check result
pub fn format_vuln_report(result: &VulnCheckResult) -> String {
    if result.vulnerable.is_empty() {
        return format!(
            "No vulnerabilities found. Checked {} advisories.",
            result.checked
        );
    }

    let mut report = String::new();
    report.push_str(&format!(
        "Found {} vulnerable package(s) from {} advisories:\n\n",
        result.vulnerable.len(),
        result.advisories.len()
    ));

    for vuln in &result.vulnerable {
        let severity_str = match vuln.severity {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Normal => "NORMAL",
            Severity::Low => "LOW",
        };

        report.push_str(&format!(
            "  {} ({}-{})\n    Severity: {}\n    Advisory: {}\n",
            vuln.package, vuln.package.name, vuln.version, severity_str, vuln.advisory
        ));

        if let Some(ref fixed) = vuln.fixed_version {
            report.push_str(&format!("    Fixed in: {}\n", fixed));
        }

        report.push('\n');
    }

    report.push_str("Run 'glsa-check -f all' to fix affected packages.\n");

    report
}

/// Build a GLSA check command equivalent
pub fn glsa_check_command(args: &[&str]) -> Result<String> {
    let mut checker = GlsaChecker::default();
    checker.load()?;

    if args.contains(&"--list") || args.contains(&"-l") {
        let mut output = String::new();
        for advisory in checker.list_advisories() {
            let date = advisory.announced.format("%Y-%m-%d");
            output.push_str(&format!("{} [{}] {}\n", advisory.id, date, advisory.title));
        }
        return Ok(output);
    }

    // Default: show affected
    Ok("Use --list to show advisories or provide installed packages to check".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Normal);
        assert!(Severity::Normal < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_checker_default() {
        let checker = GlsaChecker::default();
        assert!(checker.advisories.is_empty());
    }

    #[test]
    fn test_version_affected() {
        let checker = GlsaChecker::default();

        let ranges = vec![VersionRange {
            range_type: RangeType::Lt,
            version: "2.0.0".to_string(),
        }];

        let v1 = semver::Version::new(1, 0, 0);
        let v2 = semver::Version::new(2, 0, 0);

        assert!(checker.is_version_affected(&v1, &ranges));
        assert!(!checker.is_version_affected(&v2, &ranges));
    }
}
