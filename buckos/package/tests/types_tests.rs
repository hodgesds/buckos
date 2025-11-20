//! Tests for core type definitions in the package manager

use buckos_package::*;
use std::collections::HashSet;

mod package_id {
    use super::*;

    #[test]
    fn test_package_id_new() {
        let pkg = PackageId::new("sys-apps", "systemd");
        assert_eq!(pkg.category, "sys-apps");
        assert_eq!(pkg.name, "systemd");
    }

    #[test]
    fn test_package_id_full_name() {
        let pkg = PackageId::new("dev-libs", "openssl");
        assert_eq!(pkg.full_name(), "dev-libs/openssl");
    }

    #[test]
    fn test_package_id_parse_valid() {
        let pkg = PackageId::parse("sys-libs/glibc").unwrap();
        assert_eq!(pkg.category, "sys-libs");
        assert_eq!(pkg.name, "glibc");
    }

    #[test]
    fn test_package_id_parse_invalid_no_slash() {
        assert!(PackageId::parse("systemd").is_none());
    }

    #[test]
    fn test_package_id_parse_invalid_multiple_slashes() {
        assert!(PackageId::parse("sys/apps/systemd").is_none());
    }

    #[test]
    fn test_package_id_display() {
        let pkg = PackageId::new("app-shells", "bash");
        assert_eq!(format!("{}", pkg), "app-shells/bash");
    }

    #[test]
    fn test_package_id_equality() {
        let pkg1 = PackageId::new("sys-apps", "systemd");
        let pkg2 = PackageId::new("sys-apps", "systemd");
        let pkg3 = PackageId::new("sys-apps", "coreutils");

        assert_eq!(pkg1, pkg2);
        assert_ne!(pkg1, pkg3);
    }

    #[test]
    fn test_package_id_hash() {
        let pkg1 = PackageId::new("sys-apps", "systemd");
        let pkg2 = PackageId::new("sys-apps", "systemd");

        let mut set = HashSet::new();
        set.insert(pkg1.clone());

        assert!(set.contains(&pkg2));
    }

    #[test]
    fn test_package_id_ordering() {
        let pkg1 = PackageId::new("app-shells", "bash");
        let pkg2 = PackageId::new("sys-apps", "systemd");

        assert!(pkg1 < pkg2);
    }
}

mod version_spec {
    use super::*;
    use semver::Version;

    #[test]
    fn test_version_spec_any() {
        let spec = VersionSpec::Any;
        assert!(spec.matches(&Version::parse("1.0.0").unwrap()));
        assert!(spec.matches(&Version::parse("999.0.0").unwrap()));
    }

