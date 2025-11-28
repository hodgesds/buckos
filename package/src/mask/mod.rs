//! Package masking and keywords system
//!
//! Implements Gentoo-style package availability control:
//! - package.mask / package.unmask for explicit masking
//! - ACCEPT_KEYWORDS for stable/testing control
//! - ~arch vs arch keyword handling
//! - License-based masking

use crate::types::{PackageId, PackageInfo, PackageSpec, VersionSpec};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Architecture keyword states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeywordState {
    /// Stable keyword (e.g., "amd64")
    Stable,
    /// Testing/unstable keyword (e.g., "~amd64")
    Testing,
    /// Broken/masked keyword (e.g., "-amd64")
    Broken,
    /// Experimental keyword (e.g., "**")
    Experimental,
}

impl KeywordState {
    /// Parse a keyword string to determine its state
    pub fn from_str(s: &str) -> (String, Self) {
        let s = s.trim();
        if s.starts_with('-') {
            (s[1..].to_string(), KeywordState::Broken)
        } else if s.starts_with('~') {
            (s[1..].to_string(), KeywordState::Testing)
        } else if s == "**" {
            (s.to_string(), KeywordState::Experimental)
        } else if s == "*" {
            (s.to_string(), KeywordState::Stable)
        } else {
            (s.to_string(), KeywordState::Stable)
        }
    }
}

/// A mask entry from package.mask or package.unmask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskEntry {
    /// Package specification (may include version constraints)
    pub spec: String,
    /// Parsed package ID
    pub package_id: PackageId,
    /// Version specification
    pub version: VersionSpec,
    /// Reason for masking (from comments)
    pub reason: Option<String>,
    /// Who added this mask (from comments)
    pub author: Option<String>,
    /// Bug reference (from comments)
    pub bug: Option<String>,
}

/// A keyword override entry from package.accept_keywords
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordEntry {
    /// Package specification
    pub spec: String,
    /// Package ID
    pub package_id: PackageId,
    /// Version specification
    pub version: VersionSpec,
    /// Keywords to accept (e.g., ["~amd64", "~arm64"])
    pub keywords: Vec<String>,
}

/// A license acceptance entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseEntry {
    /// Package specification (can be "*" for all packages)
    pub spec: String,
    /// Package ID (None for global)
    pub package_id: Option<PackageId>,
    /// Licenses to accept (can include wildcards like "*", "@FREE", "-GPL-3")
    pub licenses: Vec<String>,
}

/// Result of checking package availability
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvailabilityStatus {
    /// Package is available
    Available,
    /// Package is masked
    Masked {
        reason: Option<String>,
        by_profile: bool,
    },
    /// Package keywords don't match accepted keywords
    KeywordMasked {
        package_keywords: Vec<String>,
        accepted_keywords: Vec<String>,
    },
    /// Package license not accepted
    LicenseMasked {
        license: String,
        accepted: Vec<String>,
    },
}

/// Predefined license groups (similar to Gentoo's)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseGroups {
    /// Free software licenses
    pub free: HashSet<String>,
    /// Free software licenses (FSF approved)
    pub free_software: HashSet<String>,
    /// Open source licenses (OSI approved)
    pub osi_approved: HashSet<String>,
    /// Copyleft licenses
    pub copyleft: HashSet<String>,
    /// GPL-compatible licenses
    pub gpl_compatible: HashSet<String>,
    /// Binary redistribution allowed
    pub binary_redistributable: HashSet<String>,
}

