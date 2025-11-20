//! Core system library packages
//!
//! This module defines fundamental system libraries including:
//! - glibc (GNU C Library)
//! - musl (lightweight C library)
//! - libgcc
//! - linux-headers

use super::{dep, dep_build, dep_use, use_flag};
use crate::types::{Dependency, PackageId, PackageInfo, UseFlag, VersionSpec};
use semver::Version;

/// Get all core system library packages
pub fn get_packages() -> Vec<PackageInfo> {
    vec![
        // glibc - GNU C Library
        glibc_2_38(),
        glibc_2_39(),
        // musl - lightweight C library
        musl_1_2_4(),
        musl_1_2_5(),
        // libgcc - GCC runtime library
        libgcc_13_2_0(),
        libgcc_14_1_0(),
        // linux-headers - Linux kernel headers
        linux_headers_6_6(),
        linux_headers_6_8(),
        // libxcrypt - Modern password hashing library
        libxcrypt_4_4_36(),
        // timezone-data
        timezone_data_2024a(),
        // ncurses - Terminal handling library
        ncurses_6_4(),
        // readline - Line editing library
        readline_8_2(),
        // libffi - Foreign function interface
        libffi_3_4_6(),
        // gmp - GNU Multiple Precision Arithmetic
        gmp_6_3_0(),
        // mpfr - Multiple Precision Floating-Point
        mpfr_4_2_1(),
        // mpc - Multiple Precision Complex
        mpc_1_3_1(),
        // pam - Pluggable Authentication Modules
        pam_1_6_1(),
        // libcap - POSIX capabilities library
        libcap_2_69(),
        // libseccomp - Seccomp filtering library
        libseccomp_2_5_5(),
        // attr - Extended attributes library
        attr_2_5_2(),
        // acl - Access control list library
        acl_2_3_2(),
    ]
}

fn glibc_2_38() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "glibc"),
        version: Version::new(2, 38, 0),
        slot: "2.2".to_string(),
        description: "GNU C library - the standard C library used by GNU/Linux systems".to_string(),
        homepage: Some("https://www.gnu.org/software/libc/".to_string()),
        license: "LGPL-2.1+ GPL-2+".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("debug", "Build with debug symbols", false),
            use_flag("multilib", "Build multilib support", true),
            use_flag("nscd", "Build name service cache daemon", true),
            use_flag("suid", "Make internal pt_chown helper setuid", true),
            use_flag("systemtap", "Enable SystemTap integration", false),
            use_flag("audit", "Enable audit support", false),
        ],
        dependencies: vec![dep("sys-kernel", "linux-headers")],
        build_dependencies: vec![
            dep_build("sys-devel", "binutils"),
            dep_build("sys-devel", "gcc"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![dep("sys-libs", "timezone-data")],
        source_url: Some("https://ftp.gnu.org/gnu/glibc/glibc-2.38.tar.xz".to_string()),
        source_hash: Some(
            "fb82998998b2b29965467bc1b69d152e9c307d2cf301c9eafb4555b770ef3fd2".to_string(),
        ),
        buck_target: "//sys-libs/glibc:glibc-2.38".to_string(),
        size: 18_500_000,
        installed_size: 45_000_000,
    }
}

fn glibc_2_39() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "glibc"),
        version: Version::new(2, 39, 0),
        slot: "2.2".to_string(),
        description: "GNU C library - the standard C library used by GNU/Linux systems".to_string(),
        homepage: Some("https://www.gnu.org/software/libc/".to_string()),
        license: "LGPL-2.1+ GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("debug", "Build with debug symbols", false),
            use_flag("multilib", "Build multilib support", true),
            use_flag("nscd", "Build name service cache daemon", true),
            use_flag("suid", "Make internal pt_chown helper setuid", true),
            use_flag("systemtap", "Enable SystemTap integration", false),
            use_flag("audit", "Enable audit support", false),
        ],
        dependencies: vec![dep("sys-kernel", "linux-headers")],
        build_dependencies: vec![
            dep_build("sys-devel", "binutils"),
            dep_build("sys-devel", "gcc"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![dep("sys-libs", "timezone-data")],
        source_url: Some("https://ftp.gnu.org/gnu/glibc/glibc-2.39.tar.xz".to_string()),
        source_hash: Some(
            "f77bd47cf8e0cb395eb8fdf77dda3fb8cf9c1d57c15bdf3d3d36c53ceb0d9a2d".to_string(),
        ),
        buck_target: "//sys-libs/glibc:glibc-2.39".to_string(),
        size: 18_800_000,
        installed_size: 46_000_000,
    }
}