    #[test]
    fn test_version_spec_exact() {
        let spec = VersionSpec::Exact(Version::parse("2.0.0").unwrap());
        assert!(spec.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.1").unwrap()));
        assert!(!spec.matches(&Version::parse("1.9.9").unwrap()));
    }

    #[test]
    fn test_version_spec_greater_than() {
        let spec = VersionSpec::GreaterThan(Version::parse("1.0.0").unwrap());
        assert!(spec.matches(&Version::parse("1.0.1").unwrap()));
        assert!(spec.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("1.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_version_spec_greater_than_or_equal() {
        let spec = VersionSpec::GreaterThanOrEqual(Version::parse("1.0.0").unwrap());
        assert!(spec.matches(&Version::parse("1.0.0").unwrap()));
        assert!(spec.matches(&Version::parse("1.0.1").unwrap()));
        assert!(!spec.matches(&Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_version_spec_less_than() {
        let spec = VersionSpec::LessThan(Version::parse("2.0.0").unwrap());
        assert!(spec.matches(&Version::parse("1.9.9").unwrap()));
        assert!(spec.matches(&Version::parse("1.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn test_version_spec_less_than_or_equal() {
        let spec = VersionSpec::LessThanOrEqual(Version::parse("2.0.0").unwrap());
        assert!(spec.matches(&Version::parse("2.0.0").unwrap()));
        assert!(spec.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn test_version_spec_range() {
        let spec = VersionSpec::Range {
            min: Some(Version::parse("1.0.0").unwrap()),
            max: Some(Version::parse("2.0.0").unwrap()),
        };
        assert!(spec.matches(&Version::parse("1.0.0").unwrap()));
        assert!(spec.matches(&Version::parse("1.5.0").unwrap()));
        assert!(spec.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("0.9.9").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn test_version_spec_range_open_min() {
        let spec = VersionSpec::Range {
            min: None,
            max: Some(Version::parse("2.0.0").unwrap()),
        };
        assert!(spec.matches(&Version::parse("0.0.1").unwrap()));
        assert!(spec.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn test_version_spec_range_open_max() {
        let spec = VersionSpec::Range {
            min: Some(Version::parse("1.0.0").unwrap()),
            max: None,
        };
        assert!(spec.matches(&Version::parse("1.0.0").unwrap()));
        assert!(spec.matches(&Version::parse("999.0.0").unwrap()));
        assert!(!spec.matches(&Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_version_spec_default() {
        let spec = VersionSpec::default();
        match spec {
            VersionSpec::Any => (),
            _ => panic!("Default should be Any"),
        }
    }
}

mod dependency {
    use super::*;

    #[test]
    fn test_dependency_new() {
        let pkg = PackageId::new("sys-libs", "glibc");
        let dep = Dependency::new(pkg.clone());

        assert_eq!(dep.package, pkg);
        assert!(matches!(dep.version, VersionSpec::Any));
        assert!(dep.slot.is_none());
        assert!(!dep.optional);
        assert!(dep.build_time);
        assert!(dep.run_time);
    }
}

mod use_condition {
    use super::*;

    #[test]
    fn test_use_condition_always() {
        let cond = UseCondition::Always;
        let empty: HashSet<String> = HashSet::new();
        let flags: HashSet<String> = ["ssl", "ipv6"].iter().map(|s| s.to_string()).collect();

        assert!(cond.evaluate(&empty));
        assert!(cond.evaluate(&flags));
    }

    #[test]
    fn test_use_condition_if_enabled() {
        let cond = UseCondition::IfEnabled("ssl".to_string());
        let with_ssl: HashSet<String> = ["ssl"].iter().map(|s| s.to_string()).collect();
        let without_ssl: HashSet<String> = ["ipv6"].iter().map(|s| s.to_string()).collect();

        assert!(cond.evaluate(&with_ssl));
        assert!(!cond.evaluate(&without_ssl));
    }

    #[test]
    fn test_use_condition_if_disabled() {
        let cond = UseCondition::IfDisabled("debug".to_string());
        let with_debug: HashSet<String> = ["debug"].iter().map(|s| s.to_string()).collect();
        let without_debug: HashSet<String> = ["ssl"].iter().map(|s| s.to_string()).collect();

        assert!(!cond.evaluate(&with_debug));
        assert!(cond.evaluate(&without_debug));
    }

    #[test]
    fn test_use_condition_and() {
        let cond = UseCondition::And(vec![
            UseCondition::IfEnabled("ssl".to_string()),
            UseCondition::IfEnabled("ipv6".to_string()),
        ]);

        let both: HashSet<String> = ["ssl", "ipv6"].iter().map(|s| s.to_string()).collect();
        let only_ssl: HashSet<String> = ["ssl"].iter().map(|s| s.to_string()).collect();
        let empty: HashSet<String> = HashSet::new();

        assert!(cond.evaluate(&both));
        assert!(!cond.evaluate(&only_ssl));
        assert!(!cond.evaluate(&empty));
    }

    #[test]
    fn test_use_condition_or() {
        let cond = UseCondition::Or(vec![
            UseCondition::IfEnabled("ssl".to_string()),
            UseCondition::IfEnabled("gnutls".to_string()),
        ]);

        let with_ssl: HashSet<String> = ["ssl"].iter().map(|s| s.to_string()).collect();
        let with_gnutls: HashSet<String> = ["gnutls"].iter().map(|s| s.to_string()).collect();
        let both: HashSet<String> = ["ssl", "gnutls"].iter().map(|s| s.to_string()).collect();
        let neither: HashSet<String> = ["ipv6"].iter().map(|s| s.to_string()).collect();

        assert!(cond.evaluate(&with_ssl));
        assert!(cond.evaluate(&with_gnutls));
        assert!(cond.evaluate(&both));
        assert!(!cond.evaluate(&neither));
    }

    #[test]
    fn test_use_condition_nested() {
        let cond = UseCondition::Or(vec![
            UseCondition::And(vec![
                UseCondition::IfEnabled("ssl".to_string()),
                UseCondition::IfEnabled("static".to_string()),
            ]),
            UseCondition::IfEnabled("minimal".to_string()),
        ]);

        let ssl_static: HashSet<String> = ["ssl", "static"].iter().map(|s| s.to_string()).collect();
        let minimal: HashSet<String> = ["minimal"].iter().map(|s| s.to_string()).collect();
        let only_ssl: HashSet<String> = ["ssl"].iter().map(|s| s.to_string()).collect();

        assert!(cond.evaluate(&ssl_static));
        assert!(cond.evaluate(&minimal));
        assert!(!cond.evaluate(&only_ssl));
    }
}

mod package_spec {
    use super::*;

    #[test]
    fn test_package_spec_parse_simple() {
        let spec = PackageSpec::parse("sys-apps/systemd").unwrap();
        assert_eq!(spec.id.category, "sys-apps");
        assert_eq!(spec.id.name, "systemd");
        assert!(matches!(spec.version, VersionSpec::Any));
        assert!(spec.slot.is_none());
        assert!(spec.repo.is_none());
    }

    #[test]
    fn test_package_spec_parse_with_slot() {
        let spec = PackageSpec::parse("sys-apps/systemd:0").unwrap();
        assert_eq!(spec.id.name, "systemd");
        assert_eq!(spec.slot, Some("0".to_string()));
    }

    #[test]
    fn test_package_spec_parse_with_repo() {
        let spec = PackageSpec::parse("sys-apps/systemd::gentoo").unwrap();
        assert_eq!(spec.id.name, "systemd");
        assert_eq!(spec.repo, Some("gentoo".to_string()));
    }

    #[test]
    fn test_package_spec_parse_exact_version() {
        let spec = PackageSpec::parse("=sys-apps/systemd-250.0.0").unwrap();
        assert_eq!(spec.id.name, "systemd");
        match spec.version {
            VersionSpec::Exact(v) => assert_eq!(v.to_string(), "250.0.0"),
            _ => panic!("Expected exact version"),
        }
    }

    #[test]
    fn test_package_spec_parse_gte_version() {
        let spec = PackageSpec::parse(">=sys-apps/systemd-250.0.0").unwrap();
        match spec.version {
            VersionSpec::GreaterThanOrEqual(_) => (),
            _ => panic!("Expected >= version"),
        }
    }

    #[test]
    fn test_package_spec_parse_gt_version() {
        let spec = PackageSpec::parse(">sys-apps/systemd-250.0.0").unwrap();
        match spec.version {
            VersionSpec::GreaterThan(_) => (),
            _ => panic!("Expected > version"),
        }
    }

    #[test]
    fn test_package_spec_parse_lte_version() {
        let spec = PackageSpec::parse("<=sys-apps/systemd-250.0.0").unwrap();
        match spec.version {
            VersionSpec::LessThanOrEqual(_) => (),
            _ => panic!("Expected <= version"),
        }
    }

    #[test]
    fn test_package_spec_parse_lt_version() {
        let spec = PackageSpec::parse("<sys-apps/systemd-250.0.0").unwrap();
        match spec.version {
            VersionSpec::LessThan(_) => (),
            _ => panic!("Expected < version"),
        }
    }

    #[test]
    fn test_package_spec_parse_tilde_version() {
        let spec = PackageSpec::parse("~sys-apps/systemd-250.0.0").unwrap();
        match spec.version {
            VersionSpec::Exact(_) => (),
            _ => panic!("Expected exact version for tilde"),
        }
    }

    #[test]
    fn test_package_spec_parse_invalid() {
        assert!(PackageSpec::parse("invalid").is_err());
    }

    #[test]
    fn test_package_spec_parse_complex() {
        let spec = PackageSpec::parse(">=sys-apps/systemd-250.0.0:0::gentoo").unwrap();
        assert_eq!(spec.id.category, "sys-apps");
        assert_eq!(spec.id.name, "systemd");
        assert_eq!(spec.slot, Some("0".to_string()));
        assert_eq!(spec.repo, Some("gentoo".to_string()));
    }
}

mod buck_target {
    use super::*;

    #[test]
    fn test_buck_target_new() {
        let target = BuckTarget::new("packages/sys-libs/glibc", "package");
        assert_eq!(target.cell, "");
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "package");
    }

    #[test]
    fn test_buck_target_in_cell() {
        let target = BuckTarget::in_cell("root", "packages/sys-libs/glibc", "package");
        assert_eq!(target.cell, "root");
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "package");
    }

    #[test]
    fn test_buck_target_parse_simple() {
        let target = BuckTarget::parse("//packages/sys-libs/glibc:package").unwrap();
        assert_eq!(target.cell, "");
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "package");
    }

    #[test]
    fn test_buck_target_parse_with_cell() {
        let target = BuckTarget::parse("root//packages/sys-libs/glibc:package").unwrap();
        assert_eq!(target.cell, "root");
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "package");
    }

    #[test]
    fn test_buck_target_parse_no_explicit_name() {
        let target = BuckTarget::parse("//packages/sys-libs/glibc").unwrap();
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "glibc");
    }

    #[test]
    fn test_buck_target_parse_invalid() {
        assert!(BuckTarget::parse("not-a-target").is_none());
    }

    #[test]
    fn test_buck_target_to_string() {
        let target = BuckTarget::new("packages/sys-libs/glibc", "package");
        assert_eq!(target.to_string(), "//packages/sys-libs/glibc:package");
    }

    #[test]
    fn test_buck_target_to_string_with_cell() {
        let target = BuckTarget::in_cell("root", "packages/sys-libs/glibc", "package");
        assert_eq!(target.to_string(), "root//packages/sys-libs/glibc:package");
    }

    #[test]
    fn test_buck_target_for_package() {
        let target = BuckTarget::for_package("sys-libs", "glibc");
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "package");
    }

    #[test]
    fn test_buck_target_for_package_lib() {
        let target = BuckTarget::for_package_lib("sys-libs", "glibc");
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "glibc");
    }

    #[test]
    fn test_buck_target_from_package_id() {
        let pkg = PackageId::new("sys-libs", "glibc");
        let target = BuckTarget::from(&pkg);
        assert_eq!(target.path, "packages/sys-libs/glibc");
        assert_eq!(target.name, "package");
    }

    #[test]
    fn test_buck_target_display() {
        let target = BuckTarget::new("packages/sys-libs/glibc", "package");
        assert_eq!(format!("{}", target), "//packages/sys-libs/glibc:package");
    }
}

mod world_set {
    use super::*;

    #[test]
    fn test_world_set_default() {
        let world = WorldSet::default();
        assert!(world.packages.is_empty());
    }

    #[test]
    fn test_world_set_insert() {
        let mut world = WorldSet::default();
        world.packages.insert(PackageId::new("sys-apps", "systemd"));
        world.packages.insert(PackageId::new("dev-libs", "openssl"));

        assert_eq!(world.packages.len(), 2);
        assert!(world
            .packages
            .contains(&PackageId::new("sys-apps", "systemd")));
    }
}

mod use_config {
    use super::*;

    #[test]
    fn test_use_config_default() {
        let config = UseConfig::default();
        assert!(config.global.is_empty());
        assert!(config.package.is_empty());
    }

    #[test]
    fn test_use_config_get_flags_global() {
        let mut config = UseConfig::default();
        config.global.insert("ssl".to_string());
        config.global.insert("ipv6".to_string());

        let pkg = PackageId::new("dev-libs", "openssl");
        let flags = config.get_flags(&pkg);

        assert!(flags.contains("ssl"));
        assert!(flags.contains("ipv6"));
    }

    #[test]
    fn test_use_config_get_flags_package_specific() {
        let mut config = UseConfig::default();
        config.global.insert("ssl".to_string());

        let pkg = PackageId::new("dev-libs", "openssl");
        let mut pkg_flags = HashSet::new();
        pkg_flags.insert("static".to_string());
        config.package.insert(pkg.clone(), pkg_flags);

        let flags = config.get_flags(&pkg);

        assert!(flags.contains("ssl"));
        assert!(flags.contains("static"));
    }

    #[test]
    fn test_use_config_package_flags_extend_global() {
        let mut config = UseConfig::default();
        config.global.insert("ssl".to_string());

        let pkg1 = PackageId::new("dev-libs", "openssl");
        let pkg2 = PackageId::new("net-misc", "curl");

        let mut pkg1_flags = HashSet::new();
        pkg1_flags.insert("static".to_string());
        config.package.insert(pkg1.clone(), pkg1_flags);

        // pkg1 should have both global and package-specific flags
        let flags1 = config.get_flags(&pkg1);
        assert!(flags1.contains("ssl"));
        assert!(flags1.contains("static"));

        // pkg2 should only have global flags
        let flags2 = config.get_flags(&pkg2);
        assert!(flags2.contains("ssl"));
        assert!(!flags2.contains("static"));
    }
}

mod buck_build_mode {
    use super::*;

    #[test]
    fn test_buck_build_mode_default() {
        let mode = BuckBuildMode::default();
        assert!(matches!(mode, BuckBuildMode::Release));
    }

    #[test]
    fn test_buck_build_mode_display() {
        assert_eq!(format!("{}", BuckBuildMode::Debug), "debug");
        assert_eq!(format!("{}", BuckBuildMode::Release), "release");
        assert_eq!(format!("{}", BuckBuildMode::Profile), "profile");
    }
}

mod buck_config {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_buck_config_default() {
        let config = BuckConfig::default();
        assert_eq!(config.buck_path, PathBuf::from("/usr/bin/buck2"));
        assert_eq!(config.repo_path, PathBuf::from("/var/db/repos/buckos"));
        assert_eq!(
            config.output_dir,
            PathBuf::from("/var/cache/buckos/buck-out")
        );
        assert!(config.jobs > 0);
        assert!(matches!(config.mode, BuckBuildMode::Release));
        assert!(config.extra_args.is_empty());
        assert!(config.env.is_empty());
    }
}

mod file_type {
    use super::*;

    #[test]
    fn test_file_type_equality() {
        assert_eq!(FileType::Regular, FileType::Regular);
        assert_ne!(FileType::Regular, FileType::Directory);
    }
}

mod serialization {
    use super::*;

    #[test]
    fn test_package_id_serialize() {
        let pkg = PackageId::new("sys-apps", "systemd");
        let json = serde_json::to_string(&pkg).unwrap();
        assert!(json.contains("sys-apps"));
        assert!(json.contains("systemd"));
    }

    #[test]
    fn test_package_id_deserialize() {
        let json = r#"{"category":"sys-apps","name":"systemd"}"#;
        let pkg: PackageId = serde_json::from_str(json).unwrap();
        assert_eq!(pkg.category, "sys-apps");
        assert_eq!(pkg.name, "systemd");
    }

    #[test]
    fn test_version_spec_serialize() {
        let spec = VersionSpec::Exact(semver::Version::parse("1.0.0").unwrap());
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("Exact"));
    }

    #[test]
    fn test_use_flag_serialize() {
        let flag = UseFlag {
            name: "ssl".to_string(),
            description: "Enable SSL support".to_string(),
            default: true,
        };
        let json = serde_json::to_string(&flag).unwrap();
        assert!(json.contains("ssl"));
        assert!(json.contains("Enable SSL support"));
    }

    #[test]
    fn test_buck_target_serialize() {
        let target = BuckTarget::new("packages/sys-libs/glibc", "package");
        let json = serde_json::to_string(&target).unwrap();
        assert!(json.contains("packages/sys-libs/glibc"));
    }

    #[test]
    fn test_world_set_serialize() {
        let mut world = WorldSet::default();
        world.packages.insert(PackageId::new("sys-apps", "systemd"));

        let json = serde_json::to_string(&world).unwrap();
        let deserialized: WorldSet = serde_json::from_str(&json).unwrap();

        assert_eq!(world.packages.len(), deserialized.packages.len());
    }
}
