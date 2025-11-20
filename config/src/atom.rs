//! Package atom parsing and matching
//!
//! Implements Gentoo-style package atoms like:
//! - `category/package`
//! - `>=category/package-1.0`
//! - `category/package:slot`
//! - `category/package[use_flag]`

use crate::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Version comparison operators
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionOp {
    /// No version constraint
    Any,
    /// Exact version match (=)
    Equal,
    /// Greater than (>)
    Greater,
    /// Greater than or equal (>=)
    GreaterEqual,
    /// Less than (<)
    Less,
    /// Less than or equal (<=)
    LessEqual,
    /// Version glob match (=*), e.g., =category/package-1.0*
    GlobEqual,
    /// Revision bump match (~)
    RevisionBump,
}

impl Default for VersionOp {
    fn default() -> Self {
        Self::Any
    }
}

/// A package atom representing a package specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageAtom {
    /// Version operator
    pub operator: VersionOp,
    /// Package category (e.g., "sys-apps")
    pub category: String,
    /// Package name (e.g., "systemd")
    pub name: String,
    /// Version string (optional)
    pub version: Option<String>,
    /// Slot specification (optional)
    pub slot: Option<String>,
    /// Sub-slot specification (optional)
    pub subslot: Option<String>,
    /// Repository restriction (optional)
    pub repository: Option<String>,
    /// USE flag requirements
    pub use_deps: Vec<UseDep>,
}

impl PackageAtom {
    /// Create a new package atom with just category and name
    pub fn new(category: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            operator: VersionOp::Any,
            category: category.into(),
            name: name.into(),
            version: None,
            slot: None,
            subslot: None,
            repository: None,
            use_deps: Vec::new(),
        }
    }

    /// Set the version operator
    pub fn with_operator(mut self, op: VersionOp) -> Self {
        self.operator = op;
        self
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the slot
    pub fn with_slot(mut self, slot: impl Into<String>) -> Self {
        self.slot = Some(slot.into());
        self
    }

    /// Set the subslot
    pub fn with_subslot(mut self, subslot: impl Into<String>) -> Self {
        self.subslot = Some(subslot.into());
        self
    }

    /// Set the repository
    pub fn with_repository(mut self, repo: impl Into<String>) -> Self {
        self.repository = Some(repo.into());
        self
    }

    /// Add a USE dependency
    pub fn with_use_dep(mut self, dep: UseDep) -> Self {
        self.use_deps.push(dep);
        self
    }

    /// Get the fully qualified package name (category/name)
    pub fn cpn(&self) -> String {
        format!("{}/{}", self.category, self.name)
    }

    /// Check if this atom matches a given category/name
    pub fn matches_cpn(&self, category: &str, name: &str) -> bool {
        self.category == category && self.name == name
    }

    /// Check if this is a wildcard match (e.g., */package or category/*)
    pub fn is_wildcard(&self) -> bool {
        self.category == "*" || self.name == "*"
    }
}

impl FromStr for PackageAtom {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ConfigError::InvalidAtom("empty atom".to_string()));
        }

        let mut remaining = s;

        // Parse operator
        let operator = if remaining.starts_with(">=") {
            remaining = &remaining[2..];
            VersionOp::GreaterEqual
        } else if remaining.starts_with("<=") {
            remaining = &remaining[2..];
            VersionOp::LessEqual
        } else if remaining.starts_with('>') {
            remaining = &remaining[1..];
            VersionOp::Greater
        } else if remaining.starts_with('<') {
            remaining = &remaining[1..];
            VersionOp::Less
        } else if remaining.starts_with('~') {
            remaining = &remaining[1..];
            VersionOp::RevisionBump
        } else if remaining.starts_with('=') {
            remaining = &remaining[1..];
            if remaining.ends_with('*') {
                VersionOp::GlobEqual
            } else {
                VersionOp::Equal
            }
        } else {
            VersionOp::Any
        };

        // Extract USE deps [flag1,flag2]
        let mut use_deps = Vec::new();
        let use_start = remaining.find('[');
        if let Some(start) = use_start {
            let end = remaining
                .find(']')
                .ok_or_else(|| ConfigError::InvalidAtom(format!("unclosed USE deps: {}", s)))?;
            let use_str = &remaining[start + 1..end];
            for dep in use_str.split(',') {
                use_deps.push(dep.trim().parse()?);
            }
            remaining = &remaining[..start];
        }

        // Extract repository ::repo
        let mut repository = None;
        if let Some(idx) = remaining.find("::") {
            repository = Some(remaining[idx + 2..].to_string());
            remaining = &remaining[..idx];
        }

        // Extract slot :slot/subslot
        let mut slot = None;
        let mut subslot = None;
        if let Some(idx) = remaining.find(':') {
            let slot_str = &remaining[idx + 1..];
            if let Some(sub_idx) = slot_str.find('/') {
                slot = Some(slot_str[..sub_idx].to_string());
                subslot = Some(slot_str[sub_idx + 1..].to_string());
            } else {
                slot = Some(slot_str.to_string());
            }
            remaining = &remaining[..idx];
        }

        // Remove trailing * for glob matches
        if remaining.ends_with('*') && operator == VersionOp::GlobEqual {
            remaining = &remaining[..remaining.len() - 1];
        }

        // Parse category/name-version
        let slash_idx = remaining
            .find('/')
            .ok_or_else(|| ConfigError::InvalidAtom(format!("missing category: {}", s)))?;

        let category = remaining[..slash_idx].to_string();
        let name_version = &remaining[slash_idx + 1..];

        // Try to extract version from name
        let (name, version) = if operator != VersionOp::Any {
            // Find the last occurrence of -<digit> which starts the version
            let mut version_start = None;
            let chars: Vec<char> = name_version.chars().collect();
            for i in (0..chars.len().saturating_sub(1)).rev() {
                if chars[i] == '-'
                    && chars
                        .get(i + 1)
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                {
                    version_start = Some(i);
                    break;
                }
            }

            if let Some(idx) = version_start {
                (
                    name_version[..idx].to_string(),
                    Some(name_version[idx + 1..].to_string()),
                )
            } else {
                (name_version.to_string(), None)
            }
        } else {
            (name_version.to_string(), None)
        };

        if category.is_empty() || name.is_empty() {
            return Err(ConfigError::InvalidAtom(format!("invalid atom: {}", s)));
        }

        Ok(PackageAtom {
            operator,
            category,
            name,
            version,
            slot,
            subslot,
            repository,
            use_deps,
        })
    }
}

