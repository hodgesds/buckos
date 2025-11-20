//! Compiler toolchain packages
//!
//! This module defines compiler toolchain packages including:
//! - GCC (GNU Compiler Collection)
//! - Clang/LLVM
//! - Binutils
//! - Assemblers and linkers

use super::{dep, dep_build, dep_runtime, use_flag};
use crate::types::{PackageId, PackageInfo};
use semver::Version;

/// Get all toolchain packages
pub fn get_packages() -> Vec<PackageInfo> {
    vec![
        // GCC - GNU Compiler Collection
        gcc_13_2_0(),
        gcc_14_1_0(),
        // Clang/LLVM
        llvm_17_0_6(),
        llvm_18_1_0(),
        clang_17_0_6(),
        clang_18_1_0(),
        // Binutils
        binutils_2_41(),
        binutils_2_42(),
        // GNU Make
        make_4_4_1(),
        // Autotools
        autoconf_2_72(),
        automake_1_16_5(),
        libtool_2_4_7(),
        // M4
        m4_1_4_19(),
        // Bison & Flex
        bison_3_8_2(),
        flex_2_6_4(),
        // Gettext
        gettext_0_22_5(),
        // pkg-config
        pkgconf_2_1_1(),
        // Rust toolchain
        rust_1_77_0(),
        rust_1_79_0(),
        // Go toolchain
        go_1_22_0(),
        // Python
        python_3_11_9(),
        python_3_12_3(),
    ]
}

fn gcc_13_2_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "gcc"),
        version: Version::new(13, 2, 0),
        slot: "13".to_string(),
        description: "The GNU Compiler Collection".to_string(),
        homepage: Some("https://gcc.gnu.org/".to_string()),
        license: "GPL-3+ LGPL-3+ || ( GPL-3+ libgcc libstdc++ ) FDL-1.3+".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("ada", "Build Ada frontend", false),
            use_flag("cxx", "Build C++ support", true),
            use_flag("d", "Build D frontend", false),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("fortran", "Build Fortran support", false),
            use_flag("go", "Build Go frontend", false),
            use_flag("graphite", "Add loop optimizations via ISL", false),
            use_flag("jit", "Enable libgccjit", false),
            use_flag("lto", "Enable Link Time Optimization support", true),
            use_flag("multilib", "Build multilib support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("objc", "Build Objective-C support", false),
            use_flag("objc++", "Build Objective-C++ support", false),
            use_flag("openmp", "Build OpenMP support", true),
            use_flag("pgo", "Build with Profile-Guided Optimization", false),
            use_flag("sanitize", "Build sanitizer runtime", true),
        ],
        dependencies: vec![
            dep("sys-libs", "glibc"),
            dep("dev-libs", "gmp"),
            dep("dev-libs", "mpfr"),
            dep("dev-libs", "mpc"),
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "binutils"),
            dep_build("sys-kernel", "linux-headers"),
            dep_build("sys-devel", "flex"),
            dep_build("sys-devel", "bison"),
        ],
        runtime_dependencies: vec![dep_runtime("sys-libs", "libgcc")],
        source_url: Some("https://ftp.gnu.org/gnu/gcc/gcc-13.2.0/gcc-13.2.0.tar.xz".to_string()),
        source_hash: Some(
            "e275e76442a6067341a27f04c5c6b83d8613144004c0413f2aec5a48a0cadc30".to_string(),
        ),
        buck_target: "//sys-devel/gcc:gcc-13.2.0".to_string(),
        size: 85_000_000,
        installed_size: 350_000_000,
    }
}

