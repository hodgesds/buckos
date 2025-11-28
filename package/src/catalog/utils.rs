//! Common utility packages
//!
//! This module defines common system utilities including:
//! - coreutils
//! - util-linux
//! - findutils
//! - grep, sed, awk, etc.

use super::{dep, dep_build, dep_runtime, use_flag};
use crate::types::{PackageId, PackageInfo};
use semver::Version;

/// Get all utility packages
pub fn get_packages() -> Vec<PackageInfo> {
    vec![
        // Coreutils
        coreutils_9_4(),
        coreutils_9_5(),
        // Util-linux
        util_linux_2_39(),
        util_linux_2_40(),
        // Findutils
        findutils_4_9_0(),
        // Grep
        grep_3_11(),
        // Sed
        sed_4_9(),
        // Gawk
        gawk_5_3_0(),
        // Diffutils
        diffutils_3_10(),
        // Patch
        patch_2_7_6(),
        // File
        file_5_45(),
        // Which
        which_2_21(),
        // Less
        less_643(),
        // Gzip
        gzip_1_13(),
        // Tar
        tar_1_35(),
        // Cpio
        cpio_2_15(),
        // Procps
        procps_4_0_4(),
        // Psmisc
        psmisc_23_6(),
        // Shadow
        shadow_4_14_6(),
        // Sudo
        sudo_1_9_15(),
        // E2fsprogs
        e2fsprogs_1_47_0(),
    ]
}

