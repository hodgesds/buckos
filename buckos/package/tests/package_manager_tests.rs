//! Tests for the PackageManager core API
//!
//! These tests verify that the PackageManager API works correctly.
//! Tests use temporary directories for isolation.

use buckos_package::{
    BuildOptions, CleanOptions, Config, InstallOptions, PackageManager, RemoveOptions,
    UpdateOptions,
};
use buckos_package::config::{RepositoryConfig, SyncType};
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
        repositories: vec![RepositoryConfig {
            name: "test".to_string(),
            location: temp_path.join("repo"),
            sync_type: SyncType::Local,
            sync_uri: "".to_string(),
            priority: 0,
            auto_sync: false,
        }],
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
    std::fs::create_dir_all(&config.buck_repo).unwrap();

    (config, temp_dir)
}

mod config_tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.root, PathBuf::from("/"));
        assert!(config.parallelism > 0);
        assert!(!config.repositories.is_empty());
    }

    #[test]
    fn test_config_system_path() {
        let config = Config::default();
        let path = config.system_path("/etc/passwd");
        assert_eq!(path, PathBuf::from("/etc/passwd"));
    }

    #[test]
    fn test_config_system_path_custom_root() {
        let mut config = Config::default();
        config.root = PathBuf::from("/mnt/newroot");
        let path = config.system_path("/etc/passwd");
        assert_eq!(path, PathBuf::from("/mnt/newroot/etc/passwd"));
    }

    #[test]
    fn test_config_download_cache() {
        let config = Config::default();
        let path = config.download_cache();
        assert!(path.ends_with("distfiles"));
    }

    #[test]
    fn test_config_build_dir() {
        let config = Config::default();
        let path = config.build_dir();
        assert!(path.ends_with("build"));
    }

    #[test]
    fn test_config_packages_dir() {
        let config = Config::default();
        let path = config.packages_dir();
        assert!(path.ends_with("packages"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("root"));
        assert!(toml_str.contains("parallelism"));
    }

    #[test]
    fn test_config_save_and_load() {
        let (config, temp_dir) = create_test_config();
        let config_path = temp_dir.path().join("test_config.toml");

        // Save config
        config.save_to(&config_path).unwrap();
        assert!(config_path.exists());

        // Load config back
        let loaded = Config::load_from(&config_path).unwrap();
        assert_eq!(loaded.arch, config.arch);
        assert_eq!(loaded.parallelism, config.parallelism);
    }

    #[test]
    fn test_repository_config_default() {
        let repo = RepositoryConfig::default();
        assert_eq!(repo.name, "buckos");
        assert!(matches!(repo.sync_type, SyncType::Git));
        assert!(repo.auto_sync);
    }
}

mod package_manager_initialization {
    use super::*;

    #[tokio::test]
    async fn test_package_manager_new() {
        let (config, _temp_dir) = create_test_config();
        let result = PackageManager::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_package_manager_with_custom_parallelism() {
        let (mut config, _temp_dir) = create_test_config();
        config.parallelism = 8;
        let pm = PackageManager::new(config).await.unwrap();
        // PackageManager should initialize correctly with custom parallelism
        drop(pm);
    }
}

mod install_options {
    use super::*;

    #[test]
    fn test_install_options_default() {
        let opts = InstallOptions::default();
        assert!(!opts.force);
        assert!(!opts.no_deps);
        assert!(!opts.build);
        assert!(opts.use_flags.is_empty());
        assert!(!opts.oneshot);
        assert!(!opts.fetch_only);
        assert!(!opts.deep);
        assert!(!opts.newuse);
        assert!(!opts.empty_tree);
        assert!(!opts.no_replace);
    }

    #[test]
    fn test_install_options_with_use_flags() {
        let opts = InstallOptions {
            use_flags: vec!["ssl".to_string(), "ipv6".to_string()],
            ..Default::default()
        };
        assert_eq!(opts.use_flags.len(), 2);
        assert!(opts.use_flags.contains(&"ssl".to_string()));
    }
}

mod remove_options {
    use super::*;

    #[test]
    fn test_remove_options_default() {
        let opts = RemoveOptions::default();
        assert!(!opts.force);
        assert!(!opts.recursive);
    }
}

mod update_options {
    use super::*;

    #[test]
    fn test_update_options_default() {
        let opts = UpdateOptions::default();
        assert!(opts.sync);
        assert!(!opts.check_only);
        assert!(!opts.deep);
        assert!(!opts.newuse);
        assert!(!opts.with_bdeps);
    }
}

mod build_options {
    use super::*;

    #[test]
    fn test_build_options_default() {
        let opts = BuildOptions::default();
        assert!(opts.jobs.is_none());
        assert!(!opts.release);
        assert!(opts.buck_args.is_empty());
    }

