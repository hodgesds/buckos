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
    ///
    /// GLSA XML format follows a specific structure with elements like:
    /// - `<glsa id="...">` - Advisory ID
    /// - `<title>` - Title
    /// - `<synopsis>` - Short description
    /// - `<affected>` - Affected packages with version ranges
    /// - `<severity>` - Severity level
    /// - `<announced>` / `<revised>` - Dates
    fn parse_advisory(&self, path: &PathBuf) -> Result<SecurityAdvisory> {
        let content = std::fs::read_to_string(path)?;

        // Extract GLSA ID from filename (fallback) or from XML
        let id = self
            .extract_xml_attr(&content, "glsa", "id")
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            });

        // Extract basic fields using simple regex-based parsing
        let title = self
            .extract_xml_element(&content, "title")
            .unwrap_or_else(|| "Unknown".to_string());
        let synopsis = self
            .extract_xml_element(&content, "synopsis")
            .unwrap_or_default();
        let description = self
            .extract_xml_element(&content, "description")
            .unwrap_or_default();
        let impact = self
            .extract_xml_element(&content, "impact")
            .unwrap_or_default();
        let resolution = self
            .extract_xml_element(&content, "resolution")
            .unwrap_or_default();
        let background = self.extract_xml_element(&content, "background");
        let workaround = self.extract_xml_element(&content, "workaround");

        // Parse severity
        let severity = self
            .extract_xml_attr(&content, "impact", "type")
            .map(|s| match s.to_lowercase().as_str() {
                "low" => Severity::Low,
                "normal" => Severity::Normal,
                "high" => Severity::High,
                "critical" => Severity::Critical,
                _ => Severity::Normal,
            })
            .unwrap_or(Severity::Normal);

        // Parse dates
        let announced = self
            .extract_xml_element(&content, "announced")
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        let revised = self
            .extract_xml_element(&content, "revised")
            .and_then(|s| {
                // Revised format might be "2024-01-01: r1"
                let date_part = s.split(':').next().unwrap_or(&s).trim();
                chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()
            })
            .unwrap_or(announced);

        // Parse affected packages
        let affected = self.parse_affected_packages(&content);

        // Parse references (CVE, etc.)
        let references = self.parse_references(&content);

        Ok(SecurityAdvisory {
            id,
            title,
            synopsis,
            affected,
            background,
            description,
            impact,
            workaround,
            resolution,
            references,
            severity,
            announced,
            revised,
        })
    }

    /// Extract an XML element's text content
    fn extract_xml_element(&self, content: &str, element: &str) -> Option<String> {
        let start_tag = format!("<{}", element);
        let end_tag = format!("</{}>", element);

        let start_pos = content.find(&start_tag)?;
        let tag_end = content[start_pos..].find('>')? + start_pos + 1;
        let end_pos = content[tag_end..].find(&end_tag)? + tag_end;

        let text = &content[tag_end..end_pos];
        // Clean up whitespace and XML entities
        let cleaned = text
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .trim()
            .to_string();

        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    /// Extract an XML attribute value
    fn extract_xml_attr(&self, content: &str, element: &str, attr: &str) -> Option<String> {
        let start_tag = format!("<{}", element);
        let start_pos = content.find(&start_tag)?;
        let tag_content = &content[start_pos..];
        let tag_end = tag_content.find('>')?;
        let tag_str = &tag_content[..tag_end];

        // Look for attr="value" or attr='value'
        let attr_pattern = format!("{}=\"", attr);
        let alt_pattern = format!("{}='", attr);

        if let Some(attr_start) = tag_str.find(&attr_pattern) {
            let value_start = attr_start + attr_pattern.len();
            let value_end = tag_str[value_start..].find('"')? + value_start;
            return Some(tag_str[value_start..value_end].to_string());
        }

        if let Some(attr_start) = tag_str.find(&alt_pattern) {
            let value_start = attr_start + alt_pattern.len();
            let value_end = tag_str[value_start..].find('\'')? + value_start;
            return Some(tag_str[value_start..value_end].to_string());
        }

        None
    }

    /// Parse affected packages from the XML content
    fn parse_affected_packages(&self, content: &str) -> Vec<AffectedPackage> {
        let mut affected = Vec::new();

        // Find all <package> elements within <affected>
        let affected_start = match content.find("<affected>") {
            Some(pos) => pos,
            None => return affected,
        };
        let affected_end = match content[affected_start..].find("</affected>") {
            Some(pos) => affected_start + pos,
            None => return affected,
        };
        let affected_content = &content[affected_start..affected_end];

        // Simple state machine to find package elements
        let mut pos = 0;
        while let Some(pkg_start) = affected_content[pos..].find("<package") {
            let pkg_start = pos + pkg_start;
            let pkg_end = match affected_content[pkg_start..].find("</package>") {
                Some(end) => pkg_start + end + "</package>".len(),
                None => break,
            };
            let pkg_content = &affected_content[pkg_start..pkg_end];

            // Extract package name
            let name = self
                .extract_xml_attr(pkg_content, "package", "name")
                .unwrap_or_default();
            let arch = self
                .extract_xml_attr(pkg_content, "package", "arch")
                .unwrap_or_else(|| "*".to_string());

            if let Some(pkg_id) = PackageId::parse(&name) {
                // Parse version ranges
                let mut ranges = Vec::new();
                let mut fixed_version = None;

                // Look for <vulnerable> and <unaffected> elements
                if let Some(vuln) = self.extract_xml_element(pkg_content, "vulnerable") {
                    if let Some(range_type) =
                        self.extract_xml_attr(pkg_content, "vulnerable", "range")
                    {
                        ranges.push(VersionRange {
                            range_type: self.parse_range_type(&range_type),
                            version: vuln,
                        });
                    }
                }

                if let Some(unaffected) = self.extract_xml_element(pkg_content, "unaffected") {
                    fixed_version = Some(unaffected);
                }

                affected.push(AffectedPackage {
                    package: pkg_id,
                    affected_versions: ranges,
                    fixed_version,
                    arch,
                });
            }

            pos = pkg_end;
        }

        affected
    }

    /// Parse range type string
    fn parse_range_type(&self, s: &str) -> RangeType {
        match s.to_lowercase().as_str() {
            "lt" | "less" => RangeType::Lt,
            "le" | "lessequal" => RangeType::Le,
            "eq" | "equal" => RangeType::Eq,
            "ge" | "greaterequal" => RangeType::Ge,
            "rlt" | "rle" | "rge" | "rgt" => RangeType::Range,
            _ => RangeType::Lt, // Default to less-than
        }
    }

    /// Parse references (CVE IDs, URLs) from the XML content
    fn parse_references(&self, content: &str) -> Vec<Reference> {
        let mut refs = Vec::new();

        // Find references section
        let refs_start = match content.find("<references>") {
            Some(pos) => pos,
            None => return refs,
        };
        let refs_end = match content[refs_start..].find("</references>") {
            Some(pos) => refs_start + pos,
            None => return refs,
        };
        let refs_content = &content[refs_start..refs_end];

        // Parse <uri> elements
        let mut pos = 0;
        while let Some(uri_start) = refs_content[pos..].find("<uri") {
            let uri_start = pos + uri_start;
            let uri_end = match refs_content[uri_start..].find("</uri>") {
                Some(end) => uri_start + end + "</uri>".len(),
                None => break,
            };
            let uri_content = &refs_content[uri_start..uri_end];

            if let Some(link) = self.extract_xml_attr(uri_content, "uri", "link") {
                let ref_type = if link.contains("cve.mitre.org") || link.contains("nvd.nist.gov") {
                    "CVE"
                } else {
                    "URL"
                };

                // Try to extract CVE ID from link
                let id = if ref_type == "CVE" {
                    link.split('/')
                        .last()
                        .map(|s| s.to_string())
                        .unwrap_or(link.clone())
                } else {
                    link.clone()
                };

                refs.push(Reference {
                    ref_type: ref_type.to_string(),
                    id,
                });
            }

            pos = uri_end;
        }

        refs
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
