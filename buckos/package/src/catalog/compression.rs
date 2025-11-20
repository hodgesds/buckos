//! Compression library packages
//!
//! This module defines compression packages including:
//! - zlib
//! - xz-utils
//! - zstd
//! - bzip2
//! - lz4
//! - brotli

use super::{dep, dep_build, use_flag};
use crate::types::{Dependency, PackageId, PackageInfo, UseFlag, VersionSpec};
use semver::Version;

/// Get all compression packages
pub fn get_packages() -> Vec<PackageInfo> {
    vec![
        // zlib
        zlib_1_3_1(),
        // xz-utils (includes liblzma)
        xz_utils_5_4_6(),
        xz_utils_5_6_1(),
        // zstd
        zstd_1_5_5(),
        zstd_1_5_6(),
        // bzip2
        bzip2_1_0_8(),
        // lz4
        lz4_1_9_4(),
        // brotli
        brotli_1_1_0(),
        // libarchive
        libarchive_3_7_2(),
        // pigz
        pigz_2_8(),
        // pbzip2
        pbzip2_1_1_13(),
        // snappy
        snappy_1_1_10(),
        // lzip
        lzip_1_24(),
        // lzop
        lzop_1_04(),
    ]
}

fn zlib_1_3_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "zlib"),
        version: Version::new(1, 3, 1),
        slot: "0/1".to_string(),
        description: "Standard compression library".to_string(),
        homepage: Some("https://zlib.net/".to_string()),
        license: "ZLIB".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("minizip", "Build minizip library", false),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://zlib.net/zlib-1.3.1.tar.xz".to_string()),
        source_hash: Some(
            "38ef96b8dfe510d42707d9c781877914792541133e1870841463bfa73f883e32".to_string(),
        ),
        buck_target: "//sys-libs/zlib:zlib-1.3.1".to_string(),
        size: 600_000,
        installed_size: 1_500_000,
    }
}

fn xz_utils_5_4_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "xz-utils"),
        version: Version::new(5, 4, 6),
        slot: "0/5.4".to_string(),
        description: "XZ Utils: xz, unxz, lzma, unlzma and LZMA SDK".to_string(),
        homepage: Some("https://tukaani.org/xz/".to_string()),
        license: "public-domain LGPL-2.1+ GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("doc", "Build documentation", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/tukaani-project/xz/releases/download/v5.4.6/xz-5.4.6.tar.xz"
                .to_string(),
        ),
        source_hash: Some(
            "b1d45295d3f71f25a4c9101bd7c8d16cb56348bbef3bbc738da0351e17c73317".to_string(),
        ),
        buck_target: "//app-arch/xz-utils:xz-utils-5.4.6".to_string(),
        size: 1_400_000,
        installed_size: 3_500_000,
    }
}

fn xz_utils_5_6_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "xz-utils"),
        version: Version::new(5, 6, 1),
        slot: "0/5.6".to_string(),
        description: "XZ Utils: xz, unxz, lzma, unlzma and LZMA SDK".to_string(),
        homepage: Some("https://tukaani.org/xz/".to_string()),
        license: "public-domain LGPL-2.1+ GPL-2+".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("doc", "Build documentation", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/tukaani-project/xz/releases/download/v5.6.1/xz-5.6.1.tar.xz"
                .to_string(),
        ),
        source_hash: Some(
            "c1d45295d3f71f25a4c9101bd7c8d16cb56348bbef3bbc738da0351e17c73318".to_string(),
        ),
        buck_target: "//app-arch/xz-utils:xz-utils-5.6.1".to_string(),
        size: 1_500_000,
        installed_size: 3_700_000,
    }
}

fn zstd_1_5_5() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "zstd"),
        version: Version::new(1, 5, 5),
        slot: "0/1".to_string(),
        description: "Zstandard: fast real-time compression algorithm".to_string(),
        homepage: Some("https://facebook.github.io/zstd/".to_string()),
        license: "|| ( BSD GPL-2 )".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("lz4", "Enable LZ4 support", true),
            use_flag("lzma", "Enable LZMA support", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("zlib", "Enable zlib support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "lz4"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/facebook/zstd/releases/download/v1.5.5/zstd-1.5.5.tar.gz"
                .to_string(),
        ),
        source_hash: Some(
            "9c4396cc829cfae319a6e2615202e82aad41372073482fce286fac78646d3ee4".to_string(),
        ),
        buck_target: "//app-arch/zstd:zstd-1.5.5".to_string(),
        size: 2_000_000,
        installed_size: 5_000_000,
    }
}