impl Default for LicenseGroups {
    fn default() -> Self {
        let mut free = HashSet::new();
        let mut free_software = HashSet::new();
        let mut osi_approved = HashSet::new();
        let mut copyleft = HashSet::new();
        let mut gpl_compatible = HashSet::new();
        let mut binary_redistributable = HashSet::new();

        // Free software licenses
        for license in &[
            "MIT",
            "BSD",
            "BSD-2",
            "BSD-3",
            "ISC",
            "Apache-2.0",
            "GPL-2",
            "GPL-3",
            "LGPL-2",
            "LGPL-2.1",
            "LGPL-3",
            "MPL-2.0",
            "Artistic-2",
            "Zlib",
            "WTFPL",
            "Unlicense",
            "CC0-1.0",
            "public-domain",
            "HPND",
            "OFL-1.1",
            "PSF-2",
            "Ruby",
            "PHP-3.01",
        ] {
            free.insert(license.to_string());
            binary_redistributable.insert(license.to_string());
        }

        // FSF-approved free software
        for license in &[
            "GPL-2",
            "GPL-3",
            "LGPL-2",
            "LGPL-2.1",
            "LGPL-3",
            "AGPL-3",
            "Apache-2.0",
            "BSD",
            "BSD-2",
            "BSD-3",
            "MIT",
            "ISC",
            "MPL-2.0",
            "Artistic-2",
            "WTFPL",
        ] {
            free_software.insert(license.to_string());
        }

        // OSI approved
        for license in &[
            "MIT",
            "Apache-2.0",
            "GPL-2",
            "GPL-3",
            "LGPL-2.1",
            "LGPL-3",
            "BSD-2",
            "BSD-3",
            "MPL-2.0",
            "ISC",
            "Artistic-2",
            "Zlib",
            "EPL-1.0",
            "EPL-2.0",
            "CDDL",
        ] {
            osi_approved.insert(license.to_string());
        }

        // Copyleft licenses
        for license in &[
            "GPL-2", "GPL-3", "LGPL-2", "LGPL-2.1", "LGPL-3", "AGPL-3", "MPL-2.0",
        ] {
            copyleft.insert(license.to_string());
        }

        // GPL-compatible
        for license in &[
            "MIT",
            "BSD",
            "BSD-2",
            "BSD-3",
            "ISC",
            "LGPL-2",
            "LGPL-2.1",
            "Apache-2.0",
            "MPL-2.0",
            "Zlib",
            "public-domain",
        ] {
            gpl_compatible.insert(license.to_string());
        }

        Self {
            free,
            free_software,
            osi_approved,
            copyleft,
            gpl_compatible,
            binary_redistributable,
        }
    }
}

/// Package mask and keyword manager
pub struct MaskManager {
    /// Root directory (e.g., "/")
    root: PathBuf,
    /// Portage configuration directory
    config_dir: PathBuf,
    /// System architecture (e.g., "amd64")
    arch: String,
    /// Masked packages
    masks: Vec<MaskEntry>,
    /// Unmasked packages (overrides masks)
    unmasks: Vec<MaskEntry>,
    /// Profile masks
    profile_masks: Vec<MaskEntry>,
    /// Keyword overrides
    keyword_overrides: Vec<KeywordEntry>,
    /// Global accepted keywords
    accept_keywords: HashSet<String>,
    /// License acceptance rules
    license_entries: Vec<LicenseEntry>,
    /// Global accepted licenses
    accept_licenses: Vec<String>,
    /// License groups
    license_groups: LicenseGroups,
}

impl MaskManager {
    /// Create a new mask manager
    pub fn new(root: &Path, arch: &str) -> Self {
        let config_dir = root.join("etc/portage");
        Self {
            root: root.to_path_buf(),
            config_dir,
            arch: arch.to_string(),
            masks: Vec::new(),
            unmasks: Vec::new(),
            profile_masks: Vec::new(),
            keyword_overrides: Vec::new(),
            accept_keywords: HashSet::new(),
            license_entries: Vec::new(),
            accept_licenses: vec!["*".to_string()], // Accept all licenses by default
            license_groups: LicenseGroups::default(),
        }
    }