    #[test]
    fn test_build_options_with_jobs() {
        let opts = BuildOptions {
            jobs: Some(4),
            release: true,
            buck_args: vec!["--show-output".to_string()],
        };
        assert_eq!(opts.jobs, Some(4));
        assert!(opts.release);
        assert_eq!(opts.buck_args.len(), 1);
    }
}

mod clean_options {
    use super::*;

    #[test]
    fn test_clean_options_default() {
        let opts = CleanOptions::default();
        assert!(!opts.all);
        assert!(!opts.downloads);
        assert!(!opts.builds);
    }

    #[test]
    fn test_clean_options_all() {
        let opts = CleanOptions {
            all: true,
            downloads: false,
            builds: false,
        };
        assert!(opts.all);
    }
}

mod package_manager_operations {
    use super::*;

    #[tokio::test]
    async fn test_list_installed_empty() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let installed = pm.list_installed().await.unwrap();
        assert!(installed.is_empty());
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let results = pm.search("nonexistent-package-xyz").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_info_not_found() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let info = pm.info("nonexistent-package").await.unwrap();
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_verify_empty() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let results = pm.verify().await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_get_world_set_empty() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let world = pm.get_world_set().await.unwrap();
        assert!(world.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_system_set() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let system = pm.get_system_set().await.unwrap();
        // System set should have predefined essential packages
        assert!(!system.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_selected_set() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let selected = pm.get_selected_set().await.unwrap();
        // Selected set should include system packages
        assert!(!selected.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_removal_list_empty() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let packages = vec!["nonexistent".to_string()];
        let opts = RemoveOptions::default();
        let to_remove = pm.get_removal_list(&packages, &opts).await.unwrap();
        assert!(to_remove.is_empty());
    }

    #[tokio::test]
    async fn test_clean_all() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let opts = CleanOptions {
            all: true,
            downloads: false,
            builds: false,
        };
        let result = pm.clean(opts).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clean_downloads_only() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let opts = CleanOptions {
            all: false,
            downloads: true,
            builds: false,
        };
        let result = pm.clean(opts).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clean_builds_only() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let opts = CleanOptions {
            all: false,
            downloads: false,
            builds: true,
        };
        let result = pm.clean(opts).await;
        assert!(result.is_ok());
    }
}

mod resolve_operations {
    use super::*;

    #[tokio::test]
    async fn test_resolve_packages_empty() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let packages: Vec<String> = vec![];
        let opts = InstallOptions::default();
        let resolution = pm.resolve_packages(&packages, &opts).await.unwrap();
        assert!(resolution.packages.is_empty());
    }

    #[tokio::test]
    async fn test_resolve_nonexistent_package() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let packages = vec!["nonexistent/package".to_string()];
        let opts = InstallOptions::default();
        let result = pm.resolve_packages(&packages, &opts).await;
        // Should handle gracefully (empty or error)
        match result {
            Ok(resolution) => assert!(resolution.packages.is_empty()),
            Err(_) => (), // Error is also acceptable
        }
    }
}

mod error_handling {
    use super::*;

    #[tokio::test]
    async fn test_remove_not_installed() {
        let (config, _temp_dir) = create_test_config();
        let pm = PackageManager::new(config).await.unwrap();
        let packages = vec!["nonexistent".to_string()];
        let opts = RemoveOptions::default();
        let result = pm.remove(&packages, opts).await;
        assert!(result.is_err());
    }
}

mod emerge_options {
    use buckos_package::EmergeOptions;

    #[test]
    fn test_emerge_options_default() {
        let opts = EmergeOptions::default();
        assert!(!opts.pretend);
        assert!(!opts.ask);
        assert!(!opts.fetch_only);
        assert!(!opts.oneshot);
        assert!(!opts.deep);
        assert!(!opts.newuse);
        assert!(!opts.tree);
        assert_eq!(opts.verbose, 0);
        assert!(!opts.quiet);
        assert!(opts.jobs.is_none());
    }

    #[test]
    fn test_emerge_options_verbose() {
        let opts = EmergeOptions {
            verbose: 2,
            ..Default::default()
        };
        assert_eq!(opts.verbose, 2);
    }

    #[test]
    fn test_emerge_options_with_jobs() {
        let opts = EmergeOptions {
            jobs: Some(8),
            ..Default::default()
        };
        assert_eq!(opts.jobs, Some(8));
    }
}

mod depclean_options {
    use buckos_package::DepcleanOptions;

    #[test]
    fn test_depclean_options_default() {
        let opts = DepcleanOptions::default();
        assert!(!opts.pretend);
        assert!(opts.packages.is_empty());
    }
}

mod sync_options {
    use buckos_package::SyncOptions;

    #[test]
    fn test_sync_options_default() {
        let opts = SyncOptions::default();
        assert!(opts.repos.is_empty());
        assert!(!opts.all);
    }
}
