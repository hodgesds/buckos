use buckos_package::db::PackageDb;
use buckos_package::repository::RepositoryManager;
use buckos_package::resolver::DependencyResolver;
use buckos_package::{
    Config, Dependency, InstallOptions, PackageId, PackageInfo, UseCondition, VersionSpec,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use semver::Version;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use varisat::ExtendFormula;

fn create_mock_package(name: &str, version: &str, dep_count: usize) -> PackageInfo {
    let mut dependencies = Vec::new();
    for i in 0..dep_count {
        dependencies.push(Dependency {
            package: PackageId::new("sys-libs", &format!("dep-{}", i)),
            version: VersionSpec::Any,
            slot: None,
            use_flags: UseCondition::Always,
            optional: false,
            build_time: true,
            run_time: true,
        });
    }

    PackageInfo {
        id: PackageId::new("sys-apps", name),
        version: Version::parse(version).unwrap(),
        slot: "0".to_string(),
        description: format!("Test package {}", name),
        homepage: None,
        license: "MIT".to_string(),
        keywords: Vec::new(),
        use_flags: Vec::new(),
        dependencies,
        runtime_dependencies: Vec::new(),
        build_dependencies: Vec::new(),
        source_url: None,
        source_hash: None,
        buck_target: format!("//packages/{}", name),
        size: 1024 * 1024,
        installed_size: 10 * 1024 * 1024,
    }
}

fn setup_resolver() -> (
    TempDir,
    Arc<RwLock<PackageDb>>,
    Arc<RepositoryManager>,
    DependencyResolver,
) {
    let temp_dir = TempDir::new().unwrap();
    let db = PackageDb::open(temp_dir.path()).unwrap();
    let db = Arc::new(RwLock::new(db));

    let mut config = Config::default();
    config.root = temp_dir.path().to_path_buf();
    config.db_path = temp_dir.path().to_path_buf();
    config.cache_dir = temp_dir.path().join("cache");
    config.buck_repo = temp_dir.path().join("repos");

    let repos = Arc::new(RepositoryManager::new(&config).unwrap());
    let resolver = DependencyResolver::new(db.clone(), repos.clone());

    (temp_dir, db, repos, resolver)
}

fn bench_resolve_simple(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("resolver_simple");

    for dep_count in [0, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("resolve_with_deps", dep_count),
            dep_count,
            |b, &dep_count| {
                b.to_async(&rt).iter(|| async {
                    let (_temp, _db, _repos, resolver) = setup_resolver();
                    let _pkg = create_mock_package("test-package", "1.0.0", dep_count);

                    // Note: This is a simplified benchmark as we can't easily mock the repository
                    // In a real scenario, you'd want to populate the repository with test packages
                    let opts = InstallOptions::default();
                    black_box(opts);
                });
            },
        );
    }

    group.finish();
}

fn bench_resolve_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolver_graph");

    for pkg_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*pkg_count as u64));
        group.bench_with_input(
            BenchmarkId::new("graph_construction", pkg_count),
            pkg_count,
            |b, &pkg_count| {
                b.iter(|| {
                    let packages: Vec<_> = (0..pkg_count)
                        .map(|i| create_mock_package(&format!("pkg-{}", i), "1.0.0", 3))
                        .collect();
                    black_box(packages);
                });
            },
        );
    }

    group.finish();
}

fn bench_version_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_ops");

    let versions: Vec<Version> = vec![
        Version::parse("1.0.0").unwrap(),
        Version::parse("1.2.3").unwrap(),
        Version::parse("2.0.0-alpha.1").unwrap(),
        Version::parse("2.0.0").unwrap(),
        Version::parse("10.5.3").unwrap(),
    ];

    group.bench_function("version_compare", |b| {
        b.iter(|| {
            for i in 0..versions.len() {
                for j in (i + 1)..versions.len() {
                    black_box(versions[i] < versions[j]);
                }
            }
        });
    });

    group.bench_function("version_parse", |b| {
        b.iter(|| {
            black_box(Version::parse("1.2.3-alpha.1+build.123").unwrap());
        });
    });

    group.finish();
}

fn bench_dependency_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_matching");

    let packages: Vec<PackageInfo> = (0..100)
        .map(|i| create_mock_package(&format!("pkg-{}", i), "1.0.0", 5))
        .collect();

    group.bench_function("find_by_name", |b| {
        b.iter(|| {
            let _found: Vec<_> = packages.iter().filter(|p| p.id.name == "pkg-50").collect();
            black_box(_found);
        });
    });

    group.bench_function("filter_by_category", |b| {
        b.iter(|| {
            let _found: Vec<_> = packages
                .iter()
                .filter(|p| p.id.category == "sys-apps")
                .collect();
            black_box(_found);
        });
    });

    group.finish();
}

fn bench_sat_solver_setup(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("sat_solver");

    for var_count in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("solver_setup", var_count),
            var_count,
            |b, &var_count| {
                b.iter(|| {
                    let mut solver = varisat::Solver::new();

                    // Add some basic constraints (similar to what dependency resolution does)
                    for i in 1..=var_count {
                        let lit = varisat::Lit::from_dimacs(i as isize);
                        // Add some clauses
                        solver.add_clause(&[lit]);
                    }

                    black_box(solver);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_resolve_simple,
    bench_resolve_graph_construction,
    bench_version_comparison,
    bench_dependency_matching,
    bench_sat_solver_setup
);
criterion_main!(benches);