    /// Get the root directory
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a path relative to the root directory
    pub fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        }
    }

    /// Load all mask and keyword configuration
    pub fn load(&mut self) -> Result<()> {
        // Load package.mask
        let mask_path = self.config_dir.join("package.mask");
        if mask_path.exists() {
            if mask_path.is_dir() {
                self.load_masks_from_dir(&mask_path)?;
            } else {
                self.masks.extend(self.parse_mask_file(&mask_path)?);
            }
        }

        // Load package.unmask
        let unmask_path = self.config_dir.join("package.unmask");
        if unmask_path.exists() {
            if unmask_path.is_dir() {
                self.load_unmasks_from_dir(&unmask_path)?;
            } else {
                self.unmasks.extend(self.parse_mask_file(&unmask_path)?);
            }
        }

        // Load package.accept_keywords
        let keywords_path = self.config_dir.join("package.accept_keywords");
        if keywords_path.exists() {
            if keywords_path.is_dir() {
                self.load_keywords_from_dir(&keywords_path)?;
            } else {
                self.keyword_overrides
                    .extend(self.parse_keywords_file(&keywords_path)?);
            }
        }

        // Also check package.keywords (older format)
        let old_keywords_path = self.config_dir.join("package.keywords");
        if old_keywords_path.exists() {
            if old_keywords_path.is_dir() {
                self.load_keywords_from_dir(&old_keywords_path)?;
            } else {
                self.keyword_overrides
                    .extend(self.parse_keywords_file(&old_keywords_path)?);
            }
        }

        // Load package.license
        let license_path = self.config_dir.join("package.license");
        if license_path.exists() {
            if license_path.is_dir() {
                self.load_licenses_from_dir(&license_path)?;
            } else {
                self.license_entries
                    .extend(self.parse_license_file(&license_path)?);
            }
        }

        // Load profile masks
        self.load_profile_masks()?;

        Ok(())
    }

    /// Load masks from a directory
    fn load_masks_from_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && !path.file_name().unwrap().to_string_lossy().starts_with('.') {
                self.masks.extend(self.parse_mask_file(&path)?);
            }
        }
        Ok(())
    }

    /// Load unmasks from a directory
    fn load_unmasks_from_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && !path.file_name().unwrap().to_string_lossy().starts_with('.') {
                self.unmasks.extend(self.parse_mask_file(&path)?);
            }
        }
        Ok(())
    }

    /// Load keywords from a directory
    fn load_keywords_from_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && !path.file_name().unwrap().to_string_lossy().starts_with('.') {
                self.keyword_overrides
                    .extend(self.parse_keywords_file(&path)?);
            }
        }
        Ok(())
    }

    /// Load licenses from a directory
    fn load_licenses_from_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && !path.file_name().unwrap().to_string_lossy().starts_with('.') {
                self.license_entries.extend(self.parse_license_file(&path)?);
            }
        }
        Ok(())
    }

    /// Parse a package.mask or package.unmask file
    fn parse_mask_file(&self, path: &Path) -> Result<Vec<MaskEntry>> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();
        let mut current_reason = String::new();
        let mut current_author = None;
        let mut current_bug = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                current_reason.clear();
                current_author = None;
                current_bug = None;
                continue;
            }

            // Parse comments for metadata
            if line.starts_with('#') {
                let comment = line[1..].trim();

                // Look for author pattern
                if comment.contains('<') && comment.contains('@') {
                    current_author = Some(comment.to_string());
                }
                // Look for bug reference
                else if comment.to_lowercase().contains("bug")
                    || comment.contains("https://bugs.")
                {
                    current_bug = Some(comment.to_string());
                } else if !comment.is_empty() {
                    if !current_reason.is_empty() {
                        current_reason.push(' ');
                    }
                    current_reason.push_str(comment);
                }
                continue;
            }

            // Parse package specification
            match PackageSpec::parse(line) {
                Ok(spec) => {
                    entries.push(MaskEntry {
                        spec: line.to_string(),
                        package_id: spec.id,
                        version: spec.version,
                        reason: if current_reason.is_empty() {
                            None
                        } else {
                            Some(current_reason.clone())
                        },
                        author: current_author.clone(),
                        bug: current_bug.clone(),
                    });
                }
                Err(_) => {
                    // Skip invalid lines but log them
                    tracing::warn!("Invalid mask entry in {:?}: {}", path, line);
                }
            }
        }

        Ok(entries)
    }

    /// Parse a package.accept_keywords file
    fn parse_keywords_file(&self, path: &Path) -> Result<Vec<KeywordEntry>> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Split into package spec and keywords
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match PackageSpec::parse(parts[0]) {
                Ok(spec) => {
                    // Keywords are the rest, or default to ~arch
                    let keywords = if parts.len() > 1 {
                        parts[1..].iter().map(|s| s.to_string()).collect()
                    } else {
                        vec![format!("~{}", self.arch)]
                    };

                    entries.push(KeywordEntry {
                        spec: parts[0].to_string(),
                        package_id: spec.id,
                        version: spec.version,
                        keywords,
                    });
                }
                Err(_) => {
                    tracing::warn!("Invalid keyword entry in {:?}: {}", path, line);
                }
            }
        }

        Ok(entries)
    }

    /// Parse a package.license file
    fn parse_license_file(&self, path: &Path) -> Result<Vec<LicenseEntry>> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Split into package spec and licenses
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let (package_id, licenses) = if parts[0] == "*" {
                // Global license acceptance
                (None, parts[1..].iter().map(|s| s.to_string()).collect())
            } else {
                match PackageSpec::parse(parts[0]) {
                    Ok(spec) => (
                        Some(spec.id),
                        parts[1..].iter().map(|s| s.to_string()).collect(),
                    ),
                    Err(_) => {
                        tracing::warn!("Invalid license entry in {:?}: {}", path, line);
                        continue;
                    }
                }
            };

            entries.push(LicenseEntry {
                spec: parts[0].to_string(),
                package_id,
                licenses,
            });
        }

        Ok(entries)
    }

    /// Load profile masks
    fn load_profile_masks(&mut self) -> Result<()> {
        // Check for profile symlink
        let profile_link = self.config_dir.join("make.profile");
        if !profile_link.exists() {
            return Ok(());
        }

        // Follow the symlink and load package.mask from profile
        if let Ok(profile_path) = std::fs::read_link(&profile_link) {
            let resolved = if profile_path.is_absolute() {
                profile_path
            } else {
                self.config_dir.join(profile_path)
            };

            // Load profile package.mask
            let profile_mask = resolved.join("package.mask");
            if profile_mask.exists() {
                let entries = self.parse_mask_file(&profile_mask)?;
                self.profile_masks.extend(entries);
            }

            // Load parent profile masks recursively
            self.load_parent_profile_masks(&resolved)?;
        }

        Ok(())
    }

    /// Load masks from parent profiles
    fn load_parent_profile_masks(&mut self, profile_dir: &Path) -> Result<()> {
        let parent_file = profile_dir.join("parent");
        if !parent_file.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&parent_file)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parent_path = profile_dir.join(line);
            if parent_path.exists() {
                // Load parent's package.mask
                let parent_mask = parent_path.join("package.mask");
                if parent_mask.exists() {
                    let entries = self.parse_mask_file(&parent_mask)?;
                    self.profile_masks.extend(entries);
                }

                // Recurse to parent's parent
                self.load_parent_profile_masks(&parent_path)?;
            }
        }

        Ok(())
    }

    /// Set accepted keywords
    pub fn set_accept_keywords(&mut self, keywords: HashSet<String>) {
        self.accept_keywords = keywords;
    }

    /// Set accepted licenses
    pub fn set_accept_licenses(&mut self, licenses: Vec<String>) {
        self.accept_licenses = licenses;
    }

    /// Check if a package is masked
    pub fn is_masked(&self, pkg: &PackageInfo) -> Option<MaskEntry> {
        // Check user package.mask (highest priority for masking)
        for mask in &self.masks {
            if self.matches_entry(pkg, &mask.package_id, &mask.version) {
                // Check if it's been unmasked
                let unmasked = self
                    .unmasks
                    .iter()
                    .any(|unmask| self.matches_entry(pkg, &unmask.package_id, &unmask.version));
                if !unmasked {
                    return Some(mask.clone());
                }
            }
        }

        // Check profile masks
        for mask in &self.profile_masks {
            if self.matches_entry(pkg, &mask.package_id, &mask.version) {
                // Check if it's been unmasked by user
                let unmasked = self
                    .unmasks
                    .iter()
                    .any(|unmask| self.matches_entry(pkg, &unmask.package_id, &unmask.version));
                if !unmasked {
                    let mut entry = mask.clone();
                    entry.reason = Some(format!(
                        "Profile mask: {}",
                        mask.reason.as_deref().unwrap_or("No reason given")
                    ));
                    return Some(entry);
                }
            }
        }

        None
    }

    /// Check if package keywords are accepted
    pub fn check_keywords(&self, pkg: &PackageInfo) -> bool {
        // Check for keyword overrides first
        for entry in &self.keyword_overrides {
            if self.matches_entry(pkg, &entry.package_id, &entry.version) {
                // Apply keyword overrides
                for keyword in &entry.keywords {
                    let (arch, state) = KeywordState::from_str(keyword);

                    // Check if this override makes the package available
                    match state {
                        KeywordState::Testing => {
                            // Accept ~arch keyword for this package
                            if pkg.keywords.contains(&format!("~{}", arch))
                                || pkg.keywords.contains(&arch)
                            {
                                return true;
                            }
                        }
                        KeywordState::Stable => {
                            // Accept stable keyword
                            if pkg.keywords.contains(&arch) {
                                return true;
                            }
                        }
                        KeywordState::Experimental => {
                            // Accept any keyword ("**")
                            return true;
                        }
                        KeywordState::Broken => {
                            // Explicitly broken, skip
                        }
                    }
                }
            }
        }

        // Check against global ACCEPT_KEYWORDS
        for pkg_keyword in &pkg.keywords {
            let (arch, state) = KeywordState::from_str(pkg_keyword);

            // Skip broken keywords
            if state == KeywordState::Broken {
                continue;
            }

            // Check if this arch matches our target arch
            if arch == self.arch || arch == "*" || arch == "**" {
                match state {
                    KeywordState::Stable => {
                        // Stable keywords are always accepted
                        return true;
                    }
                    KeywordState::Testing => {
                        // Check if we accept testing keywords
                        if self.accept_keywords.contains(&format!("~{}", self.arch))
                            || self.accept_keywords.contains("~*")
                            || self.accept_keywords.contains("**")
                        {
                            return true;
                        }
                    }
                    KeywordState::Experimental => {
                        // Check if we accept experimental
                        if self.accept_keywords.contains("**") {
                            return true;
                        }
                    }
                    KeywordState::Broken => {}
                }
            }
        }

        false
    }

    /// Check if a license is accepted
    pub fn is_license_accepted(&self, pkg: &PackageInfo) -> bool {
        let license = &pkg.license;

        // Check package-specific license acceptance
        for entry in &self.license_entries {
            let matches = match &entry.package_id {
                Some(id) => id == &pkg.id,
                None => true, // Global entry
            };

            if matches {
                if self.license_matches(license, &entry.licenses) {
                    return true;
                }
            }
        }

        // Check global ACCEPT_LICENSE
        self.license_matches(license, &self.accept_licenses)
    }

    /// Check if a license matches acceptance rules
    fn license_matches(&self, license: &str, accepted: &[String]) -> bool {
        for rule in accepted {
            // Handle negation
            if rule.starts_with('-') {
                let neg_license = &rule[1..];
                if license == neg_license {
                    return false;
                }
                continue;
            }

            // Handle wildcards
            if rule == "*" {
                return true;
            }

            // Handle license groups
            if rule.starts_with('@') {
                let group_name = &rule[1..];
                let group = match group_name.to_uppercase().as_str() {
                    "FREE" => Some(&self.license_groups.free),
                    "FREE-SOFTWARE" | "FSF-APPROVED" => Some(&self.license_groups.free_software),
                    "OSI-APPROVED" => Some(&self.license_groups.osi_approved),
                    "COPYLEFT" => Some(&self.license_groups.copyleft),
                    "GPL-COMPATIBLE" => Some(&self.license_groups.gpl_compatible),
                    "BINARY-REDISTRIBUTABLE" => Some(&self.license_groups.binary_redistributable),
                    _ => None,
                };

                if let Some(licenses) = group {
                    if licenses.contains(license) {
                        return true;
                    }
                }
                continue;
            }

            // Direct match
            if rule == license {
                return true;
            }
        }

        false
    }

    /// Check if an entry matches a package
    fn matches_entry(
        &self,
        pkg: &PackageInfo,
        entry_id: &PackageId,
        entry_version: &VersionSpec,
    ) -> bool {
        if pkg.id != *entry_id {
            return false;
        }
        entry_version.matches(&pkg.version)
    }

    /// Check full availability status of a package
    pub fn check_availability(&self, pkg: &PackageInfo) -> AvailabilityStatus {
        // Check masking first
        if let Some(mask) = self.is_masked(pkg) {
            return AvailabilityStatus::Masked {
                reason: mask.reason,
                by_profile: self
                    .profile_masks
                    .iter()
                    .any(|m| self.matches_entry(pkg, &m.package_id, &m.version)),
            };
        }

        // Check keywords
        if !self.check_keywords(pkg) {
            return AvailabilityStatus::KeywordMasked {
                package_keywords: pkg.keywords.clone(),
                accepted_keywords: self.accept_keywords.iter().cloned().collect(),
            };
        }

        // Check license
        if !self.is_license_accepted(pkg) {
            return AvailabilityStatus::LicenseMasked {
                license: pkg.license.clone(),
                accepted: self.accept_licenses.clone(),
            };
        }

        AvailabilityStatus::Available
    }

    /// Get all masked packages (for display)
    pub fn get_all_masks(&self) -> Vec<&MaskEntry> {
        self.masks.iter().chain(self.profile_masks.iter()).collect()
    }

    /// Get all unmasks
    pub fn get_all_unmasks(&self) -> &[MaskEntry] {
        &self.unmasks
    }

    /// Get all keyword overrides
    pub fn get_keyword_overrides(&self) -> &[KeywordEntry] {
        &self.keyword_overrides
    }

    /// Add a mask entry
    pub fn add_mask(&mut self, entry: MaskEntry) {
        self.masks.push(entry);
    }

    /// Add an unmask entry
    pub fn add_unmask(&mut self, entry: MaskEntry) {
        self.unmasks.push(entry);
    }

    /// Add a keyword override
    pub fn add_keyword_override(&mut self, entry: KeywordEntry) {
        self.keyword_overrides.push(entry);
    }

    /// Save mask changes to disk
    pub fn save_masks(&self) -> Result<()> {
        let mask_path = self.config_dir.join("package.mask");

        // Ensure parent directory exists
        if let Some(parent) = mask_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        content.push_str("# Package masks managed by buckos\n\n");

        for mask in &self.masks {
            if let Some(ref reason) = mask.reason {
                content.push_str(&format!("# {}\n", reason));
            }
            if let Some(ref author) = mask.author {
                content.push_str(&format!("# {}\n", author));
            }
            if let Some(ref bug) = mask.bug {
                content.push_str(&format!("# {}\n", bug));
            }
            content.push_str(&format!("{}\n", mask.spec));
            content.push('\n');
        }

        std::fs::write(&mask_path, content)?;
        Ok(())
    }

    /// Save unmask changes to disk
    pub fn save_unmasks(&self) -> Result<()> {
        let unmask_path = self.config_dir.join("package.unmask");

        if let Some(parent) = unmask_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        content.push_str("# Package unmasks managed by buckos\n\n");

        for unmask in &self.unmasks {
            if let Some(ref reason) = unmask.reason {
                content.push_str(&format!("# {}\n", reason));
            }
            content.push_str(&format!("{}\n", unmask.spec));
            content.push('\n');
        }

        std::fs::write(&unmask_path, content)?;
        Ok(())
    }

    /// Save keyword overrides to disk
    pub fn save_keywords(&self) -> Result<()> {
        let keywords_path = self.config_dir.join("package.accept_keywords");

        if let Some(parent) = keywords_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        content.push_str("# Package keyword overrides managed by buckos\n\n");

        for entry in &self.keyword_overrides {
            content.push_str(&entry.spec);
            for keyword in &entry.keywords {
                content.push(' ');
                content.push_str(keyword);
            }
            content.push('\n');
        }

        std::fs::write(&keywords_path, content)?;
        Ok(())
    }

    /// Suggest autounmask changes for a package
    pub fn suggest_autounmask(&self, pkg: &PackageInfo) -> AutounmaskSuggestion {
        let mut suggestion = AutounmaskSuggestion::default();
        let status = self.check_availability(pkg);

        match status {
            AvailabilityStatus::Masked { reason, .. } => {
                suggestion.unmask = Some(MaskEntry {
                    spec: format!("={}-{}", pkg.id, pkg.version),
                    package_id: pkg.id.clone(),
                    version: VersionSpec::Exact(pkg.version.clone()),
                    reason: Some(format!("Unmasked by autounmask (was: {:?})", reason)),
                    author: None,
                    bug: None,
                });
            }
            AvailabilityStatus::KeywordMasked {
                package_keywords, ..
            } => {
                // Find the testing keyword for our arch
                let testing_keyword = format!("~{}", self.arch);
                if package_keywords.contains(&testing_keyword) {
                    suggestion.keyword = Some(KeywordEntry {
                        spec: format!("={}-{}", pkg.id, pkg.version),
                        package_id: pkg.id.clone(),
                        version: VersionSpec::Exact(pkg.version.clone()),
                        keywords: vec![testing_keyword],
                    });
                }
            }
            AvailabilityStatus::LicenseMasked { license, .. } => {
                suggestion.license = Some(LicenseEntry {
                    spec: pkg.id.full_name(),
                    package_id: Some(pkg.id.clone()),
                    licenses: vec![license],
                });
            }
            AvailabilityStatus::Available => {}
        }

        suggestion
    }

    /// Get current architecture
    pub fn arch(&self) -> &str {
        &self.arch
    }

    /// Set architecture
    pub fn set_arch(&mut self, arch: &str) {
        self.arch = arch.to_string();
    }
}

