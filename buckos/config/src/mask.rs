//! Package masking and unmasking configuration
//!
//! Implements Gentoo-style package.mask and package.unmask:
//! - Global package masks
//! - Per-profile masks
//! - User unmasks

use crate::PackageAtom;
use serde::{Deserialize, Serialize};

/// Package masking configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaskConfig {
    /// Masked packages (package.mask)
    pub masked: Vec<MaskEntry>,
    /// Unmasked packages (package.unmask)
    pub unmasked: Vec<MaskEntry>,
    /// Profile masks (from profile hierarchy)
    pub profile_masks: Vec<MaskEntry>,
}

impl MaskConfig {
    /// Create a new mask configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a package mask
    pub fn add_mask(&mut self, atom: PackageAtom, reason: Option<String>) {
        self.masked.push(MaskEntry { atom, reason, author: None, date: None });
    }

    /// Add a package unmask
    pub fn add_unmask(&mut self, atom: PackageAtom, reason: Option<String>) {
        self.unmasked.push(MaskEntry { atom, reason, author: None, date: None });
    }

    /// Check if a package is masked
    pub fn is_masked(&self, category: &str, name: &str, version: Option<&str>) -> bool {
        // Check if unmasked first (takes precedence)
        for entry in &self.unmasked {
            if self.atom_matches(&entry.atom, category, name, version) {
                return false;
            }
        }

        // Check explicit masks
        for entry in &self.masked {
            if self.atom_matches(&entry.atom, category, name, version) {
                return true;
            }
        }

        // Check profile masks
        for entry in &self.profile_masks {
            if self.atom_matches(&entry.atom, category, name, version) {
                return true;
            }
        }

        false
    }

    /// Get the mask reason for a package
    pub fn mask_reason(&self, category: &str, name: &str, version: Option<&str>) -> Option<&str> {
        // Check explicit masks first
        for entry in &self.masked {
            if self.atom_matches(&entry.atom, category, name, version) {
                return entry.reason.as_deref();
            }
        }

        // Check profile masks
        for entry in &self.profile_masks {
            if self.atom_matches(&entry.atom, category, name, version) {
                return entry.reason.as_deref();
            }
        }

        None
    }

    /// Get all masked atoms
    pub fn all_masked(&self) -> Vec<&PackageAtom> {
        self.masked.iter()
            .chain(self.profile_masks.iter())
            .map(|e| &e.atom)
            .collect()
    }

    /// Get all unmasked atoms
    pub fn all_unmasked(&self) -> Vec<&PackageAtom> {
        self.unmasked.iter().map(|e| &e.atom).collect()
    }

    /// Clear all user masks
    pub fn clear_masks(&mut self) {
        self.masked.clear();
    }

    /// Clear all user unmasks
    pub fn clear_unmasks(&mut self) {
        self.unmasked.clear();
    }

    /// Merge another mask configuration
    pub fn merge(&mut self, other: &MaskConfig) {
        self.masked.extend(other.masked.iter().cloned());
        self.unmasked.extend(other.unmasked.iter().cloned());
        self.profile_masks.extend(other.profile_masks.iter().cloned());
    }

    // Helper to check if an atom matches
    fn atom_matches(&self, atom: &PackageAtom, category: &str, name: &str, version: Option<&str>) -> bool {
        if !atom.matches_cpn(category, name) {
            return false;
        }

        // If no version specified in atom, match all versions
        if atom.version.is_none() {
            return true;
        }

        // If version specified, check version match
        if let (Some(atom_ver), Some(pkg_ver)) = (&atom.version, version) {
            // Simple version comparison (a full implementation would use proper version sorting)
            match atom.operator {
                crate::atom::VersionOp::Any => true,
                crate::atom::VersionOp::Equal => atom_ver == pkg_ver,
                crate::atom::VersionOp::Greater => pkg_ver > atom_ver.as_str(),
                crate::atom::VersionOp::GreaterEqual => pkg_ver >= atom_ver.as_str(),
                crate::atom::VersionOp::Less => pkg_ver < atom_ver.as_str(),
                crate::atom::VersionOp::LessEqual => pkg_ver <= atom_ver.as_str(),
                crate::atom::VersionOp::GlobEqual => pkg_ver.starts_with(atom_ver.as_str()),
                crate::atom::VersionOp::RevisionBump => {
                    // Match same version, different revision
                    let base_ver = atom_ver.split("-r").next().unwrap_or(atom_ver);
                    pkg_ver.starts_with(base_ver)
                }
            }
        } else {
            true
        }
    }
}

