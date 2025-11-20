//! Virtual package support
//!
//! Virtual packages provide abstraction over provider implementations.
//! For example, `virtual/jdk` can be satisfied by `dev-java/openjdk` or `dev-java/oracle-jdk`.

use crate::{Error, PackageId, Result, VersionSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual package definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPackage {
    /// Virtual package ID (e.g., virtual/jdk)
    pub id: PackageId,
    /// Description of what this virtual provides
    pub description: String,
    /// List of providers for this virtual
    pub providers: Vec<Provider>,
    /// Default provider (if any)
    pub default_provider: Option<PackageId>,
}

/// Provider for a virtual package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Package that provides this virtual
    pub package: PackageId,
    /// Version constraint for the provider
    pub version: VersionSpec,
    /// Priority (higher is preferred)
    pub priority: i32,
    /// Whether this provider is the default
    pub is_default: bool,
}

/// Configuration for virtual package selection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VirtualConfig {
    /// User-selected providers (virtual -> provider)
    pub selected_providers: HashMap<PackageId, PackageId>,
}

/// Manager for virtual packages
pub struct VirtualManager {
    /// Known virtual packages
    virtuals: HashMap<PackageId, VirtualPackage>,
    /// User configuration
    config: VirtualConfig,
}

impl VirtualManager {
    /// Create a new virtual package manager
    pub fn new() -> Self {
        let mut manager = Self {
            virtuals: HashMap::new(),
            config: VirtualConfig::default(),
        };

        // Register built-in virtuals
        manager.register_builtin_virtuals();
        manager
    }

    /// Create with custom configuration
    pub fn with_config(config: VirtualConfig) -> Self {
        let mut manager = Self::new();
        manager.config = config;
        manager
    }

    /// Register built-in virtual packages
    fn register_builtin_virtuals(&mut self) {
        // virtual/jdk
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "jdk"),
            description: "Java Development Kit".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("dev-java", "openjdk"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("dev-java", "openjdk-bin"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
                Provider {
                    package: PackageId::new("dev-java", "oracle-jdk-bin"),
                    version: VersionSpec::Any,
                    priority: 25,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("dev-java", "openjdk")),
        });