/// Suggestion for autounmask changes
#[derive(Debug, Clone, Default)]
pub struct AutounmaskSuggestion {
    /// Entry to add to package.unmask
    pub unmask: Option<MaskEntry>,
    /// Entry to add to package.accept_keywords
    pub keyword: Option<KeywordEntry>,
    /// Entry to add to package.license
    pub license: Option<LicenseEntry>,
}

impl AutounmaskSuggestion {
    /// Check if any suggestions were made
    pub fn has_suggestions(&self) -> bool {
        self.unmask.is_some() || self.keyword.is_some() || self.license.is_some()
    }
}

/// Helper to determine the system architecture
pub fn detect_arch() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        "amd64".to_string()
    }
    #[cfg(target_arch = "x86")]
    {
        "x86".to_string()
    }
    #[cfg(target_arch = "aarch64")]
    {
        "arm64".to_string()
    }
    #[cfg(target_arch = "arm")]
    {
        "arm".to_string()
    }
    #[cfg(target_arch = "riscv64")]
    {
        "riscv64".to_string()
    }
    #[cfg(target_arch = "powerpc64")]
    {
        "ppc64".to_string()
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv64",
        target_arch = "powerpc64"
    )))]
    {
        "unknown".to_string()
    }
}

/// Format availability status for display
pub fn format_availability_status(status: &AvailabilityStatus) -> String {
    match status {
        AvailabilityStatus::Available => "Available".to_string(),
        AvailabilityStatus::Masked { reason, by_profile } => {
            let source = if *by_profile { "profile" } else { "user" };
            match reason {
                Some(r) => format!("Masked by {} ({})", source, r),
                None => format!("Masked by {}", source),
            }
        }
        AvailabilityStatus::KeywordMasked {
            package_keywords,
            accepted_keywords,
        } => {
            format!(
                "Keyword masked: package has {:?}, accepted: {:?}",
                package_keywords, accepted_keywords
            )
        }
        AvailabilityStatus::LicenseMasked { license, accepted } => {
            format!(
                "License '{}' not accepted (accepted: {:?})",
                license, accepted
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_state_parsing() {
        assert_eq!(
            KeywordState::from_str("amd64"),
            ("amd64".to_string(), KeywordState::Stable)
        );
        assert_eq!(
            KeywordState::from_str("~amd64"),
            ("amd64".to_string(), KeywordState::Testing)
        );
        assert_eq!(
            KeywordState::from_str("-amd64"),
            ("amd64".to_string(), KeywordState::Broken)
        );
        assert_eq!(
            KeywordState::from_str("**"),
            ("**".to_string(), KeywordState::Experimental)
        );
    }

    #[test]
    fn test_license_groups() {
        let groups = LicenseGroups::default();
        assert!(groups.free.contains("MIT"));
        assert!(groups.free.contains("GPL-2"));
        assert!(groups.copyleft.contains("GPL-3"));
        assert!(groups.osi_approved.contains("Apache-2.0"));
    }

    #[test]
    fn test_mask_manager_default() {
        let manager = MaskManager::new(Path::new("/"), "amd64");
        assert_eq!(manager.arch(), "amd64");
        assert!(manager.masks.is_empty());
    }

    #[test]
    fn test_license_matching() {
        let manager = MaskManager::new(Path::new("/"), "amd64");

        // Test wildcard
        assert!(manager.license_matches("MIT", &["*".to_string()]));

        // Test direct match
        assert!(manager.license_matches("MIT", &["MIT".to_string()]));
        assert!(!manager.license_matches("GPL-2", &["MIT".to_string()]));

        // Test negation
        assert!(!manager.license_matches("GPL-3", &["*".to_string(), "-GPL-3".to_string()]));

        // Test groups
        assert!(manager.license_matches("MIT", &["@FREE".to_string()]));
        assert!(manager.license_matches("GPL-2", &["@COPYLEFT".to_string()]));
    }

    #[test]
    fn test_detect_arch() {
        let arch = detect_arch();
        assert!(!arch.is_empty());
        // Should match one of the known architectures
        assert!(
            arch == "amd64"
                || arch == "x86"
                || arch == "arm64"
                || arch == "arm"
                || arch == "riscv64"
                || arch == "ppc64"
                || arch == "unknown"
        );
    }
}