/// A single mask entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskEntry {
    /// The package atom being masked/unmasked
    pub atom: PackageAtom,
    /// Reason for the mask
    pub reason: Option<String>,
    /// Author of the mask
    pub author: Option<String>,
    /// Date of the mask
    pub date: Option<String>,
}

impl MaskEntry {
    /// Create a new mask entry
    pub fn new(atom: PackageAtom) -> Self {
        Self {
            atom,
            reason: None,
            author: None,
            date: None,
        }
    }

    /// Set the reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Set the author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the date
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }
}

/// Parse a package.mask/package.unmask file content
pub fn parse_mask_file(content: &str) -> Vec<MaskEntry> {
    let mut entries = Vec::new();
    let mut current_reason = Vec::new();
    let mut current_author = None;
    let mut current_date = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            if !current_reason.is_empty() {
                current_reason.clear();
                current_author = None;
                current_date = None;
            }
            continue;
        }

        // Comment lines contain reasons
        if line.starts_with('#') {
            let comment = line.trim_start_matches('#').trim();

            // Check for author line
            if comment.contains('<') && comment.contains('>') {
                // Parse author and date
                // Format: "Author Name <email> (YYYY-MM-DD)"
                if let Some(email_start) = comment.find('<') {
                    let author_part = comment[..email_start].trim();
                    let rest = &comment[email_start..];

                    if let Some(email_end) = rest.find('>') {
                        let email = &rest[1..email_end];
                        current_author = Some(format!("{} <{}>", author_part, email));

                        // Look for date
                        if let Some(date_start) = rest.find('(') {
                            if let Some(date_end) = rest.find(')') {
                                current_date = Some(rest[date_start + 1..date_end].to_string());
                            }
                        }
                    }
                }
            } else if !comment.is_empty() {
                current_reason.push(comment.to_string());
            }
            continue;
        }

        // Parse package atom
        if let Ok(atom) = line.parse::<PackageAtom>() {
            let mut entry = MaskEntry::new(atom);

            if !current_reason.is_empty() {
                entry.reason = Some(current_reason.join("\n"));
            }
            if let Some(author) = current_author.take() {
                entry.author = Some(author);
            }
            if let Some(date) = current_date.take() {
                entry.date = Some(date);
            }

            entries.push(entry);
        }
    }

    entries
}

/// Format mask entries to file content
pub fn format_mask_file(entries: &[MaskEntry]) -> String {
    let mut output = String::new();

    for entry in entries {
        // Add author and date
        if let Some(ref author) = entry.author {
            output.push_str(&format!("# {}", author));
            if let Some(ref date) = entry.date {
                output.push_str(&format!(" ({})", date));
            }
            output.push('\n');
        }

        // Add reason
        if let Some(ref reason) = entry.reason {
            for line in reason.lines() {
                output.push_str(&format!("# {}\n", line));
            }
        }

        // Add atom
        output.push_str(&format!("{}\n", entry.atom));
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_config() {
        let mut config = MaskConfig::new();

        // Mask a package
        let atom = PackageAtom::new("sys-apps", "broken-package");
        config.add_mask(atom, Some("Known security vulnerability".to_string()));

        assert!(config.is_masked("sys-apps", "broken-package", None));
        assert!(!config.is_masked("sys-apps", "good-package", None));
    }

    #[test]
    fn test_unmask_override() {
        let mut config = MaskConfig::new();

        // Add profile mask
        let atom = PackageAtom::new("dev-lang", "experimental-lang");
        config.profile_masks.push(MaskEntry::new(atom));

        // User unmask
        let atom = PackageAtom::new("dev-lang", "experimental-lang");
        config.add_unmask(atom, Some("I need this".to_string()));

        assert!(!config.is_masked("dev-lang", "experimental-lang", None));
    }

    #[test]
    fn test_parse_mask_file() {
        let content = r#"
# John Doe <john@example.com> (2024-01-15)
# This package has a critical bug
# See bug #12345
>=sys-apps/buggy-1.0

# Jane Smith <jane@example.com> (2024-02-01)
# Security vulnerability
=app-misc/vulnerable-2.3.4
"#;

        let entries = parse_mask_file(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].atom.name, "buggy");
        assert!(entries[0].reason.as_ref().unwrap().contains("critical bug"));
        assert!(entries[0].author.as_ref().unwrap().contains("John Doe"));
    }
}