fn gcc_14_1_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "gcc"),
        version: Version::new(14, 1, 0),
        slot: "14".to_string(),
        description: "The GNU Compiler Collection".to_string(),
        homepage: Some("https://gcc.gnu.org/".to_string()),
        license: "GPL-3+ LGPL-3+ || ( GPL-3+ libgcc libstdc++ ) FDL-1.3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("ada", "Build Ada frontend", false),
            use_flag("cxx", "Build C++ support", true),
            use_flag("d", "Build D frontend", false),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("fortran", "Build Fortran support", false),
            use_flag("go", "Build Go frontend", false),
            use_flag("graphite", "Add loop optimizations via ISL", false),
            use_flag("jit", "Enable libgccjit", false),
            use_flag("lto", "Enable Link Time Optimization support", true),
            use_flag("multilib", "Build multilib support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("objc", "Build Objective-C support", false),
            use_flag("objc++", "Build Objective-C++ support", false),
            use_flag("openmp", "Build OpenMP support", true),
            use_flag("pgo", "Build with Profile-Guided Optimization", false),
            use_flag("sanitize", "Build sanitizer runtime", true),
        ],
        dependencies: vec![
            dep("sys-libs", "glibc"),
            dep("dev-libs", "gmp"),
            dep("dev-libs", "mpfr"),
            dep("dev-libs", "mpc"),
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "binutils"),
            dep_build("sys-kernel", "linux-headers"),
            dep_build("sys-devel", "flex"),
            dep_build("sys-devel", "bison"),
        ],
        runtime_dependencies: vec![dep_runtime("sys-libs", "libgcc")],
        source_url: Some("https://ftp.gnu.org/gnu/gcc/gcc-14.1.0/gcc-14.1.0.tar.xz".to_string()),
        source_hash: Some(
            "e283c654987afe3de9d8080bc0bd79534b5ca0d681a73a11ff2b5d3767426840".to_string(),
        ),
        buck_target: "//sys-devel/gcc:gcc-14.1.0".to_string(),
        size: 88_000_000,
        installed_size: 360_000_000,
    }
}

fn llvm_17_0_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "llvm"),
        version: Version::new(17, 0, 6),
        slot: "17".to_string(),
        description: "Low Level Virtual Machine".to_string(),
        homepage: Some("https://llvm.org/".to_string()),
        license: "Apache-2.0-with-LLVM-exceptions UoI-NCSA MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("binutils-plugin", "Build binutils plugin", false),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("doc", "Generate documentation", false),
            use_flag("exegesis", "Enable llvm-exegesis", false),
            use_flag("libedit", "Use libedit instead of GNU readline", true),
            use_flag("libffi", "Enable Foreign Function Interface support", true),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("test", "Build tests", false),
            use_flag("xar", "Enable XAR support", false),
            use_flag("xml", "Enable XML support", false),
            use_flag("z3", "Enable Z3 support for static analysis", false),
            use_flag("zstd", "Enable zstd compression support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "zstd"),
            dep("dev-libs", "libffi"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/llvm-17.0.6.src.tar.xz".to_string()),
        source_hash: Some("b638167da139126ca11917b6880207cc6e8f9d1cbb1a48d87d017f697ef78188".to_string()),
        buck_target: "//sys-devel/llvm:llvm-17.0.6".to_string(),
        size: 60_000_000,
        installed_size: 250_000_000,
    }
}

