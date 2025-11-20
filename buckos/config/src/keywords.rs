//! Keyword acceptance configuration
//!
//! Implements Gentoo-style ACCEPT_KEYWORDS handling:
//! - Architecture keywords (amd64, arm64, etc.)
//! - Stability levels (stable, testing ~, masked -)
//! - Per-package keyword acceptance

use crate::{ConfigError, PackageAtom, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Keyword acceptance configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeywordConfig {
    /// Global ACCEPT_KEYWORDS (e.g., "amd64 ~amd64")
    pub accept_keywords: HashSet<String>,
    /// Per-package keyword acceptance
    pub package: Vec<PackageKeywordEntry>,
    /// Architecture
    pub arch: String,
}

impl KeywordConfig {
    /// Create a new keyword configuration for an architecture
    pub fn new(arch: impl Into<String>) -> Self {
        let arch = arch.into();
        let mut accept_keywords = HashSet::new();
        accept_keywords.insert(arch.clone());

        Self {
            accept_keywords,
            package: Vec::new(),
            arch,
        }
    }

    /// Accept testing packages (~arch)
    pub fn accept_testing(&mut self) {
        self.accept_keywords.insert(format!("~{}", self.arch));
    }

    /// Accept all unstable packages (**)
    pub fn accept_all(&mut self) {
        self.accept_keywords.insert("**".to_string());
    }

    /// Add a keyword to accept
    pub fn add_keyword(&mut self, keyword: impl Into<String>) {
        self.accept_keywords.insert(keyword.into());
    }

    /// Remove a keyword
    pub fn remove_keyword(&mut self, keyword: &str) {
        self.accept_keywords.remove(keyword);
    }

    /// Add per-package keyword acceptance
    pub fn add_package_keywords(&mut self, atom: PackageAtom, keywords: Vec<Keyword>) {
        self.package.push(PackageKeywordEntry { atom, keywords });
    }

    /// Check if a package with given keywords is acceptable
    pub fn is_acceptable(&self, category: &str, name: &str, package_keywords: &[&str]) -> bool {
        // Check for ** (accept all)
        if self.accept_keywords.contains("**") {
            return true;
        }

        // Check per-package overrides
        for entry in &self.package {
            if entry.atom.matches_cpn(category, name) {
                // Per-package rules take precedence
                for keyword in &entry.keywords {
                    if package_keywords.iter().any(|k| keyword.matches(k)) {
                        return true;
                    }
                }
            }
        }

        // Check global accept_keywords
        for pkg_kw in package_keywords {
            // Direct match
            if self.accept_keywords.contains(*pkg_kw) {
                return true;
            }

            // Check if we accept testing version of this arch
            let testing = format!("~{}", pkg_kw);
            if self.accept_keywords.contains(&testing) {
                return true;
            }

            // Check if package has testing keyword that we accept
            if pkg_kw.starts_with('~') {
                let base = &pkg_kw[1..];
                if self.accept_keywords.contains(&format!("~{}", base)) {
                    return true;
                }
            }
        }

        false
    }

    /// Get all accepted keywords for a package
    pub fn effective_keywords(&self, category: &str, name: &str) -> HashSet<String> {
        let mut keywords = self.accept_keywords.clone();

        // Add per-package keywords
        for entry in &self.package {
            if entry.atom.matches_cpn(category, name) {
                for keyword in &entry.keywords {
                    keywords.insert(keyword.value.clone());
                }
            }
        }

        keywords
    }

    /// Parse an ACCEPT_KEYWORDS string
    pub fn parse_keywords_string(s: &str) -> Vec<Keyword> {
        s.split_whitespace()
            .filter(|s| !s.is_empty())
            .map(Keyword::parse)
            .collect()
    }
}

/// A single keyword
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keyword {
    /// The keyword value (e.g., "amd64", "~amd64", "-amd64")
    pub value: String,
    /// The stability level
    pub stability: KeywordStability,
    /// The base architecture
    pub arch: String,
}

impl Keyword {
    /// Parse a keyword string
    pub fn parse(s: &str) -> Self {
        let s = s.trim();

        if s == "**" {
            return Self {
                value: s.to_string(),
                stability: KeywordStability::Any,
                arch: "*".to_string(),
            };
        }

        if s == "*" {
            return Self {
                value: s.to_string(),
                stability: KeywordStability::Stable,
                arch: "*".to_string(),
            };
        }

        if s == "~*" {
            return Self {
                value: s.to_string(),
                stability: KeywordStability::Testing,
                arch: "*".to_string(),
            };
        }

        let (stability, arch) = if s.starts_with("~") {
            (KeywordStability::Testing, &s[1..])
        } else if s.starts_with("-") {
            (KeywordStability::Masked, &s[1..])
        } else {
            (KeywordStability::Stable, s)
        };

        Self {
            value: s.to_string(),
            stability,
            arch: arch.to_string(),
        }
    }

    /// Create a stable keyword
    pub fn stable(arch: impl Into<String>) -> Self {
        let arch = arch.into();
        Self {
            value: arch.clone(),
            stability: KeywordStability::Stable,
            arch,
        }
    }

    /// Create a testing keyword
    pub fn testing(arch: impl Into<String>) -> Self {
        let arch = arch.into();
        Self {
            value: format!("~{}", arch),
            stability: KeywordStability::Testing,
            arch,
        }
    }

    /// Create a masked keyword
    pub fn masked(arch: impl Into<String>) -> Self {
        let arch = arch.into();
        Self {
            value: format!("-{}", arch),
            stability: KeywordStability::Masked,
            arch,
        }
    }

    /// Check if this keyword matches another keyword string
    pub fn matches(&self, other: &str) -> bool {
        if self.value == "**" {
            return true;
        }

        if self.arch == "*" {
            match self.stability {
                KeywordStability::Any => true,
                KeywordStability::Stable => !other.starts_with('~') && !other.starts_with('-'),
                KeywordStability::Testing => other.starts_with('~'),
                KeywordStability::Masked => other.starts_with('-'),
            }
        } else {
            self.value == other
                || (self.stability == KeywordStability::Testing && other == self.arch)
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Keyword stability levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeywordStability {
    /// Stable release
    Stable,
    /// Testing/unstable (~arch)
    Testing,
    /// Masked (-arch)
    Masked,
    /// Accept any (**)
    Any,
}

/// Per-package keyword entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageKeywordEntry {
    /// The package atom
    pub atom: PackageAtom,
    /// Keywords to accept for this package
    pub keywords: Vec<Keyword>,
}

/// Known architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arch {
    Amd64,
    X86,
    Arm64,
    Arm,
    Ppc64,
    Riscv,
    S390,
    Sparc,
    Mips,
}

impl Arch {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::Amd64 => "amd64",
            Arch::X86 => "x86",
            Arch::Arm64 => "arm64",
            Arch::Arm => "arm",
            Arch::Ppc64 => "ppc64",
            Arch::Riscv => "riscv",
            Arch::S390 => "s390",
            Arch::Sparc => "sparc",
            Arch::Mips => "mips",
        }
    }

    /// Detect current architecture
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        return Arch::Amd64;

        #[cfg(target_arch = "x86")]
        return Arch::X86;

        #[cfg(target_arch = "aarch64")]
        return Arch::Arm64;

        #[cfg(target_arch = "arm")]
        return Arch::Arm;

        #[cfg(target_arch = "powerpc64")]
        return Arch::Ppc64;

        #[cfg(target_arch = "riscv64")]
        return Arch::Riscv;

        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "powerpc64",
            target_arch = "riscv64"
        )))]
        return Arch::Amd64; // Default fallback
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Arch {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "amd64" | "x86_64" => Ok(Arch::Amd64),
            "x86" | "i386" | "i486" | "i586" | "i686" => Ok(Arch::X86),
            "arm64" | "aarch64" => Ok(Arch::Arm64),
            "arm" => Ok(Arch::Arm),
            "ppc64" | "powerpc64" => Ok(Arch::Ppc64),
            "riscv" | "riscv64" => Ok(Arch::Riscv),
            "s390" => Ok(Arch::S390),
            "sparc" => Ok(Arch::Sparc),
            "mips" => Ok(Arch::Mips),
            _ => Err(ConfigError::InvalidKeyword(format!(
                "unknown architecture: {}",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_parse() {
        let kw = Keyword::parse("amd64");
        assert_eq!(kw.stability, KeywordStability::Stable);
        assert_eq!(kw.arch, "amd64");

        let kw = Keyword::parse("~amd64");
        assert_eq!(kw.stability, KeywordStability::Testing);
        assert_eq!(kw.arch, "amd64");

        let kw = Keyword::parse("-amd64");
        assert_eq!(kw.stability, KeywordStability::Masked);
        assert_eq!(kw.arch, "amd64");
    }

    #[test]
    fn test_keyword_acceptance() {
        let mut config = KeywordConfig::new("amd64");

        // Only stable by default
        assert!(config.is_acceptable("cat", "pkg", &["amd64"]));
        assert!(!config.is_acceptable("cat", "pkg", &["~amd64"]));

        // Accept testing
        config.accept_testing();
        assert!(config.is_acceptable("cat", "pkg", &["~amd64"]));
    }

    #[test]
    fn test_per_package_keywords() {
        let mut config = KeywordConfig::new("amd64");

        // Accept testing for specific package
        let atom = PackageAtom::new("dev-lang", "rust");
        config.add_package_keywords(atom, vec![Keyword::testing("amd64")]);

        assert!(config.is_acceptable("dev-lang", "rust", &["~amd64"]));
        assert!(!config.is_acceptable("dev-lang", "python", &["~amd64"]));
    }

    #[test]
    fn test_wildcard_keywords() {
        let mut config = KeywordConfig::new("amd64");
        config.accept_all();

        assert!(config.is_acceptable("cat", "pkg", &["~amd64"]));
        assert!(config.is_acceptable("cat", "pkg", &["-amd64"]));
        assert!(config.is_acceptable("cat", "pkg", &["anything"]));
    }
}