fn coreutils_9_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "coreutils"),
        version: Version::new(9, 4, 0),
        slot: "0".to_string(),
        description: "GNU core utilities: mv, cp, ls, etc.".to_string(),
        homepage: Some("https://www.gnu.org/software/coreutils/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("caps", "Enable capabilities support", true),
            use_flag("gmp", "Enable GMP support for faster math", false),
            use_flag("hostname", "Install hostname binary", false),
            use_flag("kill", "Install kill binary", false),
            use_flag("multicall", "Build as multicall binary", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("split-usr", "Split /usr paths", false),
            use_flag("static", "Build static binaries", false),
            use_flag("xattr", "Enable extended attributes", true),
        ],
        dependencies: vec![
            dep("sys-libs", "acl"),
            dep("sys-libs", "attr"),
            dep("sys-libs", "libcap"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/coreutils/coreutils-9.4.tar.xz".to_string()),
        source_hash: Some(
            "ea613a4cf44612326e917201bbbcdfbd301de21ffc3b59b6e5c07e040b275e52".to_string(),
        ),
        buck_target: "//sys-apps/coreutils:coreutils-9.4".to_string(),
        size: 5_500_000,
        installed_size: 18_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn coreutils_9_5() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "coreutils"),
        version: Version::new(9, 5, 0),
        slot: "0".to_string(),
        description: "GNU core utilities: mv, cp, ls, etc.".to_string(),
        homepage: Some("https://www.gnu.org/software/coreutils/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("caps", "Enable capabilities support", true),
            use_flag("gmp", "Enable GMP support for faster math", false),
            use_flag("hostname", "Install hostname binary", false),
            use_flag("kill", "Install kill binary", false),
            use_flag("multicall", "Build as multicall binary", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("split-usr", "Split /usr paths", false),
            use_flag("static", "Build static binaries", false),
            use_flag("xattr", "Enable extended attributes", true),
        ],
        dependencies: vec![
            dep("sys-libs", "acl"),
            dep("sys-libs", "attr"),
            dep("sys-libs", "libcap"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/coreutils/coreutils-9.5.tar.xz".to_string()),
        source_hash: Some(
            "fa613a4cf44612326e917201bbbcdfbd301de21ffc3b59b6e5c07e040b275e53".to_string(),
        ),
        buck_target: "//sys-apps/coreutils:coreutils-9.5".to_string(),
        size: 5_600_000,
        installed_size: 18_500_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn util_linux_2_39() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "util-linux"),
        version: Version::new(2, 39, 0),
        slot: "0".to_string(),
        description: "Various useful Linux utilities".to_string(),
        homepage: Some("https://www.kernel.org/pub/linux/utils/util-linux/".to_string()),
        license: "GPL-2 GPL-3 LGPL-2.1 BSD-4 MIT public-domain".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("audit", "Enable audit support", false),
            use_flag("caps", "Enable capabilities support", true),
            use_flag("cramfs", "Build mkfs.cramfs and fsck.cramfs", false),
            use_flag("cryptsetup", "Enable cryptsetup support", false),
            use_flag("fdformat", "Build fdformat", false),
            use_flag("hardlink", "Build hardlink", true),
            use_flag("kill", "Install kill binary", true),
            use_flag("logger", "Install logger", true),
            use_flag("magic", "Use magic library for file type detection", true),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("python", "Build Python bindings", false),
            use_flag("readline", "Enable readline support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("slang", "Use S-Lang instead of ncurses", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("su", "Build and install su binary", true),
            use_flag("systemd", "Enable systemd support", true),
            use_flag("tty-helpers", "Install tty helpers", true),
            use_flag("udev", "Enable udev support", true),
            use_flag("unicode", "Enable Unicode support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "ncurses"),
            dep("sys-libs", "readline"),
            dep("sys-libs", "libcap"),
            dep("sys-libs", "pam"),
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
            dep_build("sys-kernel", "linux-headers"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.kernel.org/pub/linux/utils/util-linux/v2.39/util-linux-2.39.tar.xz"
                .to_string(),
        ),
        source_hash: Some(
            "32b744f4f3e6b1c8c1d8f7e7e0a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3".to_string(),
        ),
        buck_target: "//sys-apps/util-linux:util-linux-2.39".to_string(),
        size: 8_000_000,
        installed_size: 25_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn util_linux_2_40() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "util-linux"),
        version: Version::new(2, 40, 0),
        slot: "0".to_string(),
        description: "Various useful Linux utilities".to_string(),
        homepage: Some("https://www.kernel.org/pub/linux/utils/util-linux/".to_string()),
        license: "GPL-2 GPL-3 LGPL-2.1 BSD-4 MIT public-domain".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("audit", "Enable audit support", false),
            use_flag("caps", "Enable capabilities support", true),
            use_flag("cramfs", "Build mkfs.cramfs and fsck.cramfs", false),
            use_flag("cryptsetup", "Enable cryptsetup support", false),
            use_flag("fdformat", "Build fdformat", false),
            use_flag("hardlink", "Build hardlink", true),
            use_flag("kill", "Install kill binary", true),
            use_flag("logger", "Install logger", true),
            use_flag("magic", "Use magic library for file type detection", true),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("python", "Build Python bindings", false),
            use_flag("readline", "Enable readline support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("slang", "Use S-Lang instead of ncurses", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("su", "Build and install su binary", true),
            use_flag("systemd", "Enable systemd support", true),
            use_flag("tty-helpers", "Install tty helpers", true),
            use_flag("udev", "Enable udev support", true),
            use_flag("unicode", "Enable Unicode support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "ncurses"),
            dep("sys-libs", "readline"),
            dep("sys-libs", "libcap"),
            dep("sys-libs", "pam"),
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
            dep_build("sys-kernel", "linux-headers"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.kernel.org/pub/linux/utils/util-linux/v2.40/util-linux-2.40.tar.xz"
                .to_string(),
        ),
        source_hash: Some(
            "42b744f4f3e6b1c8c1d8f7e7e0a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a4".to_string(),
        ),
        buck_target: "//sys-apps/util-linux:util-linux-2.40".to_string(),
        size: 8_200_000,
        installed_size: 26_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn findutils_4_9_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "findutils"),
        version: Version::new(4, 9, 0),
        slot: "0".to_string(),
        description: "GNU utilities for finding files".to_string(),
        homepage: Some("https://www.gnu.org/software/findutils/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("static", "Build static binaries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/findutils/findutils-4.9.0.tar.xz".to_string()),
        source_hash: Some(
            "a2bfb8c09d436770edc59f50fa483e785b161a3b7b9d547573cb08065fd462fe".to_string(),
        ),
        buck_target: "//sys-apps/findutils:findutils-4.9.0".to_string(),
        size: 2_000_000,
        installed_size: 4_500_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn grep_3_11() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "grep"),
        version: Version::new(3, 11, 0),
        slot: "0".to_string(),
        description: "GNU regular expression matcher".to_string(),
        homepage: Some("https://www.gnu.org/software/grep/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("pcre", "Enable Perl compatible regex support", true),
            use_flag("static", "Build static binary", false),
        ],
        dependencies: vec![dep("dev-libs", "pcre2")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/grep/grep-3.11.tar.xz".to_string()),
        source_hash: Some(
            "1db2aedde89d0dea42b16d9528f894c8d15dae4e190b59aecc78f5a951276eab".to_string(),
        ),
        buck_target: "//sys-apps/grep:grep-3.11".to_string(),
        size: 1_600_000,
        installed_size: 2_500_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn sed_4_9() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "sed"),
        version: Version::new(4, 9, 0),
        slot: "0".to_string(),
        description: "GNU stream editor".to_string(),
        homepage: Some("https://www.gnu.org/software/sed/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("static", "Build static binary", false),
        ],
        dependencies: vec![dep("sys-libs", "acl")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/sed/sed-4.9.tar.xz".to_string()),
        source_hash: Some(
            "6e226b732e1cd739464ad6862bd1a1aba42d7982922da7a53519631d24975181".to_string(),
        ),
        buck_target: "//sys-apps/sed:sed-4.9".to_string(),
        size: 1_400_000,
        installed_size: 2_200_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn gawk_5_3_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "gawk"),
        version: Version::new(5, 3, 0),
        slot: "0".to_string(),
        description: "GNU awk pattern matching language".to_string(),
        homepage: Some("https://www.gnu.org/software/gawk/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("mpfr", "Enable arbitrary precision math", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("readline", "Enable readline support", true),
        ],
        dependencies: vec![dep("sys-libs", "readline")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/gawk/gawk-5.3.0.tar.xz".to_string()),
        source_hash: Some(
            "ca9c16d3d11d0ff8c69d79dc0b47267e1329a69b39b799895604ed447d3ca90b".to_string(),
        ),
        buck_target: "//sys-apps/gawk:gawk-5.3.0".to_string(),
        size: 3_200_000,
        installed_size: 7_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn diffutils_3_10() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "diffutils"),
        version: Version::new(3, 10, 0),
        slot: "0".to_string(),
        description: "GNU diffutils: diff, cmp, etc.".to_string(),
        homepage: Some("https://www.gnu.org/software/diffutils/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static", "Build static binaries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/diffutils/diffutils-3.10.tar.xz".to_string()),
        source_hash: Some(
            "90e5e93cc724e4ebe12ede80df1634063c7a855f92f8a5fde5e9e0974e353ddd".to_string(),
        ),
        buck_target: "//sys-apps/diffutils:diffutils-3.10".to_string(),
        size: 1_500_000,
        installed_size: 2_800_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn patch_2_7_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-devel", "patch"),
        version: Version::new(2, 7, 6),
        slot: "0".to_string(),
        description: "Utility to apply diffs to files".to_string(),
        homepage: Some("https://www.gnu.org/software/patch/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("static", "Build static binary", false),
            use_flag("xattr", "Enable extended attributes", true),
        ],
        dependencies: vec![dep("sys-libs", "attr")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/patch/patch-2.7.6.tar.xz".to_string()),
        source_hash: Some(
            "ac610bda97abe0d9f6b7c963255a11dcb196c25e337c61f94e4778d632f1d8fd".to_string(),
        ),
        buck_target: "//sys-devel/patch:patch-2.7.6".to_string(),
        size: 800_000,
        installed_size: 1_500_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn file_5_45() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "file"),
        version: Version::new(5, 45, 0),
        slot: "0".to_string(),
        description: "Identify file types".to_string(),
        homepage: Some("https://www.darwinsys.com/file/".to_string()),
        license: "BSD-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("bzip2", "Enable bzip2 support", true),
            use_flag("lzma", "Enable LZMA support", true),
            use_flag("python", "Build Python bindings", false),
            use_flag("seccomp", "Enable seccomp sandbox", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("zlib", "Enable zlib support", true),
            use_flag("zstd", "Enable zstd support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
            dep("app-arch", "bzip2"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "zstd"),
            dep("sys-libs", "libseccomp"),
        ],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://astron.com/pub/file/file-5.45.tar.gz".to_string()),
        source_hash: Some(
            "fc97f51029bb0e2c9f4e3bffefdaf678f0e039ee872b9de5c002a6d09c784c82".to_string(),
        ),
        buck_target: "//sys-apps/file:file-5.45".to_string(),
        size: 1_100_000,
        installed_size: 4_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn which_2_21() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "which"),
        version: Version::new(2, 21, 0),
        slot: "0".to_string(),
        description: "Prints out location of specified executables".to_string(),
        homepage: Some("https://carlowood.github.io/which/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/which/which-2.21.tar.gz".to_string()),
        source_hash: Some(
            "f4a245b94124b377d8b49646bf421f9155d36aa7614b6ebf83705d3ffc76eaad".to_string(),
        ),
        buck_target: "//sys-apps/which:which-2.21".to_string(),
        size: 150_000,
        installed_size: 300_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn less_643() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "less"),
        version: Version::parse("643.0.0").unwrap(),
        slot: "0".to_string(),
        description: "Excellent text pager".to_string(),
        homepage: Some("https://www.greenwoodsoftware.com/less/".to_string()),
        license: "|| ( GPL-3 BSD-2 )".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("pcre", "Enable PCRE regular expressions", false),
            use_flag("unicode", "Enable Unicode support", true),
        ],
        dependencies: vec![dep("sys-libs", "ncurses")],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://www.greenwoodsoftware.com/less/less-643.tar.gz".to_string()),
        source_hash: Some(
            "2911b5432c836fa084c8a2e68f182afe46d2d50c728f0ca11ac6f1694d29d5af".to_string(),
        ),
        buck_target: "//sys-apps/less:less-643".to_string(),
        size: 400_000,
        installed_size: 800_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn gzip_1_13() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "gzip"),
        version: Version::new(1, 13, 0),
        slot: "0".to_string(),
        description: "GNU compression utility".to_string(),
        homepage: Some("https://www.gnu.org/software/gzip/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("pic", "Build as position independent code", false),
            use_flag("static", "Build static binary", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![dep_build("sys-devel", "gcc")],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/gzip/gzip-1.13.tar.xz".to_string()),
        source_hash: Some(
            "7454eb6935db17c6655576c2e1b0fabefd38b4d0936e0f87f48cd062ce91a057".to_string(),
        ),
        buck_target: "//app-arch/gzip:gzip-1.13".to_string(),
        size: 800_000,
        installed_size: 1_500_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn tar_1_35() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "tar"),
        version: Version::new(1, 35, 0),
        slot: "0".to_string(),
        description: "GNU tape archive utility".to_string(),
        homepage: Some("https://www.gnu.org/software/tar/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("xattr", "Enable extended attributes", true),
        ],
        dependencies: vec![dep("sys-libs", "acl"), dep("sys-libs", "attr")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/tar/tar-1.35.tar.xz".to_string()),
        source_hash: Some(
            "4d62ff37342ec7aed748535323930c7cf94acf71c3591882b26a7ea50f3edc16".to_string(),
        ),
        buck_target: "//app-arch/tar:tar-1.35".to_string(),
        size: 2_200_000,
        installed_size: 4_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn cpio_2_15() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-arch", "cpio"),
        version: Version::new(2, 15, 0),
        slot: "0".to_string(),
        description: "Archive utility for cpio archives".to_string(),
        homepage: Some("https://www.gnu.org/software/cpio/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![use_flag("nls", "Enable Native Language Support", true)],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://ftp.gnu.org/gnu/cpio/cpio-2.15.tar.gz".to_string()),
        source_hash: Some(
            "efa50ef983137eefc0a02fdb51509d624b5e3295c980b127f16ea5637c8b6417".to_string(),
        ),
        buck_target: "//app-arch/cpio:cpio-2.15".to_string(),
        size: 1_200_000,
        installed_size: 2_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn procps_4_0_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-process", "procps"),
        version: Version::new(4, 0, 4),
        slot: "0/0".to_string(),
        description: "Standard informational utilities and process-handling tools".to_string(),
        homepage: Some("https://gitlab.com/procps-ng/procps".to_string()),
        license: "GPL-2+ LGPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("kill", "Build kill binary", true),
            use_flag("modern-top", "Build modern top", true),
            use_flag("ncurses", "Enable ncurses support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("systemd", "Enable systemd support", true),
            use_flag("unicode", "Enable Unicode support", true),
        ],
        dependencies: vec![dep("sys-libs", "ncurses")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://gitlab.com/procps-ng/procps/-/archive/v4.0.4/procps-v4.0.4.tar.gz".to_string(),
        ),
        source_hash: Some(
            "a3b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5".to_string(),
        ),
        buck_target: "//sys-process/procps:procps-4.0.4".to_string(),
        size: 1_000_000,
        installed_size: 3_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn psmisc_23_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-process", "psmisc"),
        version: Version::new(23, 6, 0),
        slot: "0".to_string(),
        description: "A set of tools using /proc: fuser, killall, pstree, etc.".to_string(),
        homepage: Some("https://gitlab.com/psmisc/psmisc".to_string()),
        license: "GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("ipv6", "Enable IPv6 support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("selinux", "Enable SELinux support", false),
        ],
        dependencies: vec![dep("sys-libs", "ncurses")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://gitlab.com/psmisc/psmisc/-/archive/v23.6/psmisc-v23.6.tar.gz".to_string(),
        ),
        source_hash: Some(
            "b4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b6".to_string(),
        ),
        buck_target: "//sys-process/psmisc:psmisc-23.6".to_string(),
        size: 400_000,
        installed_size: 1_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn shadow_4_14_6() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "shadow"),
        version: Version::new(4, 14, 6),
        slot: "0".to_string(),
        description: "Utilities for managing user accounts".to_string(),
        homepage: Some("https://github.com/shadow-maint/shadow".to_string()),
        license: "BSD GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("audit", "Enable audit support", false),
            use_flag("bcrypt", "Enable bcrypt password hashing", false),
            use_flag("cracklib", "Enable cracklib support", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("skey", "Enable S/Key support", false),
            use_flag("xattr", "Enable extended attributes", true),
        ],
        dependencies: vec![
            dep("sys-libs", "pam"),
            dep("sys-libs", "acl"),
            dep("sys-libs", "attr"),
            dep("sys-libs", "libxcrypt"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/shadow-maint/shadow/releases/download/4.14.6/shadow-4.14.6.tar.xz"
                .to_string(),
        ),
        source_hash: Some(
            "c5b6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6".to_string(),
        ),
        buck_target: "//sys-apps/shadow:shadow-4.14.6".to_string(),
        size: 1_800_000,
        installed_size: 5_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn sudo_1_9_15() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-admin", "sudo"),
        version: Version::new(1, 9, 15),
        slot: "0".to_string(),
        description: "Execute a command as another user".to_string(),
        homepage: Some("https://www.sudo.ws/".to_string()),
        license: "ISC".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("gcrypt", "Use libgcrypt for crypto", false),
            use_flag("ldap", "Enable LDAP support", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("sendmail", "Enable sendmail support", false),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("sssd", "Enable SSSD support", false),
        ],
        dependencies: vec![dep("sys-libs", "pam"), dep("sys-libs", "zlib")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://www.sudo.ws/dist/sudo-1.9.15p5.tar.gz".to_string()),
        source_hash: Some(
            "d6b7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7".to_string(),
        ),
        buck_target: "//app-admin/sudo:sudo-1.9.15".to_string(),
        size: 4_500_000,
        installed_size: 10_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn e2fsprogs_1_47_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-fs", "e2fsprogs"),
        version: Version::new(1, 47, 0),
        slot: "0".to_string(),
        description: "Standard ext2/ext3/ext4 filesystem utilities".to_string(),
        homepage: Some("https://e2fsprogs.sourceforge.net/".to_string()),
        license: "GPL-2 BSD".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("cron", "Install e2scrub cron job", false),
            use_flag("fuse", "Build fuse2fs", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("split-usr", "Split /usr paths", false),
            use_flag("static-libs", "Build static libraries", false),
        ],
        dependencies: vec![
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![
            dep_runtime("sys-apps", "util-linux"),
        ],
        source_url: Some("https://kernel.org/pub/linux/kernel/people/tytso/e2fsprogs/v1.47.0/e2fsprogs-1.47.0.tar.xz".to_string()),
        source_hash: Some("e7b8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8".to_string()),
        buck_target: "//sys-fs/e2fsprogs:e2fsprogs-1.47.0".to_string(),
        size: 8_000_000,
        installed_size: 20_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}