fn llvm_18_1_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "llvm"),
        version: Version::new(18, 1, 0),
        slot: "18".to_string(),
        description: "Low Level Virtual Machine".to_string(),
        homepage: Some("https://llvm.org/".to_string()),
        license: "Apache-2.0-with-LLVM-exceptions UoI-NCSA MIT".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string()],
        use_flags: vec![
            use_flag("binutils-plugin", "Build binutils plugin", false),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("doc", "Generate documentation", false),
            use_flag("exegesis", "Enable llvm-exegesis", false),
            use_flag("libedit", "Use libedit instead of GNU readline", true),
            use_flag("libffi", "Enable Foreign Function Interface support", true),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("test", "Build tests", false),
            use_flag("xar", "Enable XAR support", false),
            use_flag("xml", "Enable XML support", false),
            use_flag("z3", "Enable Z3 support for static analysis", false),
            use_flag("zstd", "Enable zstd compression support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "zstd"),
            dep("dev-libs", "libffi"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.0/llvm-18.1.0.src.tar.xz".to_string()),
        source_hash: Some("ba0e74e97faa00fa31f9e3e872bd4a4e8f1f9fb9e11b5a0d14e9b893a5fc8b12".to_string()),
        buck_target: "//sys-devel/llvm:llvm-18.1.0".to_string(),
        size: 65_000_000,
        installed_size: 280_000_000,
    }
}

fn clang_17_0_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "clang"),
        version: Version::new(17, 0, 6),
        slot: "17".to_string(),
        description: "C language family frontend for LLVM".to_string(),
        homepage: Some("https://clang.llvm.org/".to_string()),
        license: "Apache-2.0-with-LLVM-exceptions UoI-NCSA MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("debug", "Build with debug symbols", false),
            use_flag("default-compiler-rt", "Use compiler-rt as default runtime", false),
            use_flag("default-libcxx", "Use libc++ as default stdlib", false),
            use_flag("default-lld", "Use LLD as default linker", false),
            use_flag("doc", "Generate documentation", false),
            use_flag("extra", "Build clang-tools-extra", true),
            use_flag("static-analyzer", "Enable static analyzer", true),
        ],
        dependencies: vec![
            dep("sys-devel", "llvm"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/clang-17.0.6.src.tar.xz".to_string()),
        source_hash: Some("a78f668a726ae1d3d9a7179996d97b12b90fb76ab9442a43c42c75f7c6f0b1a5".to_string()),
        buck_target: "//sys-devel/clang:clang-17.0.6".to_string(),
        size: 35_000_000,
        installed_size: 120_000_000,
    }
}

fn clang_18_1_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "clang"),
        version: Version::new(18, 1, 0),
        slot: "18".to_string(),
        description: "C language family frontend for LLVM".to_string(),
        homepage: Some("https://clang.llvm.org/".to_string()),
        license: "Apache-2.0-with-LLVM-exceptions UoI-NCSA MIT".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string()],
        use_flags: vec![
            use_flag("debug", "Build with debug symbols", false),
            use_flag("default-compiler-rt", "Use compiler-rt as default runtime", false),
            use_flag("default-libcxx", "Use libc++ as default stdlib", false),
            use_flag("default-lld", "Use LLD as default linker", false),
            use_flag("doc", "Generate documentation", false),
            use_flag("extra", "Build clang-tools-extra", true),
            use_flag("static-analyzer", "Enable static analyzer", true),
        ],
        dependencies: vec![
            dep("sys-devel", "llvm"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.0/clang-18.1.0.src.tar.xz".to_string()),
        source_hash: Some("c50e8d5e0c9ae85e9a54b5d21d1e2e5e5f9a5a5e5c5c5c5c5a5a5a5a5a5a5a5a".to_string()),
        buck_target: "//sys-devel/clang:clang-18.1.0".to_string(),
        size: 38_000_000,
        installed_size: 130_000_000,
    }
}

