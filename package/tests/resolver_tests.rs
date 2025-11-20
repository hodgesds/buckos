//! Tests for the dependency resolver module

use buckos_package::resolver::InternalResolution;
use buckos_package::{Dependency, PackageId, PackageInfo, UseCondition, VersionSpec};

mod version_constraint_tests {
    use super::*;

    #[test]
    fn test_version_constraint_exact() {
        let spec = VersionSpec::Exact(semver::Version::parse("1.2.3").unwrap());

        assert!(spec.matches(&semver::Version::parse("1.2.3").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("1.2.4").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_version_constraint_any() {
        let spec = VersionSpec::Any;

        assert!(spec.matches(&semver::Version::parse("0.0.1").unwrap()));
        assert!(spec.matches(&semver::Version::parse("999.0.0").unwrap()));
    }

    #[test]
    fn test_version_constraint_greater_than() {
        let spec = VersionSpec::GreaterThan(semver::Version::parse("1.0.0").unwrap());

        assert!(spec.matches(&semver::Version::parse("1.0.1").unwrap()));
        assert!(spec.matches(&semver::Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("1.0.0").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_version_constraint_greater_than_or_equal() {
        let spec = VersionSpec::GreaterThanOrEqual(semver::Version::parse("1.0.0").unwrap());

        assert!(spec.matches(&semver::Version::parse("1.0.0").unwrap()));
        assert!(spec.matches(&semver::Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_version_constraint_less_than() {
        let spec = VersionSpec::LessThan(semver::Version::parse("2.0.0").unwrap());

        assert!(spec.matches(&semver::Version::parse("1.9.9").unwrap()));
        assert!(spec.matches(&semver::Version::parse("0.0.1").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("2.0.0").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn test_version_constraint_less_than_or_equal() {
        let spec = VersionSpec::LessThanOrEqual(semver::Version::parse("2.0.0").unwrap());

        assert!(spec.matches(&semver::Version::parse("2.0.0").unwrap()));
        assert!(spec.matches(&semver::Version::parse("1.9.9").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn test_version_constraint_range() {
        let spec = VersionSpec::Range {
            min: Some(semver::Version::parse("1.0.0").unwrap()),
            max: Some(semver::Version::parse("2.0.0").unwrap()),
        };

        // Edge cases
        assert!(spec.matches(&semver::Version::parse("1.0.0").unwrap()));
        assert!(spec.matches(&semver::Version::parse("2.0.0").unwrap()));

        // Inside range
        assert!(spec.matches(&semver::Version::parse("1.5.0").unwrap()));

        // Outside range
        assert!(!spec.matches(&semver::Version::parse("0.9.9").unwrap()));
        assert!(!spec.matches(&semver::Version::parse("2.0.1").unwrap()));
    }
}

mod dependency_tests {
    use super::*;

    #[test]
    fn test_dependency_new() {
        let dep = Dependency::new(PackageId::new("dev-libs", "openssl"));

        assert_eq!(dep.package.name, "openssl");
        assert!(dep.build_time);
        assert!(dep.run_time);
        assert!(!dep.optional);
    }

    #[test]
    fn test_dependency_with_use_condition() {
        let mut dep = Dependency::new(PackageId::new("dev-libs", "openssl"));
        dep.use_flags = UseCondition::IfEnabled("ssl".to_string());
        dep.optional = true;

        assert!(dep.optional);
        match dep.use_flags {
            UseCondition::IfEnabled(flag) => assert_eq!(flag, "ssl"),
            _ => panic!("Expected IfEnabled"),
        }
    }

    #[test]
    fn test_dependency_build_vs_runtime() {
        let mut dep = Dependency::new(PackageId::new("dev-libs", "openssl"));
        dep.build_time = true;
        dep.run_time = false;

        assert!(dep.build_time);
        assert!(!dep.run_time);
    }

    #[test]
    fn test_dependency_with_slot() {
        let mut dep = Dependency::new(PackageId::new("dev-libs", "openssl"));
        dep.slot = Some("3".to_string());

        assert_eq!(dep.slot, Some("3".to_string()));
    }
}

mod resolution_tests {
    use super::*;

    #[test]
    fn test_internal_resolution_empty() {
        let resolution = InternalResolution {
            packages: vec![],
            build_order: vec![],
            download_size: 0,
            install_size: 0,
        };

        assert!(resolution.packages.is_empty());
        assert!(resolution.build_order.is_empty());
        assert_eq!(resolution.download_size, 0);
        assert_eq!(resolution.install_size, 0);
    }

    #[test]
    fn test_internal_resolution_single_package() {
        let pkg = PackageInfo {
            id: PackageId::new("sys-apps", "systemd"),
            version: semver::Version::parse("250.0.0").unwrap(),
            slot: "0".to_string(),
            description: "System and service manager".to_string(),
            homepage: None,
            license: "LGPL-2.1".to_string(),
            keywords: vec!["amd64".to_string()],
            use_flags: vec![],
            dependencies: vec![],
            build_dependencies: vec![],
            runtime_dependencies: vec![],
            source_url: Some("https://example.com/systemd.tar.gz".to_string()),
            source_hash: Some("abc123".to_string()),
            buck_target: "//packages/sys-apps/systemd:package".to_string(),
            size: 10000,
            installed_size: 50000,
        };

        let resolution = InternalResolution {
            packages: vec![pkg],
            build_order: vec![0],
            download_size: 10000,
            install_size: 50000,
        };

        assert_eq!(resolution.packages.len(), 1);
        assert_eq!(resolution.build_order, vec![0]);
        assert_eq!(resolution.download_size, 10000);
        assert_eq!(resolution.install_size, 50000);
    }
}
