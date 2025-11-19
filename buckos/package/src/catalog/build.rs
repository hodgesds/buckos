//! Build system packages
//!
//! This module defines build system packages including:
//! - CMake
//! - Meson
//! - Ninja
//! - SCons

use crate::types::{PackageId, PackageInfo, Dependency, UseFlag, VersionSpec};
use super::{dep, dep_build, dep_use, use_flag};
use semver::Version;

/// Get all build system packages
pub fn get_packages() -> Vec<PackageInfo> {
    vec![
        // CMake
        cmake_3_28_3(),
        cmake_3_29_0(),

        // Meson
        meson_1_3_2(),
        meson_1_4_0(),

        // Ninja
        ninja_1_11_1(),
        ninja_1_12_0(),

        // SCons
        scons_4_7_0(),

        // Bazel
        bazel_7_1_0(),

        // Buck2
        buck2_2024_01(),
    ]
}

fn cmake_3_28_3() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "cmake"),
        version: Version::new(3, 28, 3),
        slot: "0".to_string(),
        description: "Cross platform Make".to_string(),
        homepage: Some("https://cmake.org/".to_string()),
        license: "CMake".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("doc", "Build documentation", false),
            use_flag("emacs", "Install Emacs support files", false),
            use_flag("ncurses", "Enable ncurses support for ccmake", true),
            use_flag("qt5", "Build cmake-gui with Qt5", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "zstd"),
            dep("net-misc", "curl"),
            dep("dev-libs", "expat"),
            dep("dev-libs", "jsoncpp"),
            dep("dev-libs", "libuv"),
            dep("app-arch", "libarchive"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://cmake.org/files/v3.28/cmake-3.28.3.tar.gz".to_string()),
        source_hash: Some("72b7570e5c8593de6ac4ab433b73eab18c5fb328f3e74f53e5e8f4a93f8c8f0e".to_string()),
        buck_target: "//dev-util/cmake:cmake-3.28.3".to_string(),
        size: 11_000_000,
        installed_size: 55_000_000,
    }
}

fn cmake_3_29_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "cmake"),
        version: Version::new(3, 29, 0),
        slot: "0".to_string(),
        description: "Cross platform Make".to_string(),
        homepage: Some("https://cmake.org/".to_string()),
        license: "CMake".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("doc", "Build documentation", false),
            use_flag("emacs", "Install Emacs support files", false),
            use_flag("ncurses", "Enable ncurses support for ccmake", true),
            use_flag("qt5", "Build cmake-gui with Qt5", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "zstd"),
            dep("net-misc", "curl"),
            dep("dev-libs", "expat"),
            dep("dev-libs", "jsoncpp"),
            dep("dev-libs", "libuv"),
            dep("app-arch", "libarchive"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://cmake.org/files/v3.29/cmake-3.29.0.tar.gz".to_string()),
        source_hash: Some("82b7570e5c8593de6ac4ab433b73eab18c5fb328f3e74f53e5e8f4a93f8c8f1f".to_string()),
        buck_target: "//dev-util/cmake:cmake-3.29.0".to_string(),
        size: 11_200_000,
        installed_size: 56_000_000,
    }
}

fn meson_1_3_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "meson"),
        version: Version::new(1, 3, 2),
        slot: "0".to_string(),
        description: "High performance build system".to_string(),
        homepage: Some("https://mesonbuild.com/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![
            dep("dev-lang", "python"),
        ],
        build_dependencies: vec![
            dep_build("dev-python", "setuptools"),
        ],
        runtime_dependencies: vec![
            dep("dev-util", "ninja"),
        ],
        source_url: Some("https://github.com/mesonbuild/meson/releases/download/1.3.2/meson-1.3.2.tar.gz".to_string()),
        source_hash: Some("4533a43c34548edd1f63a276b42d95e4dc4a7df0c71f8905d6c7a54f2f9a3b5c".to_string()),
        buck_target: "//dev-util/meson:meson-1.3.2".to_string(),
        size: 2_100_000,
        installed_size: 8_000_000,
    }
}

