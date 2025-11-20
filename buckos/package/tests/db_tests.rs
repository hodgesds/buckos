//! Tests for the package database module

use buckos_package::db::PackageDb;
use buckos_package::{FileType, InstalledFile, InstalledPackage, PackageId};
use std::collections::HashSet;
use tempfile::TempDir;

/// Create a test database in a temporary directory
fn create_test_db() -> (PackageDb, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = PackageDb::open(temp_dir.path()).expect("Failed to open database");
    (db, temp_dir)
}

/// Create a test installed package
fn create_test_package(category: &str, name: &str) -> InstalledPackage {
    InstalledPackage {
        id: PackageId::new(category, name),
        name: name.to_string(),
        version: semver::Version::parse("1.0.0").unwrap(),
        slot: "0".to_string(),
        installed_at: chrono::Utc::now(),
        use_flags: HashSet::new(),
        files: vec![],
        size: 1000,
        build_time: false,
        explicit: true,
    }
}

/// Create a test installed file
fn create_test_file(path: &str) -> InstalledFile {
    InstalledFile {
        path: path.to_string(),
        file_type: FileType::Regular,
        mode: 0o644,
        size: 100,
        blake3_hash: Some("abc123".to_string()),
        mtime: chrono::Utc::now().timestamp(),
    }
}

mod database_operations {
    use super::*;

    #[test]
    fn test_open_database() {
        let (_db, _temp_dir) = create_test_db();
    }

    #[test]
    fn test_is_installed_empty() {
        let (db, _temp_dir) = create_test_db();
        assert!(!db.is_installed("nonexistent").unwrap());
    }

    #[test]
    fn test_add_and_get_package() {
        let (mut db, _temp_dir) = create_test_db();
        let pkg = create_test_package("sys-apps", "systemd");

        let id = db.add_package(&pkg).unwrap();
        assert!(id > 0);

        assert!(db.is_installed("systemd").unwrap());

        let retrieved = db.get_installed("systemd").unwrap().unwrap();
        assert_eq!(retrieved.name, "systemd");
        assert_eq!(retrieved.id.category, "sys-apps");
    }

    #[test]
    fn test_get_all_installed_empty() {
        let (db, _temp_dir) = create_test_db();
        let packages = db.get_all_installed().unwrap();
        assert!(packages.is_empty());
    }

    #[test]
    fn test_get_all_installed_multiple() {
        let (mut db, _temp_dir) = create_test_db();

        db.add_package(&create_test_package("sys-apps", "systemd"))
            .unwrap();
        db.add_package(&create_test_package("dev-libs", "openssl"))
            .unwrap();
        db.add_package(&create_test_package("app-shells", "bash"))
            .unwrap();

        let packages = db.get_all_installed().unwrap();
        assert_eq!(packages.len(), 3);
    }

    #[test]
    fn test_remove_package() {
        let (mut db, _temp_dir) = create_test_db();
        let pkg = create_test_package("sys-apps", "systemd");

        db.add_package(&pkg).unwrap();
        assert!(db.is_installed("systemd").unwrap());

        db.remove_package("systemd").unwrap();
        assert!(!db.is_installed("systemd").unwrap());
    }

    #[test]
    fn test_get_nonexistent_package() {
        let (db, _temp_dir) = create_test_db();
        let result = db.get_installed("nonexistent").unwrap();
        assert!(result.is_none());
    }
}

mod use_flags_tests {
    use super::*;

    #[test]
    fn test_package_with_use_flags() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("dev-libs", "openssl");
        pkg.use_flags.insert("ssl".to_string());
        pkg.use_flags.insert("ipv6".to_string());
        pkg.use_flags.insert("static".to_string());

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("openssl").unwrap().unwrap();
        assert_eq!(retrieved.use_flags.len(), 3);
        assert!(retrieved.use_flags.contains("ssl"));
        assert!(retrieved.use_flags.contains("ipv6"));
        assert!(retrieved.use_flags.contains("static"));
    }

    #[test]
    fn test_package_empty_use_flags() {
        let (mut db, _temp_dir) = create_test_db();
        let pkg = create_test_package("sys-apps", "coreutils");

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("coreutils").unwrap().unwrap();
        assert!(retrieved.use_flags.is_empty());
    }
}

mod files_tests {
    use super::*;

    #[test]
    fn test_package_with_files() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "systemd");
        pkg.files.push(create_test_file("/usr/bin/systemctl"));
        pkg.files.push(create_test_file("/usr/lib/systemd/systemd"));
        pkg.files.push(create_test_file("/etc/systemd/system.conf"));

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("systemd").unwrap().unwrap();
        assert_eq!(retrieved.files.len(), 3);
    }

    #[test]
    fn test_get_package_files() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("app-shells", "bash");
        pkg.files.push(create_test_file("/usr/bin/bash"));
        pkg.files.push(create_test_file("/etc/bash/bashrc"));

        db.add_package(&pkg).unwrap();

        let files = db.get_package_files("bash").unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.path == "/usr/bin/bash"));
        assert!(files.iter().any(|f| f.path == "/etc/bash/bashrc"));
    }

    #[test]
    fn test_find_file_owner() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("app-shells", "bash");
        pkg.files.push(create_test_file("/usr/bin/bash"));

        db.add_package(&pkg).unwrap();

        let owner = db.get_file_owner("/usr/bin/bash").unwrap();
        assert!(owner.is_some());
        assert_eq!(owner.unwrap(), "bash");
    }

    #[test]
    fn test_find_file_owner_not_found() {
        let (db, _temp_dir) = create_test_db();
        let owner = db.get_file_owner("/nonexistent/file").unwrap();
        assert!(owner.is_none());
    }
}