fn musl_1_2_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "musl"),
        version: Version::new(1, 2, 4),
        slot: "0".to_string(),
        description: "Lightweight, fast and simple C library for Linux".to_string(),
        homepage: Some("https://musl.libc.org/".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string()],
        use_flags: vec![use_flag("static-libs", "Build static libraries", true)],
        dependencies: vec![dep("sys-kernel", "linux-headers")],
        build_dependencies: vec![
            dep_build("sys-devel", "binutils"),
            dep_build("sys-devel", "gcc"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://musl.libc.org/releases/musl-1.2.4.tar.gz".to_string()),
        source_hash: Some(
            "7a35eae33d5372a7c0da1188de798726f68825513b7ae3ebe97aaaa52114f039".to_string(),
        ),
        buck_target: "//sys-libs/musl:musl-1.2.4".to_string(),
        size: 1_100_000,
        installed_size: 3_500_000,
    }
}

fn musl_1_2_5() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "musl"),
        version: Version::new(1, 2, 5),
        slot: "0".to_string(),
        description: "Lightweight, fast and simple C library for Linux".to_string(),
        homepage: Some("https://musl.libc.org/".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string()],
        use_flags: vec![use_flag("static-libs", "Build static libraries", true)],
        dependencies: vec![dep("sys-kernel", "linux-headers")],
        build_dependencies: vec![
            dep_build("sys-devel", "binutils"),
            dep_build("sys-devel", "gcc"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://musl.libc.org/releases/musl-1.2.5.tar.gz".to_string()),
        source_hash: Some(
            "a9a118bbe84d8764da0ea0d28b3ab3fae8477fc7e4085d90102b8596fc7c75e4".to_string(),
        ),
        buck_target: "//sys-libs/musl:musl-1.2.5".to_string(),
        size: 1_150_000,
        installed_size: 3_600_000,
    }
}

fn libgcc_13_2_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "libgcc"),
        version: Version::new(13, 2, 0),
        slot: "0".to_string(),
        description: "GCC runtime library".to_string(),
        homepage: Some("https://gcc.gnu.org/".to_string()),
        license: "GPL-3+ LGPL-3+".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![],
        dependencies: vec![],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: None, // Built as part of GCC
        source_hash: None,
        buck_target: "//sys-libs/libgcc:libgcc-13.2.0".to_string(),
        size: 150_000,
        installed_size: 500_000,
    }
}

fn libgcc_14_1_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "libgcc"),
        version: Version::new(14, 1, 0),
        slot: "0".to_string(),
        description: "GCC runtime library".to_string(),
        homepage: Some("https://gcc.gnu.org/".to_string()),
        license: "GPL-3+ LGPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "~x86".to_string()],
        use_flags: vec![],
        dependencies: vec![],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: None, // Built as part of GCC
        source_hash: None,
        buck_target: "//sys-libs/libgcc:libgcc-14.1.0".to_string(),
        size: 160_000,
        installed_size: 520_000,
    }
}

fn linux_headers_6_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-kernel", "linux-headers"),
        version: Version::new(6, 6, 0),
        slot: "0".to_string(),
        description: "Linux kernel headers for userspace".to_string(),
        homepage: Some("https://www.kernel.org/".to_string()),
        license: "GPL-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![],
        dependencies: vec![],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.kernel.org/pub/linux/kernel/v6.x/linux-6.6.tar.xz".to_string(),
        ),
        source_hash: Some(
            "d55f5a834fd6cd2a70e3c5a7e2e5e0f7cc1d7a9be7b6c17f6daf82e4e06a5f3a".to_string(),
        ),
        buck_target: "//sys-kernel/linux-headers:linux-headers-6.6".to_string(),
        size: 1_500_000,
        installed_size: 8_000_000,
    }
}

