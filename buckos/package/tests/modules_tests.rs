//! Tests for various package manager modules
//!
//! This file contains tests for executor, transaction, repository,
//! cache, and other supporting modules.

use buckos_package::*;
use std::collections::HashSet;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test configuration with temporary directories
fn create_test_config() -> (Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    let config = Config {
        root: temp_path.join("root"),
        db_path: temp_path.join("db"),
        cache_dir: temp_path.join("cache"),
        buck_repo: temp_path.join("repo"),
        buck_path: PathBuf::from("/usr/bin/buck2"),
        parallelism: 2,
        repositories: vec![],
        use_flags: Default::default(),
        world: Default::default(),
        arch: "amd64".to_string(),
        chost: "x86_64-pc-linux-gnu".to_string(),
        cflags: "-O2".to_string(),
        cxxflags: "${CFLAGS}".to_string(),
        ldflags: "-Wl,-O1".to_string(),
        makeopts: "-j2".to_string(),
        features: HashSet::new(),
        accept_keywords: HashSet::new(),
        accept_license: "@FREE".to_string(),
    };

    // Create necessary directories
    std::fs::create_dir_all(&config.root).unwrap();
    std::fs::create_dir_all(&config.db_path).unwrap();
    std::fs::create_dir_all(&config.cache_dir).unwrap();
    std::fs::create_dir_all(config.download_cache()).unwrap();
    std::fs::create_dir_all(config.build_dir()).unwrap();
    std::fs::create_dir_all(config.packages_dir()).unwrap();

    (config, temp_dir)
}

mod cache_tests {
    use super::*;
    use buckos_package::cache::PackageCache;

    #[test]
    fn test_cache_new() {
        let (config, _temp_dir) = create_test_config();
        let cache = PackageCache::new(&config.cache_dir);
        assert!(cache.is_ok());
    }

    #[test]
    fn test_cache_clean_all() {
        let (config, _temp_dir) = create_test_config();
        let cache = PackageCache::new(&config.cache_dir).unwrap();
        let result = cache.clean_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_clean_downloads() {
        let (config, _temp_dir) = create_test_config();
        let cache = PackageCache::new(&config.cache_dir).unwrap();
        let result = cache.clean_downloads();
        assert!(result.is_ok());
    }
}

mod config_path_tests {
    use super::*;

    #[test]
    fn test_system_path_absolute() {
        let (config, _temp_dir) = create_test_config();
        let path = config.system_path("/etc/passwd");
        assert!(path.is_absolute());
    }

    #[test]
    fn test_download_cache_path() {
        let (config, _temp_dir) = create_test_config();
        let path = config.download_cache();
        assert!(path.ends_with("distfiles"));
    }

    #[test]
    fn test_build_dir_path() {
        let (config, _temp_dir) = create_test_config();
        let path = config.build_dir();
        assert!(path.ends_with("build"));
    }

    #[test]
    fn test_packages_dir_path() {
        let (config, _temp_dir) = create_test_config();
        let path = config.packages_dir();
        assert!(path.ends_with("packages"));
    }
}

mod security_tests {
    use super::*;

    #[test]
    fn test_vulnerability_structure() {
        let vuln = Vulnerability {
            id: "GLSA-202301-01".to_string(),
            title: "Test vulnerability".to_string(),
            severity: "high".to_string(),
            package: PackageId::new("dev-libs", "openssl"),
            affected_versions: "<3.0.0".to_string(),
            fixed_version: Some("3.0.0".to_string()),
        };

        assert_eq!(vuln.id, "GLSA-202301-01");
        assert_eq!(vuln.severity, "high");
        assert!(vuln.fixed_version.is_some());
    }