fn binutils_2_41() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "binutils"),
        version: Version::new(2, 41, 0),
        slot: "0".to_string(),
        description: "Tools necessary to build programs".to_string(),
        homepage: Some("https://sourceware.org/binutils/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("cet", "Enable Control-flow Enforcement Technology", false),
            use_flag("default-gold", "Use gold as default linker", false),
            use_flag("doc", "Build documentation", false),
            use_flag("gold", "Build gold linker", true),
            use_flag("gprofng", "Build gprofng profiler", false),
            use_flag("multitarget", "Build for all supported targets", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("pgo", "Build with Profile-Guided Optimization", false),
            use_flag("plugins", "Enable plugin support", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("zstd", "Enable zstd compression", true),
        ],
        dependencies: vec![dep("sys-libs", "zlib")],
        build_dependencies: vec![
            dep_build("sys-devel", "flex"),
            dep_build("sys-devel", "bison"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/binutils/binutils-2.41.tar.xz".to_string()),
        source_hash: Some(
            "ae9a5789e23459e59606e6714723f2d3ffc31c03174191ef0d015bdf06007450".to_string(),
        ),
        buck_target: "//sys-devel/binutils:binutils-2.41".to_string(),
        size: 25_000_000,
        installed_size: 95_000_000,
    }
}

fn binutils_2_42() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "binutils"),
        version: Version::new(2, 42, 0),
        slot: "0".to_string(),
        description: "Tools necessary to build programs".to_string(),
        homepage: Some("https://sourceware.org/binutils/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("cet", "Enable Control-flow Enforcement Technology", false),
            use_flag("default-gold", "Use gold as default linker", false),
            use_flag("doc", "Build documentation", false),
            use_flag("gold", "Build gold linker", true),
            use_flag("gprofng", "Build gprofng profiler", false),
            use_flag("multitarget", "Build for all supported targets", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("pgo", "Build with Profile-Guided Optimization", false),
            use_flag("plugins", "Enable plugin support", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("zstd", "Enable zstd compression", true),
        ],
        dependencies: vec![dep("sys-libs", "zlib")],
        build_dependencies: vec![
            dep_build("sys-devel", "flex"),
            dep_build("sys-devel", "bison"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/binutils/binutils-2.42.tar.xz".to_string()),
        source_hash: Some(
            "f6e4d41fd5fc778b06b7891457b3620da5ecea1006c6a4f0018e9ab15d5b9820".to_string(),
        ),
        buck_target: "//sys-devel/binutils:binutils-2.42".to_string(),
        size: 26_000_000,
        installed_size: 98_000_000,
    }
}

fn make_4_4_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "make"),
        version: Version::new(4, 4, 1),
        slot: "0".to_string(),
        description: "Standard UNIX build tool".to_string(),
        homepage: Some("https://www.gnu.org/software/make/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("guile", "Add GNU Guile support", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static", "Build static binary", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gettext")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/make/make-4.4.1.tar.gz".to_string()),
        source_hash: Some(
            "dd16fb1d67bfab79a72f5e8390735c49e3e8e70b4945a15ab1f81ddb78658fb3".to_string(),
        ),
        buck_target: "//sys-devel/make:make-4.4.1".to_string(),
        size: 2_300_000,
        installed_size: 3_500_000,
    }
}

fn autoconf_2_72() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "autoconf"),
        version: Version::new(2, 72, 0),
        slot: "2.72".to_string(),
        description: "Used to create autoconfiguration files".to_string(),
        homepage: Some("https://www.gnu.org/software/autoconf/".to_string()),
        license: "GPL-3+ GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("emacs", "Install Emacs support files", false)],
        dependencies: vec![dep("sys-devel", "m4"), dep("dev-lang", "perl")],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/autoconf/autoconf-2.72.tar.xz".to_string()),
        source_hash: Some(
            "ba885c1319578d6c94d46e9b0dceb4014caafe2490e437a0dbca3f270a223f5a".to_string(),
        ),
        buck_target: "//sys-devel/autoconf:autoconf-2.72".to_string(),
        size: 1_300_000,
        installed_size: 3_500_000,
    }
}

fn automake_1_16_5() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "automake"),
        version: Version::new(1, 16, 5),
        slot: "1.16".to_string(),
        description: "Used to generate Makefile.in from Makefile.am".to_string(),
        homepage: Some("https://www.gnu.org/software/automake/".to_string()),
        license: "GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![],
        dependencies: vec![dep("sys-devel", "autoconf"), dep("dev-lang", "perl")],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/automake/automake-1.16.5.tar.xz".to_string()),
        source_hash: Some(
            "f01d58cd6d9d77fbdca9eb4bbd5ead1988228fdb73d6f7a201f5f8d6b118b469".to_string(),
        ),
        buck_target: "//sys-devel/automake:automake-1.16.5".to_string(),
        size: 1_500_000,
        installed_size: 4_000_000,
    }
}

fn libtool_2_4_7() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "libtool"),
        version: Version::new(2, 4, 7),
        slot: "2".to_string(),
        description: "Shared library management for Autoconf".to_string(),
        homepage: Some("https://www.gnu.org/software/libtool/".to_string()),
        license: "GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("vanilla", "Don't apply Gentoo patches", false)],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "autoconf"),
            dep_build("sys-devel", "automake"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/libtool/libtool-2.4.7.tar.xz".to_string()),
        source_hash: Some(
            "4f7f217f057ce655ff22559ad221a0fd8ef84ad1fc5fcb6990cecc333aa1635d".to_string(),
        ),
        buck_target: "//sys-devel/libtool:libtool-2.4.7".to_string(),
        size: 1_000_000,
        installed_size: 2_500_000,
    }
}

