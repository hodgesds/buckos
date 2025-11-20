use buckos_package::cache::PackageCache;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::Write;
use tempfile::TempDir;

fn setup_cache() -> (TempDir, PackageCache) {
    let temp_dir = TempDir::new().unwrap();
    let cache = PackageCache::new(temp_dir.path()).unwrap();
    (temp_dir, cache)
}

fn create_test_file(path: &std::path::Path, size: usize) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    let data = vec![0u8; size];
    file.write_all(&data)?;
    Ok(())
}

fn bench_cache_hash_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hashing");

    let temp_dir = TempDir::new().unwrap();

    for size in [1024, 10 * 1024, 100 * 1024, 1024 * 1024].iter() {
        let test_file = temp_dir.path().join(format!("test-{}.dat", size));
        create_test_file(&test_file, *size).unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("sha256", size), size, |b, _| {
            b.iter(|| {
                black_box(buckos_package::cache::compute_sha256(&test_file).unwrap());
            });
        });

        group.bench_with_input(BenchmarkId::new("blake3", size), size, |b, _| {
            b.iter(|| {
                black_box(buckos_package::cache::compute_blake3(&test_file).unwrap());
            });
        });
    }

    group.finish();
}

fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_ops");

    group.bench_function("cache_creation", |b| {
        b.iter_batched(
            || TempDir::new().unwrap(),
            |temp_dir| {
                black_box(PackageCache::new(temp_dir.path()).unwrap());
            },
            criterion::BatchSize::SmallInput,
        );
    });

    let (_temp, cache) = setup_cache();

    group.bench_function("has_distfile", |b| {
        b.iter(|| {
            black_box(cache.has_distfile("nonexistent.tar.gz"));
        });
    });

    group.bench_function("has_package", |b| {
        b.iter(|| {
            black_box(cache.has_package("sys-apps", "test", "1.0.0"));
        });
    });

    group.bench_function("distfile_path", |b| {
        b.iter(|| {
            black_box(cache.distfile_path("test.tar.gz"));
        });
    });

    group.bench_function("package_path", |b| {
        b.iter(|| {
            black_box(cache.package_path("sys-apps", "test", "1.0.0"));
        });
    });

    group.finish();
}

fn bench_cache_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_compression");

    let temp_dir = TempDir::new().unwrap();

    for size in [10 * 1024, 100 * 1024, 1024 * 1024].iter() {
        let test_data = vec![0u8; *size];

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("gzip_compress", size), size, |b, _| {
            b.iter(|| {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&test_data).unwrap();
                black_box(encoder.finish().unwrap());
            });
        });

        group.bench_with_input(BenchmarkId::new("zstd_compress", size), size, |b, _| {
            b.iter(|| {
                black_box(zstd::encode_all(&test_data[..], 3).unwrap());
            });
        });

        // Create compressed data for decompression benchmarks
        let gzip_compressed = {
            use flate2::write::GzEncoder;
            use flate2::Compression;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&test_data).unwrap();
            encoder.finish().unwrap()
        };

        let zstd_compressed = zstd::encode_all(&test_data[..], 3).unwrap();

        group.bench_with_input(BenchmarkId::new("gzip_decompress", size), size, |b, _| {
            b.iter(|| {
                use flate2::read::GzDecoder;
                use std::io::Read;
                let mut decoder = GzDecoder::new(&gzip_compressed[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).unwrap();
                black_box(decompressed);
            });
        });

        group.bench_with_input(BenchmarkId::new("zstd_decompress", size), size, |b, _| {
            b.iter(|| {
                black_box(zstd::decode_all(&zstd_compressed[..]).unwrap());
            });
        });
    }

    group.finish();
}

fn bench_cache_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_file_ops");

    let temp_dir = TempDir::new().unwrap();

    for file_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_files", file_count),
            file_count,
            |b, &count| {
                b.iter_batched(
                    || TempDir::new().unwrap(),
                    |temp| {
                        for i in 0..count {
                            let path = temp.path().join(format!("file-{}.txt", i));
                            create_test_file(&path, 1024).unwrap();
                        }
                        black_box(temp);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("check_exists", file_count),
            file_count,
            |b, &count| {
                // Setup: create files
                let temp = TempDir::new().unwrap();
                for i in 0..count {
                    let path = temp.path().join(format!("file-{}.txt", i));
                    create_test_file(&path, 1024).unwrap();
                }

                b.iter(|| {
                    for i in 0..count {
                        let path = temp.path().join(format!("file-{}.txt", i));
                        black_box(path.exists());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_cache_tar_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_tar");

    let temp_dir = TempDir::new().unwrap();

    // Create a test archive
    for file_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("tar_create", file_count),
            file_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let temp = TempDir::new().unwrap();
                        let files_dir = temp.path().join("files");
                        std::fs::create_dir(&files_dir).unwrap();
                        for i in 0..count {
                            let path = files_dir.join(format!("file-{}.txt", i));
                            create_test_file(&path, 1024).unwrap();
                        }
                        (temp, files_dir)
                    },
                    |(temp, files_dir)| {
                        let archive_path = temp.path().join("archive.tar");
                        let file = std::fs::File::create(&archive_path).unwrap();
                        let mut archive = tar::Builder::new(file);
                        archive.append_dir_all(".", &files_dir).unwrap();
                        archive.finish().unwrap();
                        black_box(archive_path);
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
    bench_cache_hash_computation,
    bench_cache_operations,
    bench_cache_compression,
    bench_cache_file_operations,
    bench_cache_tar_operations
);
criterion_main!(benches);