    #[test]
    fn test_vulnerability_no_fix() {
        let vuln = Vulnerability {
            id: "CVE-2023-0001".to_string(),
            title: "Unpatched vulnerability".to_string(),
            severity: "critical".to_string(),
            package: PackageId::new("app-misc", "broken"),
            affected_versions: "*".to_string(),
            fixed_version: None,
        };

        assert!(vuln.fixed_version.is_none());
    }
}

mod build_result_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_build_result_success() {
        let result = BuildResult {
            target: "//packages/sys-libs/glibc:package".to_string(),
            success: true,
            output_path: Some(PathBuf::from("/var/cache/buckos/buck-out/glibc")),
            duration: Duration::from_secs(120),
            stdout: "Build successful".to_string(),
            stderr: String::new(),
        };

        assert!(result.success);
        assert!(result.output_path.is_some());
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_build_result_failure() {
        let result = BuildResult {
            target: "//packages/broken:package".to_string(),
            success: false,
            output_path: None,
            duration: Duration::from_secs(5),
            stdout: String::new(),
            stderr: "error: compilation failed".to_string(),
        };

        assert!(!result.success);
        assert!(result.output_path.is_none());
        assert!(!result.stderr.is_empty());
    }
}

mod verify_result_tests {
    use super::*;

    #[test]
    fn test_verify_result_ok() {
        let result = VerifyResult {
            package: "systemd".to_string(),
            missing: vec![],
            modified: vec![],
            ok: true,
        };

        assert!(result.ok);
        assert!(result.missing.is_empty());
        assert!(result.modified.is_empty());
    }

    #[test]
    fn test_verify_result_missing_files() {
        let result = VerifyResult {
            package: "test".to_string(),
            missing: vec![
                "/usr/bin/missing1".to_string(),
                "/usr/bin/missing2".to_string(),
            ],
            modified: vec![],
            ok: false,
        };

        assert!(!result.ok);
        assert_eq!(result.missing.len(), 2);
    }

    #[test]
    fn test_verify_result_modified_files() {
        let result = VerifyResult {
            package: "test".to_string(),
            missing: vec![],
            modified: vec!["/etc/test.conf".to_string()],
            ok: false,
        };

        assert!(!result.ok);
        assert_eq!(result.modified.len(), 1);
    }
}

mod repository_config_tests {
    use buckos_package::config::{RepositoryConfig, SyncType};

    #[test]
    fn test_repository_config_default() {
        let repo = RepositoryConfig::default();
        assert_eq!(repo.name, "buckos");
        assert!(matches!(repo.sync_type, SyncType::Git));
        assert!(repo.auto_sync);
        assert_eq!(repo.priority, 0);
    }

    #[test]
    fn test_sync_type_git() {
        let sync = SyncType::Git;
        assert!(matches!(sync, SyncType::Git));
    }

    #[test]
    fn test_sync_type_rsync() {
        let sync = SyncType::Rsync;
        assert!(matches!(sync, SyncType::Rsync));
    }

    #[test]
    fn test_sync_type_http() {
        let sync = SyncType::Http;
        assert!(matches!(sync, SyncType::Http));
    }

    #[test]
    fn test_sync_type_local() {
        let sync = SyncType::Local;
        assert!(matches!(sync, SyncType::Local));
    }
}

mod package_build_meta_tests {
    use super::*;

    fn create_test_package_info() -> PackageInfo {
        PackageInfo {
            id: PackageId::new("sys-libs", "glibc"),
            version: semver::Version::parse("2.38.0").unwrap(),
            slot: "0".to_string(),
            description: "GNU C Library".to_string(),
            homepage: Some("https://www.gnu.org/software/libc/".to_string()),
            license: "LGPL-2.1".to_string(),
            keywords: vec!["amd64".to_string()],
            use_flags: vec![
                UseFlag {
                    name: "debug".to_string(),
                    description: "Enable debug symbols".to_string(),
                    default: false,
                },
                UseFlag {
                    name: "static".to_string(),
                    description: "Build static libs".to_string(),
                    default: true,
                },
            ],
            dependencies: vec![Dependency::new(PackageId::new("sys-apps", "util-linux"))],
            build_dependencies: vec![],
            runtime_dependencies: vec![],
            source_url: Some("https://example.com/glibc.tar.xz".to_string()),
            source_hash: Some("abc123".to_string()),
            buck_target: "//packages/sys-libs/glibc:package".to_string(),
            size: 50_000_000,
            installed_size: 200_000_000,
        }
    }