fn linux_headers_6_8() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-kernel", "linux-headers"),
        version: Version::new(6, 8, 0),
        slot: "0".to_string(),
        description: "Linux kernel headers for userspace".to_string(),
        homepage: Some("https://www.kernel.org/".to_string()),
        license: "GPL-2".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![],
        dependencies: vec![],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.kernel.org/pub/linux/kernel/v6.x/linux-6.8.tar.xz".to_string(),
        ),
        source_hash: Some(
            "c969dea4e8bb6be991bbf7c010ba0e0a5643a3a8d8fb0a2b9e1c8d9c0e7f6b5c".to_string(),
        ),
        buck_target: "//sys-kernel/linux-headers:linux-headers-6.8".to_string(),
        size: 1_550_000,
        installed_size: 8_200_000,
    }
}

fn libxcrypt_4_4_36() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "libxcrypt"),
        version: Version::new(4, 4, 36),
        slot: "0/1".to_string(),
        description: "Extended crypt library for DES, MD5, Blowfish and others".to_string(),
        homepage: Some("https://github.com/besser82/libxcrypt".to_string()),
        license: "LGPL-2.1+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("compat", "Build legacy API compatibility", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("system", "Use as system crypt library", true),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "autoconf"),
            dep_build("sys-devel", "automake"),
            dep_build("sys-devel", "libtool"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/besser82/libxcrypt/releases/download/v4.4.36/libxcrypt-4.4.36.tar.xz".to_string()),
        source_hash: Some("e5e1f4caee0a01de2aee26e3138807d6d3ca2b8e67287966d1fefd65e1fd8943".to_string()),
        buck_target: "//sys-libs/libxcrypt:libxcrypt-4.4.36".to_string(),
        size: 400_000,
        installed_size: 1_200_000,
    }
}

fn timezone_data_2024a() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "timezone-data"),
        version: Version::parse("2024.1.0").unwrap(),
        slot: "0".to_string(),
        description: "Timezone data and utilities".to_string(),
        homepage: Some("https://www.iana.org/time-zones".to_string()),
        license: "public-domain".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("leaps", "Install leap second data", false)],
        dependencies: vec![],
        build_dependencies: vec![],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://data.iana.org/time-zones/releases/tzdata2024a.tar.gz".to_string(),
        ),
        source_hash: Some(
            "0d0434459acbd2059a7a8da1f3304a84a86591f6ed69c6248fffa502b6edffe3".to_string(),
        ),
        buck_target: "//sys-libs/timezone-data:timezone-data-2024a".to_string(),
        size: 450_000,
        installed_size: 2_000_000,
    }
}

fn ncurses_6_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "ncurses"),
        version: Version::new(6, 4, 0),
        slot: "0/6".to_string(),
        description: "Console display library supporting color and cursor control".to_string(),
        homepage: Some("https://invisible-island.net/ncurses/".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("cxx", "Build C++ bindings", true),
            use_flag("debug", "Build with debug information", false),
            use_flag("gpm", "Add mouse support via sys-libs/gpm", false),
            use_flag("minimal", "Build minimal ncurses", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("tinfo", "Build separate tinfo library", true),
            use_flag("unicode", "Build wide character support", true),
        ],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/ncurses/ncurses-6.4.tar.gz".to_string()),
        source_hash: Some(
            "6931283d9ac87c5073f30b6290c4c75f21632bb4fc3603ac8100812bed248159".to_string(),
        ),
        buck_target: "//sys-libs/ncurses:ncurses-6.4".to_string(),
        size: 3_500_000,
        installed_size: 8_000_000,
    }
}

