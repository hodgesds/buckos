use buckos_package::db::PackageDb;
use buckos_package::{InstalledFile, InstalledPackage, PackageId};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use semver::Version;
use std::collections::HashSet;
use tempfile::TempDir;

fn setup_db() -> (TempDir, PackageDb) {
    let temp_dir = TempDir::new().unwrap();
    let db = PackageDb::open(temp_dir.path()).unwrap();
    (temp_dir, db)
}

fn create_test_package(name: &str, version: &str, file_count: usize) -> InstalledPackage {
    let mut files = Vec::new();
    for i in 0..file_count {
        files.push(InstalledFile {
            path: format!("/usr/bin/{}-{}", name, i),
            file_type: buckos_package::FileType::Regular,
            mode: 0o755,
            size: 1024,
            blake3_hash: Some("test_hash".to_string()),
            mtime: chrono::Utc::now().timestamp(),
        });
    }

    InstalledPackage {
        id: PackageId::new("sys-apps", name),
        name: name.to_string(),
        version: Version::parse(version).unwrap(),
        slot: "0".to_string(),
        installed_at: chrono::Utc::now(),
        use_flags: HashSet::new(),
        files,
        size: 1024 * file_count as u64,
        build_time: false,
        explicit: true,
    }
}

fn bench_db_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_insert");

    for file_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("insert_package", file_count),
            file_count,
            |b, &file_count| {
                b.iter_batched(
                    || {
                        let (_temp, db) = setup_db();
                        let pkg = create_test_package("test-package", "1.0.0", file_count);
                        (db, pkg)
                    },
                    |(mut db, pkg)| {
                        db.add_package(&pkg).unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_db_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_query");

    // Setup database with packages
    let (_temp, mut db) = setup_db();
    for i in 0..100 {
        let pkg = create_test_package(&format!("package-{}", i), "1.0.0", 10);
        db.add_package(&pkg).unwrap();
    }

    group.bench_function("is_installed", |b| {
        b.iter(|| {
            black_box(db.is_installed("package-50").unwrap());
        });
    });

    group.bench_function("get_installed", |b| {
        b.iter(|| {
            black_box(db.get_installed("package-50").unwrap());
        });
    });

    group.bench_function("get_all_installed", |b| {
        b.iter(|| {
            black_box(db.get_all_installed().unwrap());
        });
    });

    group.finish();
}

fn bench_db_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_file_ops");

    // Setup database with a package that has many files
    let (_temp, mut db) = setup_db();
    let pkg = create_test_package("large-package", "1.0.0", 1000);
    db.add_package(&pkg).unwrap();

    group.bench_function("get_file_owner", |b| {
        b.iter(|| {
            black_box(db.get_file_owner("/usr/bin/large-package-500").unwrap());
        });
    });

    group.bench_function("get_package_files", |b| {
        b.iter(|| {
            black_box(db.get_package_files("large-package").unwrap());
        });
    });

    group.finish();
}

fn bench_db_dependency_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_dependencies");

    // Setup database with packages and dependencies
    let (_temp, mut db) = setup_db();

    // Create a package with dependencies
    let pkg = create_test_package("dependent-package", "1.0.0", 10);
    db.add_package(&pkg).unwrap();

    // Add some dependencies
    for i in 0..10 {
        let dep_pkg = create_test_package(&format!("dep-{}", i), "1.0.0", 5);
        db.add_package(&dep_pkg).unwrap();
    }

    group.bench_function("get_reverse_dependencies", |b| {
        b.iter(|| {
            black_box(db.get_reverse_dependencies("dep-5").unwrap());
        });
    });

    group.finish();
}

fn bench_db_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_batch");

    for package_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_insert", package_count),
            package_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let (_temp, db) = setup_db();
                        let packages: Vec<_> = (0..count)
                            .map(|i| create_test_package(&format!("pkg-{}", i), "1.0.0", 10))
                            .collect();
                        (db, packages)
                    },
                    |(mut db, packages)| {
                        for pkg in packages {
                            db.add_package(&pkg).unwrap();
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_db_insert,
    bench_db_query,
    bench_db_file_operations,
    bench_db_dependency_operations,
    bench_db_batch_operations
);
criterion_main!(benches);