    #[test]
    fn test_package_build_meta_from_info() {
        let info = create_test_package_info();
        let meta = PackageBuildMeta::from_package_info(&info);

        assert_eq!(meta.id.name, "glibc");
        assert_eq!(meta.version.to_string(), "2.38.0");
        assert!(!meta.deps.is_empty());
        // Default features should be included
        assert!(meta.features.contains("static"));
        assert!(!meta.features.contains("debug"));
    }

    #[test]
    fn test_package_build_meta_buck_target() {
        let info = create_test_package_info();
        let meta = PackageBuildMeta::from_package_info(&info);

        assert_eq!(
            meta.buck_target.to_string(),
            "//packages/sys-libs/glibc:package"
        );
    }
}

mod installed_package_tests {
    use super::*;

    #[test]
    fn test_installed_package_creation() {
        let pkg = InstalledPackage {
            id: PackageId::new("sys-apps", "systemd"),
            name: "systemd".to_string(),
            version: semver::Version::parse("255.0.0").unwrap(),
            slot: "0".to_string(),
            installed_at: chrono::Utc::now(),
            use_flags: HashSet::new(),
            files: vec![],
            size: 100_000_000,
            build_time: false,
            explicit: true,
        };

        assert_eq!(pkg.name, "systemd");
        assert!(pkg.explicit);
    }

    #[test]
    fn test_installed_package_with_files() {
        let mut pkg = InstalledPackage {
            id: PackageId::new("app-shells", "bash"),
            name: "bash".to_string(),
            version: semver::Version::parse("5.2.0").unwrap(),
            slot: "0".to_string(),
            installed_at: chrono::Utc::now(),
            use_flags: HashSet::new(),
            files: vec![],
            size: 5_000_000,
            build_time: false,
            explicit: true,
        };

        pkg.files.push(InstalledFile {
            path: "/usr/bin/bash".to_string(),
            file_type: FileType::Regular,
            mode: 0o755,
            size: 1_000_000,
            blake3_hash: Some("abc".to_string()),
            mtime: 0,
        });

        assert_eq!(pkg.files.len(), 1);
        assert_eq!(pkg.files[0].path, "/usr/bin/bash");
    }
}

mod resolution_tests {
    use super::*;

    #[test]
    fn test_resolution_empty() {
        let resolution = Resolution {
            packages: vec![],
            build_order: vec![],
            download_size: 0,
            install_size: 0,
        };

        assert!(resolution.packages.is_empty());
        assert_eq!(resolution.download_size, 0);
    }

    #[test]
    fn test_resolved_package() {
        let pkg = ResolvedPackage {
            id: PackageId::new("sys-apps", "systemd"),
            version: semver::Version::parse("255.0.0").unwrap(),
            slot: "0".to_string(),
            description: "System manager".to_string(),
            use_flags: vec![
                UseFlagStatus {
                    name: "acl".to_string(),
                    enabled: true,
                },
                UseFlagStatus {
                    name: "audit".to_string(),
                    enabled: false,
                },
            ],
            dependencies: vec![],
            size: 10_000_000,
            installed_size: 50_000_000,
            is_upgrade: true,
            is_rebuild: false,
            is_new: false,
            old_version: Some(semver::Version::parse("254.0.0").unwrap()),
        };

        assert!(pkg.is_upgrade);
        assert!(!pkg.is_new);
        assert!(pkg.old_version.is_some());
        assert_eq!(pkg.use_flags.len(), 2);
    }
}

mod newuse_tests {
    use super::*;

    #[test]
    fn test_use_flag_change_added() {
        let change = UseFlagChange {
            flag: "ssl".to_string(),
            added: true,
        };

        assert!(change.added);
        assert_eq!(change.flag, "ssl");
    }

    #[test]
    fn test_use_flag_change_removed() {
        let change = UseFlagChange {
            flag: "debug".to_string(),
            added: false,
        };

        assert!(!change.added);
    }