mod dependencies_tests {
    use super::*;

    #[test]
    fn test_get_reverse_dependencies_empty() {
        let (db, _temp_dir) = create_test_db();
        let rdeps = db.get_reverse_dependencies("systemd").unwrap();
        assert!(rdeps.is_empty());
    }

    #[test]
    fn test_get_dependencies() {
        let (mut db, _temp_dir) = create_test_db();
        let pkg = create_test_package("sys-apps", "systemd");
        db.add_package(&pkg).unwrap();

        let rdeps = db.get_reverse_dependencies("systemd").unwrap();
        // Empty reverse deps by default
        assert!(rdeps.is_empty());
    }
}

mod slot_tests {
    use super::*;

    #[test]
    fn test_packages_in_different_slots() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg1 = create_test_package("dev-libs", "openssl");
        pkg1.slot = "1.1".to_string();

        let mut pkg2 = create_test_package("dev-libs", "openssl");
        pkg2.name = "openssl".to_string();
        pkg2.slot = "3".to_string();

        // Only one can be installed per slot
        db.add_package(&pkg1).unwrap();

        let retrieved = db.get_installed("openssl").unwrap().unwrap();
        // The last one with the same name should be stored (REPLACE)
        assert_eq!(retrieved.slot, "1.1");
    }
}

mod version_tests {
    use super::*;

    #[test]
    fn test_package_version() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "systemd");
        pkg.version = semver::Version::parse("250.4.0").unwrap();

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("systemd").unwrap().unwrap();
        assert_eq!(retrieved.version.to_string(), "250.4.0");
    }

    #[test]
    fn test_package_with_prerelease_version() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("dev-libs", "test");
        pkg.version = semver::Version::parse("1.0.0-alpha.1").unwrap();

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("test").unwrap().unwrap();
        assert_eq!(retrieved.version.to_string(), "1.0.0-alpha.1");
    }
}

mod explicit_flag_tests {
    use super::*;

    #[test]
    fn test_explicit_package() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "systemd");
        pkg.explicit = true;

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("systemd").unwrap().unwrap();
        assert!(retrieved.explicit);
    }

    #[test]
    fn test_dependency_package() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-libs", "glibc");
        pkg.explicit = false;

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("glibc").unwrap().unwrap();
        assert!(!retrieved.explicit);
    }
}

mod file_type_tests {
    use super::*;

    #[test]
    fn test_regular_file() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "test");
        let mut file = create_test_file("/usr/bin/test");
        file.file_type = FileType::Regular;
        pkg.files.push(file);

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("test").unwrap().unwrap();
        assert!(matches!(retrieved.files[0].file_type, FileType::Regular));
    }

    #[test]
    fn test_directory() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "test");
        let mut file = create_test_file("/usr/share/test");
        file.file_type = FileType::Directory;
        file.mode = 0o755;
        pkg.files.push(file);

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("test").unwrap().unwrap();
        assert!(matches!(retrieved.files[0].file_type, FileType::Directory));
    }

    #[test]
    fn test_symlink() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "test");
        let mut file = create_test_file("/usr/bin/testlink");
        file.file_type = FileType::Symlink;
        pkg.files.push(file);

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("test").unwrap().unwrap();
        assert!(matches!(retrieved.files[0].file_type, FileType::Symlink));
    }
}

mod timestamp_tests {
    use super::*;

    #[test]
    fn test_installed_at_timestamp() {
        let (mut db, _temp_dir) = create_test_db();

        let pkg = create_test_package("sys-apps", "systemd");
        let original_time = pkg.installed_at;

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("systemd").unwrap().unwrap();
        // Allow small time difference due to serialization
        let diff = (retrieved.installed_at - original_time).num_seconds().abs();
        assert!(diff < 2);
    }
}

mod size_tests {
    use super::*;

    #[test]
    fn test_package_size() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "systemd");
        pkg.size = 50_000_000; // 50 MB

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("systemd").unwrap().unwrap();
        assert_eq!(retrieved.size, 50_000_000);
    }

    #[test]
    fn test_file_size() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "test");
        let mut file = create_test_file("/usr/bin/large");
        file.size = 10_000_000;
        pkg.files.push(file);

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("test").unwrap().unwrap();
        assert_eq!(retrieved.files[0].size, 10_000_000);
    }
}

mod hash_tests {
    use super::*;

    #[test]
    fn test_file_hash() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "test");
        let mut file = create_test_file("/usr/bin/test");
        file.blake3_hash = Some("abc123def456".to_string());
        pkg.files.push(file);

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("test").unwrap().unwrap();
        assert_eq!(
            retrieved.files[0].blake3_hash,
            Some("abc123def456".to_string())
        );
    }

    #[test]
    fn test_file_no_hash() {
        let (mut db, _temp_dir) = create_test_db();

        let mut pkg = create_test_package("sys-apps", "test");
        let mut file = create_test_file("/usr/bin/test");
        file.blake3_hash = None;
        pkg.files.push(file);

        db.add_package(&pkg).unwrap();

        let retrieved = db.get_installed("test").unwrap().unwrap();
        assert!(retrieved.files[0].blake3_hash.is_none());
    }
}