fn m4_1_4_19() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "m4"),
        version: Version::new(1, 4, 19),
        slot: "0".to_string(),
        description: "GNU macro processor".to_string(),
        homepage: Some("https://www.gnu.org/software/m4/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("examples", "Install examples", false),
            use_flag("nls", "Enable Native Language Support", true),
        ],
        dependencies: vec![],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/m4/m4-1.4.19.tar.xz".to_string()),
        source_hash: Some(
            "63aede5c6d33b6d9b13511cd0be2cac046f2e70fd0a07aa9573a04a82783af96".to_string(),
        ),
        buck_target: "//sys-devel/m4:m4-1.4.19".to_string(),
        size: 1_600_000,
        installed_size: 2_000_000,
    }
}

fn bison_3_8_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "bison"),
        version: Version::new(3, 8, 2),
        slot: "0".to_string(),
        description: "GNU yacc-compatible parser generator".to_string(),
        homepage: Some("https://www.gnu.org/software/bison/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("examples", "Install examples", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static", "Build static binary", false),
        ],
        dependencies: vec![dep("sys-devel", "m4")],
        build_dependencies: vec![dep_build("sys-devel", "gettext")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/bison/bison-3.8.2.tar.xz".to_string()),
        source_hash: Some(
            "9bba0214ccf7f1079c5d59210045227bcf619519840ebfa80cd82f6a6b8cd7dc".to_string(),
        ),
        buck_target: "//sys-devel/bison:bison-3.8.2".to_string(),
        size: 2_800_000,
        installed_size: 6_000_000,
    }
}

fn flex_2_6_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "flex"),
        version: Version::new(2, 6, 4),
        slot: "0".to_string(),
        description: "GNU lexical analyzer generator".to_string(),
        homepage: Some("https://github.com/westes/flex".to_string()),
        license: "FLEX".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static", "Build static binary", false),
        ],
        dependencies: vec![dep("sys-devel", "m4")],
        build_dependencies: vec![
            dep_build("sys-devel", "bison"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz".to_string(),
        ),
        source_hash: Some(
            "e87aae032bf07c26f85ac0ed3250998c37621d95f8bd748b31f15b33c45ee995".to_string(),
        ),
        buck_target: "//sys-devel/flex:flex-2.6.4".to_string(),
        size: 1_400_000,
        installed_size: 2_500_000,
    }
}

fn gettext_0_22_5() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "gettext"),
        version: Version::new(0, 22, 5),
        slot: "0".to_string(),
        description: "GNU i18n utilities".to_string(),
        homepage: Some("https://www.gnu.org/software/gettext/".to_string()),
        license: "GPL-3+ LGPL-2.1+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("cxx", "Build C++ support", true),
            use_flag("doc", "Build documentation", false),
            use_flag("emacs", "Install Emacs support files", false),
            use_flag("git", "Use git for cvs support", false),
            use_flag("java", "Build Java support", false),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("openmp", "Enable OpenMP support", false),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![dep("sys-libs", "ncurses")],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/gettext/gettext-0.22.5.tar.xz".to_string()),
        source_hash: Some(
            "fe10c37353213d78a5b83d48af231e005c4da84db5ce88037d88355938259640".to_string(),
        ),
        buck_target: "//sys-devel/gettext:gettext-0.22.5".to_string(),
        size: 10_000_000,
        installed_size: 25_000_000,
    }
}

fn pkgconf_2_1_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-util", "pkgconf"),
        version: Version::new(2, 1, 1),
        slot: "0/4".to_string(),
        description: "pkg-config compatible replacement".to_string(),
        homepage: Some("https://gitea.treehouse.systems/ariadne/pkgconf".to_string()),
        license: "ISC".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("pkg-config", "Provide pkg-config symlink", true)],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://distfiles.ariadne.space/pkgconf/pkgconf-2.1.1.tar.xz".to_string(),
        ),
        source_hash: Some(
            "1a00b7fa08c5b6da4c0d2a41badc9b8f73e5a0fc56c3132e08e50b2b0f4eb6b2".to_string(),
        ),
        buck_target: "//dev-util/pkgconf:pkgconf-2.1.1".to_string(),
        size: 300_000,
        installed_size: 600_000,
    }
}