fn meson_1_4_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "meson"),
        version: Version::new(1, 4, 0),
        slot: "0".to_string(),
        description: "High performance build system".to_string(),
        homepage: Some("https://mesonbuild.com/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![
            dep("dev-lang", "python"),
        ],
        build_dependencies: vec![
            dep_build("dev-python", "setuptools"),
        ],
        runtime_dependencies: vec![
            dep("dev-util", "ninja"),
        ],
        source_url: Some("https://github.com/mesonbuild/meson/releases/download/1.4.0/meson-1.4.0.tar.gz".to_string()),
        source_hash: Some("5533a43c34548edd1f63a276b42d95e4dc4a7df0c71f8905d6c7a54f2f9a3b6d".to_string()),
        buck_target: "//dev-util/meson:meson-1.4.0".to_string(),
        size: 2_200_000,
        installed_size: 8_500_000,
    }
}

fn ninja_1_11_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "ninja"),
        version: Version::new(1, 11, 1),
        slot: "0".to_string(),
        description: "Small build system with focus on speed".to_string(),
        homepage: Some("https://ninja-build.org/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("doc", "Build documentation", false),
            use_flag("emacs", "Install Emacs support files", false),
            use_flag("test", "Build tests", false),
            use_flag("vim-syntax", "Install Vim syntax files", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/ninja-build/ninja/archive/refs/tags/v1.11.1.tar.gz".to_string()),
        source_hash: Some("31747ae633213f1eda3842686f83c2aa1412e0f5691d1c14dbbcc67fe7400cea".to_string()),
        buck_target: "//dev-util/ninja:ninja-1.11.1".to_string(),
        size: 240_000,
        installed_size: 600_000,
    }
}

fn ninja_1_12_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "ninja"),
        version: Version::new(1, 12, 0),
        slot: "0".to_string(),
        description: "Small build system with focus on speed".to_string(),
        homepage: Some("https://ninja-build.org/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("doc", "Build documentation", false),
            use_flag("emacs", "Install Emacs support files", false),
            use_flag("test", "Build tests", false),
            use_flag("vim-syntax", "Install Vim syntax files", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/ninja-build/ninja/archive/refs/tags/v1.12.0.tar.gz".to_string()),
        source_hash: Some("41747ae633213f1eda3842686f83c2aa1412e0f5691d1c14dbbcc67fe7400ceb".to_string()),
        buck_target: "//dev-util/ninja:ninja-1.12.0".to_string(),
        size: 250_000,
        installed_size: 620_000,
    }
}

fn scons_4_7_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "scons"),
        version: Version::new(4, 7, 0),
        slot: "0".to_string(),
        description: "Extensible Python-based build utility".to_string(),
        homepage: Some("https://scons.org/".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("doc", "Build documentation", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![
            dep("dev-lang", "python"),
        ],
        build_dependencies: vec![
            dep_build("dev-python", "setuptools"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/SCons/scons/archive/refs/tags/4.7.0.tar.gz".to_string()),
        source_hash: Some("d6b3e5b5f9e3a2c0e5a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7".to_string()),
        buck_target: "//dev-util/scons:scons-4.7.0".to_string(),
        size: 3_000_000,
        installed_size: 12_000_000,
    }
}

fn bazel_7_1_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "bazel"),
        version: Version::new(7, 1, 0),
        slot: "0".to_string(),
        description: "Fast and scalable build system from Google".to_string(),
        homepage: Some("https://bazel.build/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string()],
        use_flags: vec![
            use_flag("examples", "Install examples", false),
        ],
        dependencies: vec![
            dep("dev-lang", "python"),
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![
            dep_build("dev-java", "openjdk"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/bazelbuild/bazel/releases/download/7.1.0/bazel-7.1.0-dist.zip".to_string()),
        source_hash: Some("e7b8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8".to_string()),
        buck_target: "//dev-util/bazel:bazel-7.1.0".to_string(),
        size: 350_000_000,
        installed_size: 400_000_000,
    }
}

fn buck2_2024_01() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "buck2"),
        version: Version::parse("2024.1.0").unwrap(),
        slot: "0".to_string(),
        description: "Fast build system from Meta".to_string(),
        homepage: Some("https://buck2.build/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string()],
        use_flags: vec![],
        dependencies: vec![
            dep("dev-lang", "python"),
        ],
        build_dependencies: vec![
            dep_build("dev-lang", "rust"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/facebook/buck2/releases/download/2024-01-01/buck2.tar.gz".to_string()),
        source_hash: Some("f8b9c0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9".to_string()),
        buck_target: "//dev-util/buck2:buck2-2024.01".to_string(),
        size: 50_000_000,
        installed_size: 150_000_000,
    }
}