        // virtual/jre
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "jre"),
            description: "Java Runtime Environment".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("dev-java", "openjdk"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("dev-java", "openjdk-jre-bin"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("dev-java", "openjdk")),
        });

        // virtual/libc
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "libc"),
            description: "C standard library".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("sys-libs", "glibc"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("sys-libs", "musl"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("sys-libs", "glibc")),
        });

        // virtual/libcrypt
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "libcrypt"),
            description: "Library for password hashing".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("sys-libs", "libxcrypt"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("sys-libs", "glibc"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("sys-libs", "libxcrypt")),
        });

        // virtual/editor
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "editor"),
            description: "Text editor".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("app-editors", "vim"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("app-editors", "neovim"),
                    version: VersionSpec::Any,
                    priority: 90,
                    is_default: false,
                },
                Provider {
                    package: PackageId::new("app-editors", "nano"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
                Provider {
                    package: PackageId::new("app-editors", "emacs"),
                    version: VersionSpec::Any,
                    priority: 80,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("app-editors", "vim")),
        });

        // virtual/pager
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "pager"),
            description: "Terminal pager".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("sys-apps", "less"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("sys-apps", "more"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("sys-apps", "less")),
        });

        // virtual/mta
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "mta"),
            description: "Mail Transfer Agent".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("mail-mta", "postfix"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("mail-mta", "sendmail"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
                Provider {
                    package: PackageId::new("mail-mta", "exim"),
                    version: VersionSpec::Any,
                    priority: 75,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("mail-mta", "postfix")),
        });

        // virtual/rust
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "rust"),
            description: "Rust programming language".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("dev-lang", "rust"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("dev-lang", "rust-bin"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("dev-lang", "rust")),
        });

        // virtual/ssh
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "ssh"),
            description: "SSH client and server".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("net-misc", "openssh"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("net-misc", "dropbear"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("net-misc", "openssh")),
        });

        // virtual/cron
        self.register(VirtualPackage {
            id: PackageId::new("virtual", "cron"),
            description: "Cron daemon".to_string(),
            providers: vec![
                Provider {
                    package: PackageId::new("sys-process", "cronie"),
                    version: VersionSpec::Any,
                    priority: 100,
                    is_default: true,
                },
                Provider {
                    package: PackageId::new("sys-process", "fcron"),
                    version: VersionSpec::Any,
                    priority: 75,
                    is_default: false,
                },
                Provider {
                    package: PackageId::new("sys-process", "dcron"),
                    version: VersionSpec::Any,
                    priority: 50,
                    is_default: false,
                },
            ],
            default_provider: Some(PackageId::new("sys-process", "cronie")),
        });
    }

    /// Register a virtual package
    pub fn register(&mut self, virtual_pkg: VirtualPackage) {
        self.virtuals.insert(virtual_pkg.id.clone(), virtual_pkg);
    }

    /// Check if a package ID is a virtual
    pub fn is_virtual(&self, id: &PackageId) -> bool {
        id.category == "virtual" || self.virtuals.contains_key(id)
    }

    /// Get a virtual package definition
    pub fn get(&self, id: &PackageId) -> Option<&VirtualPackage> {
        self.virtuals.get(id)
    }

    /// Get the selected provider for a virtual package
    pub fn get_selected_provider(&self, virtual_id: &PackageId) -> Option<&PackageId> {
        // First check user configuration
        if let Some(provider) = self.config.selected_providers.get(virtual_id) {
            return Some(provider);
        }

        // Fall back to default provider
        if let Some(virtual_pkg) = self.virtuals.get(virtual_id) {
            return virtual_pkg.default_provider.as_ref();
        }

        None
    }

    /// Select a provider for a virtual package
    pub fn select_provider(&mut self, virtual_id: PackageId, provider: PackageId) -> Result<()> {
        // Verify the virtual exists
        let virtual_pkg = self
            .virtuals
            .get(&virtual_id)
            .ok_or_else(|| Error::PackageNotFound(virtual_id.to_string()))?;

        // Verify the provider is valid for this virtual
        let is_valid = virtual_pkg.providers.iter().any(|p| p.package == provider);

        if !is_valid {
            return Err(Error::InvalidProvider {
                virtual_pkg: virtual_id.to_string(),
                provider: provider.to_string(),
            });
        }

        self.config.selected_providers.insert(virtual_id, provider);
        Ok(())
    }

    /// Resolve a virtual package to its provider
    pub fn resolve(&self, id: &PackageId, installed: &[PackageId]) -> Option<PackageId> {
        if !self.is_virtual(id) {
            return Some(id.clone());
        }

        let virtual_pkg = self.virtuals.get(id)?;

        // Check if any provider is already installed
        for provider in &virtual_pkg.providers {
            if installed.contains(&provider.package) {
                return Some(provider.package.clone());
            }
        }

        // Return selected or default provider
        self.get_selected_provider(id).cloned()
    }

    /// Get all providers for a virtual package, sorted by priority
    pub fn get_providers(&self, id: &PackageId) -> Vec<&Provider> {
        if let Some(virtual_pkg) = self.virtuals.get(id) {
            let mut providers: Vec<_> = virtual_pkg.providers.iter().collect();
            providers.sort_by(|a, b| b.priority.cmp(&a.priority));
            providers
        } else {
            Vec::new()
        }
    }

    /// Find virtuals that a package provides
    pub fn find_virtuals_for_provider(&self, provider: &PackageId) -> Vec<&PackageId> {
        self.virtuals
            .iter()
            .filter(|(_, v)| v.providers.iter().any(|p| &p.package == provider))
            .map(|(id, _)| id)
            .collect()
    }

    /// List all known virtual packages
    pub fn list_all(&self) -> Vec<&VirtualPackage> {
        self.virtuals.values().collect()
    }
}

impl Default for VirtualManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Extend the Error type for virtual package errors
impl crate::error::Error {
    /// Check if this is an invalid provider error
    pub fn is_invalid_provider(&self) -> bool {
        matches!(self, Error::InvalidProvider { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_manager() {
        let manager = VirtualManager::new();

        // Check built-in virtuals
        assert!(manager.is_virtual(&PackageId::new("virtual", "jdk")));
        assert!(manager.is_virtual(&PackageId::new("virtual", "libc")));

        // Check providers
        let jdk = manager.get(&PackageId::new("virtual", "jdk")).unwrap();
        assert!(!jdk.providers.is_empty());
        assert_eq!(
            jdk.default_provider,
            Some(PackageId::new("dev-java", "openjdk"))
        );
    }

    #[test]
    fn test_provider_selection() {
        let mut manager = VirtualManager::new();
        let virtual_id = PackageId::new("virtual", "editor");

        // Select neovim as editor
        manager
            .select_provider(virtual_id.clone(), PackageId::new("app-editors", "neovim"))
            .unwrap();

        let selected = manager.get_selected_provider(&virtual_id).unwrap();
        assert_eq!(selected, &PackageId::new("app-editors", "neovim"));
    }

    #[test]
    fn test_resolve_with_installed() {
        let manager = VirtualManager::new();
        let virtual_id = PackageId::new("virtual", "libc");

        // When musl is installed, it should be selected
        let installed = vec![PackageId::new("sys-libs", "musl")];
        let resolved = manager.resolve(&virtual_id, &installed);
        assert_eq!(resolved, Some(PackageId::new("sys-libs", "musl")));
    }
}