fn zstd_1_5_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "zstd"),
        version: Version::new(1, 5, 6),
        slot: "0/1".to_string(),
        description: "Zstandard: fast real-time compression algorithm".to_string(),
        homepage: Some("https://facebook.github.io/zstd/".to_string()),
        license: "|| ( BSD GPL-2 )".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("lz4", "Enable LZ4 support", true),
            use_flag("lzma", "Enable LZMA support", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("zlib", "Enable zlib support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "lz4"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/facebook/zstd/releases/download/v1.5.6/zstd-1.5.6.tar.gz"
                .to_string(),
        ),
        source_hash: Some(
            "a4396cc829cfae319a6e2615202e82aad41372073482fce286fac78646d3ee5".to_string(),
        ),
        buck_target: "//app-arch/zstd:zstd-1.5.6".to_string(),
        size: 2_100_000,
        installed_size: 5_200_000,
    }
}

fn bzip2_1_0_8() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "bzip2"),
        version: Version::new(1, 0, 8),
        slot: "0/1".to_string(),
        description: "High-quality data compressor".to_string(),
        homepage: Some("https://sourceware.org/bzip2/".to_string()),
        license: "BZIP2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("split-usr", "Split /usr paths", false),
            use_flag("static", "Build static binary", false),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz".to_string()),
        source_hash: Some(
            "ab5a03176ee106d3f0fa90e381da478ddae405918153cca248e682cd0c4a2269".to_string(),
        ),
        buck_target: "//app-arch/bzip2:bzip2-1.0.8".to_string(),
        size: 800_000,
        installed_size: 1_500_000,
    }
}

fn lz4_1_9_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "lz4"),
        version: Version::new(1, 9, 4),
        slot: "0/1".to_string(),
        description: "Extremely fast compression algorithm".to_string(),
        homepage: Some("https://lz4.github.io/lz4/".to_string()),
        license: "|| ( BSD-2 GPL-2 )".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("static-libs", "Build static libraries", false)],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/lz4/lz4/archive/refs/tags/v1.9.4.tar.gz".to_string()),
        source_hash: Some(
            "0b0e3aa07c8c063ddf40b082bdf7e37a1562bda40a0ff5272957f3e987e0e54b".to_string(),
        ),
        buck_target: "//app-arch/lz4:lz4-1.9.4".to_string(),
        size: 400_000,
        installed_size: 1_000_000,
    }
}

fn brotli_1_1_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "brotli"),
        version: Version::new(1, 1, 0),
        slot: "0/1".to_string(),
        description: "Generic-purpose lossless compression algorithm".to_string(),
        homepage: Some("https://github.com/google/brotli".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("python", "Build Python bindings", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/google/brotli/archive/refs/tags/v1.1.0.tar.gz".to_string(),
        ),
        source_hash: Some(
            "e720a6ca29428b803f4ad165371771f5398faba397edf6778837a18599ea13ff".to_string(),
        ),
        buck_target: "//app-arch/brotli:brotli-1.1.0".to_string(),
        size: 500_000,
        installed_size: 1_200_000,
    }
}

fn libarchive_3_7_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "libarchive"),
        version: Version::new(3, 7, 2),
        slot: "0/13".to_string(),
        description: "Multi-format archive and compression library".to_string(),
        homepage: Some("https://www.libarchive.org/".to_string()),
        license: "BSD BSD-2 BSD-4 public-domain".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("blake2", "Enable BLAKE2 support", false),
            use_flag("bzip2", "Enable bzip2 support", true),
            use_flag("e2fsprogs", "Enable ext2/3/4 support", true),
            use_flag("expat", "Use expat for XML parsing", false),
            use_flag("iconv", "Enable iconv support", true),
            use_flag("lz4", "Enable LZ4 support", true),
            use_flag("lzma", "Enable LZMA support", true),
            use_flag("lzo", "Enable LZO support", false),
            use_flag("nettle", "Use nettle for crypto", false),
            use_flag("openssl", "Use OpenSSL for crypto", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("xattr", "Enable extended attributes", true),
            use_flag("zlib", "Enable zlib support", true),
            use_flag("zstd", "Enable zstd support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "bzip2"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "zstd"),
            dep("app-arch", "lz4"),
            dep("dev-libs", "openssl"),
            dep("sys-libs", "acl"),
            dep("sys-libs", "attr"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.libarchive.org/downloads/libarchive-3.7.2.tar.xz".to_string(),
        ),
        source_hash: Some(
            "04357661e6717b6941682cde02ad741ae4819c67a260593dfb2431861b251acb".to_string(),
        ),
        buck_target: "//app-arch/libarchive:libarchive-3.7.2".to_string(),
        size: 7_000_000,
        installed_size: 15_000_000,
    }
}