fn readline_8_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "readline"),
        version: Version::new(8, 2, 0),
        slot: "0/8".to_string(),
        description: "Library for editing command lines as they are typed".to_string(),
        homepage: Some("https://tiswww.case.edu/php/chet/readline/rltop.html".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("static-libs", "Build static libraries", false),
            use_flag("unicode", "Enable Unicode support", true),
        ],
        dependencies: vec![dep("sys-libs", "ncurses")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/readline/readline-8.2.tar.gz".to_string()),
        source_hash: Some(
            "3feb7171f16a84ee82ca18a36d7b9be109a52c04f492a053331571d0f8e7d8f4".to_string(),
        ),
        buck_target: "//sys-libs/readline:readline-8.2".to_string(),
        size: 3_000_000,
        installed_size: 5_000_000,
    }
}

fn libffi_3_4_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-libs", "libffi"),
        version: Version::new(3, 4, 6),
        slot: "0/8".to_string(),
        description: "Portable foreign function interface library".to_string(),
        homepage: Some("https://sourceware.org/libffi/".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("debug", "Build with debug information", false),
            use_flag("pax-kernel", "Enable PaX kernel support", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/libffi/libffi/releases/download/v3.4.6/libffi-3.4.6.tar.gz"
                .to_string(),
        ),
        source_hash: Some(
            "b0dea9df23c863a7a50e825440f3ebffabd65df1497108e5d437747843895a4e".to_string(),
        ),
        buck_target: "//dev-libs/libffi:libffi-3.4.6".to_string(),
        size: 1_400_000,
        installed_size: 2_500_000,
    }
}

fn gmp_6_3_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-libs", "gmp"),
        version: Version::new(6, 3, 0),
        slot: "0/10".to_string(),
        description: "Library for arbitrary precision arithmetic".to_string(),
        homepage: Some("https://gmplib.org/".to_string()),
        license: "LGPL-3+ GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("asm", "Enable assembly optimizations", true),
            use_flag("cxx", "Build C++ support", true),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc"), dep_build("sys-devel", "m4")],
        runtime_dependencies: vec![],
        source_url: Some("https://gmplib.org/download/gmp/gmp-6.3.0.tar.xz".to_string()),
        source_hash: Some(
            "a3c2b80201b89e68616f4ad30bc66aee4927c3ce50e33929ca819d5c43538898".to_string(),
        ),
        buck_target: "//dev-libs/gmp:gmp-6.3.0".to_string(),
        size: 2_100_000,
        installed_size: 4_500_000,
    }
}

fn mpfr_4_2_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-libs", "mpfr"),
        version: Version::new(4, 2, 1),
        slot: "0/6".to_string(),
        description: "Library for multiple-precision floating-point computations".to_string(),
        homepage: Some("https://www.mpfr.org/".to_string()),
        license: "LGPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("static-libs", "Build static libraries", false)],
        dependencies: vec![dep("dev-libs", "gmp")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://www.mpfr.org/mpfr-4.2.1/mpfr-4.2.1.tar.xz".to_string()),
        source_hash: Some(
            "277807353a6726978996945af13e52829e3abd7a9a5b7fb2793894e18f1fcbb2".to_string(),
        ),
        buck_target: "//dev-libs/mpfr:mpfr-4.2.1".to_string(),
        size: 1_500_000,
        installed_size: 3_000_000,
    }
}

fn mpc_1_3_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-libs", "mpc"),
        version: Version::new(1, 3, 1),
        slot: "0/3".to_string(),
        description: "Library for the arithmetic of complex numbers with high precision"
            .to_string(),
        homepage: Some("https://www.multiprecision.org/mpc/".to_string()),
        license: "LGPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("static-libs", "Build static libraries", false)],
        dependencies: vec![dep("dev-libs", "gmp"), dep("dev-libs", "mpfr")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/mpc/mpc-1.3.1.tar.gz".to_string()),
        source_hash: Some(
            "ab642492f5cf882b74aa0cb730cd410a81edcdbec895183ce930e706c1c759b8".to_string(),
        ),
        buck_target: "//dev-libs/mpc:mpc-1.3.1".to_string(),
        size: 750_000,
        installed_size: 1_500_000,
    }
}