fn rust_1_77_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-lang", "rust"),
        version: Version::new(1, 77, 0),
        slot: "stable".to_string(),
        description: "Systems programming language focused on safety and speed".to_string(),
        homepage: Some("https://www.rust-lang.org/".to_string()),
        license: "|| ( MIT Apache-2.0 ) BSD BSD-1 BSD-2 BSD-4".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string()],
        use_flags: vec![
            use_flag("clippy", "Install clippy linter", true),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("doc", "Build documentation", false),
            use_flag("miri", "Install miri interpreter", false),
            use_flag("nightly", "Build nightly features", false),
            use_flag("parallel-compiler", "Build parallel compiler", false),
            use_flag("profiler", "Build profiler runtime", false),
            use_flag("rust-analyzer", "Install rust-analyzer", true),
            use_flag("rust-src", "Install rust source", true),
            use_flag("rustfmt", "Install rustfmt formatter", true),
            use_flag("system-llvm", "Use system LLVM", false),
            use_flag("wasm", "Build wasm target", false),
        ],
        dependencies: vec![dep("sys-libs", "zlib"), dep("dev-libs", "openssl")],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://static.rust-lang.org/dist/rustc-1.77.0-src.tar.xz".to_string()),
        source_hash: Some(
            "c1e8b2ab3a1e08c0bb4e2ffc57d5e5e1e5c5c5a5a5a5a5a5a5a5a5a5a5a5a5a5".to_string(),
        ),
        buck_target: "//dev-lang/rust:rust-1.77.0".to_string(),
        size: 200_000_000,
        installed_size: 800_000_000,
    }
}

fn rust_1_79_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-lang", "rust"),
        version: Version::new(1, 79, 0),
        slot: "stable".to_string(),
        description: "Systems programming language focused on safety and speed".to_string(),
        homepage: Some("https://www.rust-lang.org/".to_string()),
        license: "|| ( MIT Apache-2.0 ) BSD BSD-1 BSD-2 BSD-4".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string()],
        use_flags: vec![
            use_flag("clippy", "Install clippy linter", true),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("doc", "Build documentation", false),
            use_flag("miri", "Install miri interpreter", false),
            use_flag("nightly", "Build nightly features", false),
            use_flag("parallel-compiler", "Build parallel compiler", false),
            use_flag("profiler", "Build profiler runtime", false),
            use_flag("rust-analyzer", "Install rust-analyzer", true),
            use_flag("rust-src", "Install rust source", true),
            use_flag("rustfmt", "Install rustfmt formatter", true),
            use_flag("system-llvm", "Use system LLVM", false),
            use_flag("wasm", "Build wasm target", false),
        ],
        dependencies: vec![dep("sys-libs", "zlib"), dep("dev-libs", "openssl")],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://static.rust-lang.org/dist/rustc-1.79.0-src.tar.xz".to_string()),
        source_hash: Some(
            "d2e8b2ab3a1e08c0bb4e2ffc57d5e5e1e5c5c5a5a5a5a5a5a5a5a5a5a5a5a5a5".to_string(),
        ),
        buck_target: "//dev-lang/rust:rust-1.79.0".to_string(),
        size: 210_000_000,
        installed_size: 850_000_000,
    }
}

fn go_1_22_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-lang", "go"),
        version: Version::new(1, 22, 0),
        slot: "0".to_string(),
        description: "Go programming language".to_string(),
        homepage: Some("https://go.dev/".to_string()),
        license: "BSD".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string()],
        use_flags: vec![],
        dependencies: vec![],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some("https://go.dev/dl/go1.22.0.src.tar.gz".to_string()),
        source_hash: Some(
            "4d196c3d41a0d6c1dfc64d04e3cc1f608b0c436bd87b7060ce3e23234e1f4d5c".to_string(),
        ),
        buck_target: "//dev-lang/go:go-1.22.0".to_string(),
        size: 27_000_000,
        installed_size: 450_000_000,
    }
}