    #[test]
    fn test_newuse_package() {
        let pkg = NewusePackage {
            id: PackageId::new("dev-libs", "openssl"),
            name: "openssl".to_string(),
            version: semver::Version::parse("3.0.0").unwrap(),
            use_changes: vec![
                UseFlagChange {
                    flag: "tls-heartbeat".to_string(),
                    added: false,
                },
                UseFlagChange {
                    flag: "sslv3".to_string(),
                    added: false,
                },
            ],
        };

        assert_eq!(pkg.use_changes.len(), 2);
    }
}

mod features_tests {
    use super::*;

    #[test]
    fn test_default_features() {
        let config = Config::default();
        assert!(config.features.contains("parallel-fetch"));
        assert!(config.features.contains("parallel-install"));
    }

    #[test]
    fn test_custom_features() {
        let mut features = HashSet::new();
        features.insert("sandbox".to_string());
        features.insert("test".to_string());

        let (mut config, _temp_dir) = create_test_config();
        config.features = features;

        assert!(config.features.contains("sandbox"));
        assert!(config.features.contains("test"));
        assert!(!config.features.contains("parallel-fetch"));
    }
}

mod architecture_tests {
    use super::*;

    #[test]
    fn test_arch_amd64() {
        let (mut config, _temp_dir) = create_test_config();
        config.arch = "amd64".to_string();
        assert_eq!(config.arch, "amd64");
    }

    #[test]
    fn test_arch_arm64() {
        let (mut config, _temp_dir) = create_test_config();
        config.arch = "arm64".to_string();
        assert_eq!(config.arch, "arm64");
    }

    #[test]
    fn test_chost() {
        let (mut config, _temp_dir) = create_test_config();
        config.chost = "aarch64-unknown-linux-gnu".to_string();
        assert!(config.chost.contains("aarch64"));
    }
}

mod compiler_flags_tests {
    use super::*;

    #[test]
    fn test_cflags() {
        let (mut config, _temp_dir) = create_test_config();
        config.cflags = "-O2 -pipe -march=native".to_string();
        assert!(config.cflags.contains("-O2"));
        assert!(config.cflags.contains("-march=native"));
    }

    #[test]
    fn test_ldflags() {
        let (mut config, _temp_dir) = create_test_config();
        config.ldflags = "-Wl,-O1 -Wl,--as-needed -Wl,--hash-style=gnu".to_string();
        assert!(config.ldflags.contains("--as-needed"));
    }

    #[test]
    fn test_makeopts() {
        let (mut config, _temp_dir) = create_test_config();
        config.makeopts = "-j8 -l8".to_string();
        assert!(config.makeopts.contains("-j8"));
    }
}

mod license_tests {
    use super::*;

    #[test]
    fn test_accept_license() {
        let (mut config, _temp_dir) = create_test_config();
        config.accept_license = "@FREE @BINARY-REDISTRIBUTABLE".to_string();
        assert!(config.accept_license.contains("@FREE"));
    }

    #[test]
    fn test_accept_all_licenses() {
        let (mut config, _temp_dir) = create_test_config();
        config.accept_license = "*".to_string();
        assert_eq!(config.accept_license, "*");
    }
}

mod keywords_tests {
    use super::*;

    #[test]
    fn test_accept_keywords_stable() {
        let (mut config, _temp_dir) = create_test_config();
        config.accept_keywords.insert("amd64".to_string());
        assert!(config.accept_keywords.contains("amd64"));
    }

    #[test]
    fn test_accept_keywords_testing() {
        let (mut config, _temp_dir) = create_test_config();
        config.accept_keywords.insert("~amd64".to_string());
        assert!(config.accept_keywords.contains("~amd64"));
    }
}

mod parallelism_tests {
    use super::*;

    #[test]
    fn test_parallelism_default() {
        let config = Config::default();
        assert!(config.parallelism > 0);
    }

    #[test]
    fn test_parallelism_custom() {
        let (mut config, _temp_dir) = create_test_config();
        config.parallelism = 16;
        assert_eq!(config.parallelism, 16);
    }
}
