//! Buckos System Configuration
//!
//! This crate provides comprehensive system configuration management
//! inspired by Gentoo's make.conf and /etc/portage structure.
//!
//! # Overview
//!
//! The configuration system is organized into several key modules:
//!
//! - [`make_conf`]: Global build settings (CFLAGS, USE, FEATURES)
//! - [`portage`]: Complete system configuration container
//! - [`use_flags`]: USE flag system
//! - [`keywords`]: Keyword acceptance (ACCEPT_KEYWORDS)
//! - [`license`]: License acceptance (ACCEPT_LICENSE)
//! - [`mask`]: Package masking/unmasking
//! - [`env`]: Environment variable configuration
//! - [`repos`]: Repository configuration (repos.conf)
//! - [`profile`]: System profile configuration
//! - [`sets`]: Package sets (@world, @system, custom)
//! - [`features`]: FEATURES configuration
//! - [`mirrors`]: Mirror configuration
//! - [`loader`]: Configuration loading utilities
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use buckos_config::{PortageConfig, ConfigLoader};
//!
//! // Load system configuration
//! let config = ConfigLoader::system().load().unwrap();
//!
//! // Check effective USE flags for a package
//! let flags = config.effective_use("app-editors", "vim");
//! println!("USE flags: {:?}", flags);
//!
//! // Check if a package is masked
//! let masked = config.is_masked("sys-apps", "systemd", Some("250"));
//! println!("Masked: {}", masked);
//! ```
//!
//! # Building Configuration Programmatically
//!
//! ```rust
//! use buckos_config::{PortageConfigBuilder, MakeConf};
//!
//! let config = PortageConfigBuilder::new()
//!     .cflags("-O2 -march=native")
//!     .use_flags(&["X", "wayland", "systemd"])
//!     .enable_feature("ccache")
//!     .accept_testing()
//!     .build();
//!
//! println!("CFLAGS: {}", config.make_conf.cflags);
//! ```
//!
//! # Configuration Structure
//!
//! The configuration mirrors Gentoo's /etc/portage structure:
//!
//! ```text
//! /etc/buckos/
//! ├── make.conf              # Global settings
//! ├── repos.conf/            # Repository configuration
//! ├── package.use/           # Per-package USE flags
//! ├── package.accept_keywords/  # Per-package keywords
//! ├── package.license/       # Per-package licenses
//! ├── package.mask/          # Package masks
//! ├── package.unmask/        # Package unmasks
//! ├── package.env/           # Per-package environment
//! ├── env/                   # Environment file definitions
//! ├── sets/                  # Custom package sets
//! └── world                  # User-selected packages
//! ```

// Core modules
pub mod atom;
pub mod error;

// Configuration modules
pub mod env;
pub mod features;
pub mod keywords;
pub mod license;
pub mod loader;
pub mod make_conf;
pub mod mask;
pub mod mirrors;
pub mod package_sets_parser;
pub mod portage;
pub mod profile;
pub mod repos;
pub mod sets;
pub mod use_flags;

// Legacy modules (kept for compatibility)
pub mod build;
pub mod security;
pub mod slot;
pub mod version;

// Re-exports for convenience
pub use atom::{PackageAtom, UseDep, VersionOp};
pub use env::{BuildPhase, EnvConfig, EnvFile, PackageEnvEntry};
pub use error::{ConfigError, Result};
pub use features::{FeatureCategory, FeatureInfo, FeaturesConfig};
pub use keywords::{Arch, Keyword, KeywordConfig, KeywordStability, PackageKeywordEntry};
pub use license::{LicenseConfig, LicenseInfo, PackageLicenseEntry};
pub use loader::{load_system_config, load_user_config, paths, ConfigLoader};
pub use make_conf::MakeConf;
pub use mask::{MaskConfig, MaskEntry};
pub use mirrors::{Mirror, MirrorConfig, MirrorStrategy, ThirdpartyMirrors};
pub use package_sets_parser::{PackageSetInfo, PackageSets};
pub use portage::{PortageConfig, PortageConfigBuilder};
pub use profile::{AvailableProfiles, ProfileConfig, ProfileEntry, ProfileInfo, ProfileStatus};
pub use repos::{RepoDefaults, ReposConfig, Repository, SyncType};
pub use sets::{PackageSet, SetsConfig};
pub use use_flags::{PackageUseEntry, UseConfig, UseExpandVariable, UseFlag, UseFlagDescription};

// Re-export legacy types
pub use build::*;
pub use security::*;
pub use version::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        ConfigError, ConfigLoader, FeaturesConfig, KeywordConfig, LicenseConfig, MakeConf,
        MaskConfig, MirrorConfig, PackageAtom, PortageConfig, PortageConfigBuilder, ProfileConfig,
        ReposConfig, Repository, Result, SetsConfig, SyncType, UseConfig, UseFlag,
    };
}