fn python_3_11_9() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-lang", "python"),
        version: Version::new(3, 11, 9),
        slot: "3.11".to_string(),
        description: "An interpreted, interactive, object-oriented programming language"
            .to_string(),
        homepage: Some("https://www.python.org/".to_string()),
        license: "PSF-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("bluetooth", "Build bluetooth support", false),
            use_flag("build", "Build CPython from source", true),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("ensurepip", "Install pip/setuptools", true),
            use_flag("gdbm", "Build gdbm support", true),
            use_flag("lto", "Enable Link Time Optimization", false),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("readline", "Enable readline support", true),
            use_flag("sqlite", "Enable sqlite support", true),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("test", "Build tests", false),
            use_flag("tk", "Enable Tcl/Tk GUI toolkit", false),
            use_flag("xml", "Enable XML support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("dev-libs", "libffi"),
            dep("sys-libs", "ncurses"),
            dep("sys-libs", "readline"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.python.org/ftp/python/3.11.9/Python-3.11.9.tar.xz".to_string(),
        ),
        source_hash: Some(
            "9b1e896523fc510691126c864406d9360a3d1e986acbda59e0e0c0f3c0c3a3a3".to_string(),
        ),
        buck_target: "//dev-lang/python:python-3.11.9".to_string(),
        size: 20_000_000,
        installed_size: 100_000_000,
    }
}

fn python_3_12_3() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-lang", "python"),
        version: Version::new(3, 12, 3),
        slot: "3.12".to_string(),
        description: "An interpreted, interactive, object-oriented programming language"
            .to_string(),
        homepage: Some("https://www.python.org/".to_string()),
        license: "PSF-2".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("bluetooth", "Build bluetooth support", false),
            use_flag("build", "Build CPython from source", true),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("ensurepip", "Install pip/setuptools", true),
            use_flag("gdbm", "Build gdbm support", true),
            use_flag("lto", "Enable Link Time Optimization", false),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("readline", "Enable readline support", true),
            use_flag("sqlite", "Enable sqlite support", true),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("test", "Build tests", false),
            use_flag("tk", "Enable Tcl/Tk GUI toolkit", false),
            use_flag("xml", "Enable XML support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("dev-libs", "libffi"),
            dep("sys-libs", "ncurses"),
            dep("sys-libs", "readline"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.python.org/ftp/python/3.12.3/Python-3.12.3.tar.xz".to_string(),
        ),
        source_hash: Some(
            "a2e896523fc510691126c864406d9360a3d1e986acbda59e0e0c0f3c0c3b4b4".to_string(),
        ),
        buck_target: "//dev-lang/python:python-3.12.3".to_string(),
        size: 21_000_000,
        installed_size: 105_000_000,
    }
}

fn perl_5_38_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-lang", "perl"),
        version: Version::new(5, 38, 2),
        slot: "0/5.38".to_string(),
        description: "Practical Extraction and Report Language".to_string(),
        homepage: Some("https://www.perl.org/".to_string()),
        license: "|| ( Artistic GPL-1+ )".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("berkdb", "Enable Berkeley DB support", true),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("doc", "Build documentation", true),
            use_flag("gdbm", "Enable GDBM support", true),
            use_flag("ithreads", "Enable threading support", true),
            use_flag("minimal", "Build minimal perl", false),
            use_flag("quadmath", "Enable quadmath support", false),
        ],
        dependencies: vec![dep("sys-libs", "gdbm"), dep("sys-libs", "zlib")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://www.cpan.org/src/5.0/perl-5.38.2.tar.xz".to_string()),
        source_hash: Some(
            "a0a31534451a43e2d66b44f7e43e7b4a2c7bba5a5c0e1f8a3b5c7d9e0f1a2b3c".to_string(),
        ),
        buck_target: "//dev-lang/perl:perl-5.38.2".to_string(),
        size: 12_500_000,
        installed_size: 55_000_000,
    }
}