fn pam_1_6_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "pam"),
        version: Version::new(1, 6, 1),
        slot: "0".to_string(),
        description: "Pluggable Authentication Modules for Linux".to_string(),
        homepage: Some("https://github.com/linux-pam/linux-pam".to_string()),
        license: "|| ( BSD GPL-2 )".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("audit", "Enable audit support", false),
            use_flag("berkdb", "Use Berkeley DB for credential storage", false),
            use_flag("debug", "Build with debug symbols", false),
            use_flag("nis", "Enable NIS support", false),
            use_flag("selinux", "Add SELinux support", false),
            use_flag("systemd", "Build systemd module", true),
        ],
        dependencies: vec![
            dep("sys-libs", "libxcrypt"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "flex"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/linux-pam/linux-pam/releases/download/v1.6.1/Linux-PAM-1.6.1.tar.xz".to_string()),
        source_hash: Some("f8923c740159052d719dbfc2a2f81942d68dd34fcaf61c706a02c9b80feeef8e".to_string()),
        buck_target: "//sys-libs/pam:pam-1.6.1".to_string(),
        size: 1_200_000,
        installed_size: 3_500_000,
    }
}

fn libcap_2_69() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "libcap"),
        version: Version::new(2, 69, 0),
        slot: "0".to_string(),
        description: "POSIX 1003.1e capabilities".to_string(),
        homepage: Some("https://sites.google.com/site/fullycapable/".to_string()),
        license: "|| ( GPL-2 BSD )".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("pam", "Build PAM module", true),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![dep("sys-libs", "attr")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://kernel.org/pub/linux/libs/security/linux-privs/libcap2/libcap-2.69.tar.xz"
                .to_string(),
        ),
        source_hash: Some(
            "f311f8f3dad84699d0566d1d6f7ec943a9298b28f714cae3c931dfd57492d7eb".to_string(),
        ),
        buck_target: "//sys-libs/libcap:libcap-2.69".to_string(),
        size: 120_000,
        installed_size: 400_000,
    }
}

fn libseccomp_2_5_5() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "libseccomp"),
        version: Version::new(2, 5, 5),
        slot: "0".to_string(),
        description: "High level interface to Linux seccomp filter".to_string(),
        homepage: Some("https://github.com/seccomp/libseccomp".to_string()),
        license: "LGPL-2.1".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("python", "Build Python bindings", false),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/seccomp/libseccomp/releases/download/v2.5.5/libseccomp-2.5.5.tar.gz".to_string()),
        source_hash: Some("248a2c8a4d9b9858aa6baf52712c34afefcf9c9e94b76dce02c1c9aa25fb3375".to_string()),
        buck_target: "//sys-libs/libseccomp:libseccomp-2.5.5".to_string(),
        size: 600_000,
        installed_size: 1_500_000,
    }
}

fn attr_2_5_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "attr"),
        version: Version::new(2, 5, 2),
        slot: "0".to_string(),
        description: "Extended attributes tools and libraries".to_string(),
        homepage: Some("https://savannah.nongnu.org/projects/attr".to_string()),
        license: "LGPL-2.1+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
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
            "https://download.savannah.gnu.org/releases/attr/attr-2.5.2.tar.gz".to_string(),
        ),
        source_hash: Some(
            "39bf67452fa41d0948c2197601053f48b3d78a029389734332a6309a680c6c87".to_string(),
        ),
        buck_target: "//sys-libs/attr:attr-2.5.2".to_string(),
        size: 350_000,
        installed_size: 900_000,
    }
}

fn acl_2_3_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-libs", "acl"),
        version: Version::new(2, 3, 2),
        slot: "0".to_string(),
        description: "Access control list utilities, libraries and headers".to_string(),
        homepage: Some("https://savannah.nongnu.org/projects/acl".to_string()),
        license: "LGPL-2.1+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![dep("sys-libs", "attr")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://download.savannah.gnu.org/releases/acl/acl-2.3.2.tar.xz".to_string(),
        ),
        source_hash: Some(
            "5f2bdbad629707aa7d85c623f994aa8a1d2dec55a73de5205bac0bf6058a2f7c".to_string(),
        ),
        buck_target: "//sys-libs/acl:acl-2.3.2".to_string(),
        size: 400_000,
        installed_size: 1_000_000,
    }
}