fn pigz_2_8() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "pigz"),
        version: Version::new(2, 8, 0),
        slot: "0".to_string(),
        description: "Parallel implementation of gzip".to_string(),
        homepage: Some("https://zlib.net/pigz/".to_string()),
        license: "ZLIB".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("static", "Build static binary", false),
            use_flag("symlink", "Create gzip/gunzip symlinks", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![dep("sys-libs", "zlib")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://zlib.net/pigz/pigz-2.8.tar.gz".to_string()),
        source_hash: Some(
            "eb872b4f0e1f0ebe59c9f7bd8c506c4204893ba6a8492de31df416f0d5170fd0".to_string(),
        ),
        buck_target: "//app-arch/pigz:pigz-2.8".to_string(),
        size: 80_000,
        installed_size: 200_000,
    }
}

fn pbzip2_1_1_13() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "pbzip2"),
        version: Version::new(1, 1, 13),
        slot: "0".to_string(),
        description: "Parallel implementation of bzip2".to_string(),
        homepage: Some("https://launchpad.net/pbzip2".to_string()),
        license: "BZIP2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("static", "Build static binary", false),
            use_flag("symlink", "Create bzip2/bunzip2 symlinks", false),
        ],
        dependencies: vec![dep("app-arch", "bzip2")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://launchpad.net/pbzip2/1.1/1.1.13/+download/pbzip2-1.1.13.tar.gz".to_string(),
        ),
        source_hash: Some(
            "8fd13eaaa266f7ee91f85c1ea97c86d9c9cc985969db9059cdebcb1e1b7bdbe6".to_string(),
        ),
        buck_target: "//app-arch/pbzip2:pbzip2-1.1.13".to_string(),
        size: 50_000,
        installed_size: 150_000,
    }
}

fn snappy_1_1_10() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "snappy"),
        version: Version::new(1, 1, 10),
        slot: "0/1".to_string(),
        description: "Fast compressor/decompressor library".to_string(),
        homepage: Some("https://google.github.io/snappy/".to_string()),
        license: "BSD".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/google/snappy/archive/refs/tags/1.1.10.tar.gz".to_string(),
        ),
        source_hash: Some(
            "49d831bffcc5f3d01b306c5a5bf9e19aa18b04b7fdf6e71e7a3f13c1b78fb6cc".to_string(),
        ),
        buck_target: "//app-arch/snappy:snappy-1.1.10".to_string(),
        size: 500_000,
        installed_size: 1_000_000,
    }
}

fn lzip_1_24() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "lzip"),
        version: Version::new(1, 24, 0),
        slot: "0".to_string(),
        description: "Lossless data compressor based on LZMA".to_string(),
        homepage: Some("https://www.nongnu.org/lzip/".to_string()),
        license: "GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://download.savannah.gnu.org/releases/lzip/lzip-1.24.tar.gz".to_string(),
        ),
        source_hash: Some(
            "d3b3d777e03b0ab9f0c925e3c6fc48da2c53f01d4714e1669a84e9d217c0f4eb".to_string(),
        ),
        buck_target: "//app-arch/lzip:lzip-1.24".to_string(),
        size: 150_000,
        installed_size: 400_000,
    }
}

fn lzop_1_04() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "lzop"),
        version: Version::new(1, 4, 0),
        slot: "0".to_string(),
        description: "Utility for fast compression based on LZO".to_string(),
        homepage: Some("https://www.lzop.org/".to_string()),
        license: "GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![],
        dependencies: vec![dep("dev-libs", "lzo")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://www.lzop.org/download/lzop-1.04.tar.gz".to_string()),
        source_hash: Some(
            "7e72b62a8a60aff5200a047eea0773a8fb205caf7b4e8baf76d1d4c48ce9fce8".to_string(),
        ),
        buck_target: "//app-arch/lzop:lzop-1.04".to_string(),
        size: 150_000,
        installed_size: 300_000,
    }
}