impl fmt::Display for PackageAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write operator
        match self.operator {
            VersionOp::Any => {}
            VersionOp::Equal => write!(f, "=")?,
            VersionOp::Greater => write!(f, ">")?,
            VersionOp::GreaterEqual => write!(f, ">=")?,
            VersionOp::Less => write!(f, "<")?,
            VersionOp::LessEqual => write!(f, "<=")?,
            VersionOp::GlobEqual => write!(f, "=")?,
            VersionOp::RevisionBump => write!(f, "~")?,
        }

        // Write category/name
        write!(f, "{}/{}", self.category, self.name)?;

        // Write version
        if let Some(ref ver) = self.version {
            write!(f, "-{}", ver)?;
        }

        // Write glob suffix
        if self.operator == VersionOp::GlobEqual {
            write!(f, "*")?;
        }

        // Write slot
        if let Some(ref slot) = self.slot {
            write!(f, ":{}", slot)?;
            if let Some(ref subslot) = self.subslot {
                write!(f, "/{}", subslot)?;
            }
        }

        // Write repository
        if let Some(ref repo) = self.repository {
            write!(f, "::{}", repo)?;
        }

        // Write USE deps
        if !self.use_deps.is_empty() {
            write!(f, "[")?;
            for (i, dep) in self.use_deps.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", dep)?;
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}

/// USE flag dependency in an atom
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UseDep {
    /// The USE flag name
    pub flag: String,
    /// Whether the flag must be enabled (true) or disabled (false)
    pub enabled: bool,
    /// Default value if flag is not set
    pub default: Option<bool>,
    /// Whether this is a conditional dependency (use?)
    pub conditional: bool,
}

impl FromStr for UseDep {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ConfigError::InvalidUseFlag("empty USE dep".to_string()));
        }

        let mut flag = s;
        let mut enabled = true;
        let mut default = None;
        let conditional = s.ends_with('?');

        // Check for default values (flag(+) or flag(-))
        if flag.ends_with("(+)") {
            default = Some(true);
            flag = &flag[..flag.len() - 3];
        } else if flag.ends_with("(-)") {
            default = Some(false);
            flag = &flag[..flag.len() - 3];
        }

        // Remove conditional marker
        let flag = if conditional {
            &flag[..flag.len() - 1]
        } else {
            flag
        };

        // Check for negation
        let flag = if flag.starts_with('-') {
            enabled = false;
            &flag[1..]
        } else if flag.starts_with('!') {
            enabled = false;
            &flag[1..]
        } else {
            flag
        };

        Ok(UseDep {
            flag: flag.to_string(),
            enabled,
            default,
            conditional,
        })
    }
}

impl fmt::Display for UseDep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.enabled {
            write!(f, "-")?;
        }
        write!(f, "{}", self.flag)?;
        if let Some(def) = self.default {
            write!(f, "({})", if def { "+" } else { "-" })?;
        }
        if self.conditional {
            write!(f, "?")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_atom() {
        let atom: PackageAtom = "sys-apps/systemd".parse().unwrap();
        assert_eq!(atom.category, "sys-apps");
        assert_eq!(atom.name, "systemd");
        assert_eq!(atom.operator, VersionOp::Any);
    }

    #[test]
    fn test_parse_versioned_atom() {
        let atom: PackageAtom = ">=sys-apps/systemd-250".parse().unwrap();
        assert_eq!(atom.category, "sys-apps");
        assert_eq!(atom.name, "systemd");
        assert_eq!(atom.version, Some("250".to_string()));
        assert_eq!(atom.operator, VersionOp::GreaterEqual);
    }

    #[test]
    fn test_parse_slotted_atom() {
        let atom: PackageAtom = "dev-lang/python:3.11".parse().unwrap();
        assert_eq!(atom.name, "python");
        assert_eq!(atom.slot, Some("3.11".to_string()));
    }

    #[test]
    fn test_parse_use_deps() {
        let atom: PackageAtom = "sys-apps/systemd[networkd,-resolved]".parse().unwrap();
        assert_eq!(atom.use_deps.len(), 2);
        assert_eq!(atom.use_deps[0].flag, "networkd");
        assert!(atom.use_deps[0].enabled);
        assert_eq!(atom.use_deps[1].flag, "resolved");
        assert!(!atom.use_deps[1].enabled);
    }

    #[test]
    fn test_atom_display() {
        let atom = PackageAtom::new("sys-apps", "systemd")
            .with_operator(VersionOp::GreaterEqual)
            .with_version("250")
            .with_slot("0");
        assert_eq!(atom.to_string(), ">=sys-apps/systemd-250:0");
    }
}
