//! System service packages
//!
//! This module defines system service packages including:
//! - systemd
//! - OpenRC
//! - D-Bus
//! - udev

use crate::types::{PackageId, PackageInfo, Dependency, UseFlag, VersionSpec};
use super::{dep, dep_build, dep_runtime, dep_use, use_flag};
use semver::Version;

/// Get all service packages
pub fn get_packages() -> Vec<PackageInfo> {
    vec![
        // systemd
        systemd_255(),
        systemd_256(),

        // OpenRC
        openrc_0_54(),

        // D-Bus
        dbus_1_14_10(),

        // Udev (standalone)
        eudev_3_2_14(),

        // Polkit
        polkit_124(),

        // Elogind
        elogind_252_9(),

        // Cron
        cronie_1_7_2(),
    ]
}

fn systemd_255() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "systemd"),
        version: Version::new(255, 0, 0),
        slot: "0/2".to_string(),
        description: "System and service manager for Linux".to_string(),
        homepage: Some("https://systemd.io/".to_string()),
        license: "GPL-2 LGPL-2.1 MIT public-domain".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "~x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("apparmor", "Enable AppArmor support", false),
            use_flag("audit", "Enable audit support", false),
            use_flag("boot", "Enable systemd-boot EFI boot manager", true),
            use_flag("cgroup-hybrid", "Enable hybrid cgroup hierarchy", false),
            use_flag("cryptsetup", "Enable cryptsetup support", false),
            use_flag("curl", "Enable curl support", true),
            use_flag("dns-over-tls", "Enable DNS over TLS", true),
            use_flag("elfutils", "Enable elfutils support", true),
            use_flag("fido2", "Enable FIDO2 support", false),
            use_flag("gcrypt", "Enable gcrypt support", true),
            use_flag("gnutls", "Enable GnuTLS support", false),
            use_flag("homed", "Enable systemd-homed", false),
            use_flag("http", "Enable HTTP support", true),
            use_flag("idn", "Enable IDN support", true),
            use_flag("importd", "Enable systemd-importd", false),
            use_flag("kmod", "Enable kmod support", true),
            use_flag("lz4", "Enable LZ4 compression", true),
            use_flag("lzma", "Enable LZMA compression", true),
            use_flag("microhttpd", "Enable microhttpd support", false),
            use_flag("networkd", "Enable systemd-networkd", true),
            use_flag("openssl", "Enable OpenSSL support", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("pcre", "Enable PCRE support", true),
            use_flag("policykit", "Enable PolicyKit support", true),
            use_flag("pwquality", "Enable password quality checking", false),
            use_flag("qrcode", "Enable QR code support", false),
            use_flag("repart", "Enable systemd-repart", false),
            use_flag("resolvconf", "Enable resolvconf compatibility", true),
            use_flag("seccomp", "Enable seccomp support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("split-usr", "Split /usr paths", false),
            use_flag("sysv-utils", "Enable SysV compatibility utilities", true),
            use_flag("test", "Build tests", false),
            use_flag("tpm", "Enable TPM support", false),
            use_flag("ukify", "Enable unified kernel image support", false),
            use_flag("xkb", "Enable XKB support", true),
            use_flag("zstd", "Enable zstd compression", true),
        ],
        dependencies: vec![
            dep("sys-libs", "libcap"),
            dep("sys-libs", "pam"),
            dep("sys-libs", "acl"),
            dep("sys-libs", "libseccomp"),
            dep("dev-libs", "openssl"),
            dep("sys-libs", "zlib"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "zstd"),
            dep("app-arch", "lz4"),
            dep("sys-apps", "dbus"),
            dep("sys-apps", "kmod"),
            dep("dev-libs", "libgcrypt"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
            dep_build("sys-devel", "gettext"),
            dep_build("dev-util", "pkgconf"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/systemd/systemd/archive/v255.tar.gz".to_string()),
        source_hash: Some("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2".to_string()),
        buck_target: "//sys-apps/systemd:systemd-255".to_string(),
        size: 15_000_000,
        installed_size: 45_000_000,
    }
}

fn systemd_256() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "systemd"),
        version: Version::new(256, 0, 0),
        slot: "0/2".to_string(),
        description: "System and service manager for Linux".to_string(),
        homepage: Some("https://systemd.io/".to_string()),
        license: "GPL-2 LGPL-2.1 MIT public-domain".to_string(),
        keywords: vec!["~amd64".to_string(), "~arm64".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("apparmor", "Enable AppArmor support", false),
            use_flag("audit", "Enable audit support", false),
            use_flag("boot", "Enable systemd-boot EFI boot manager", true),
            use_flag("cryptsetup", "Enable cryptsetup support", false),
            use_flag("curl", "Enable curl support", true),
            use_flag("dns-over-tls", "Enable DNS over TLS", true),
            use_flag("elfutils", "Enable elfutils support", true),
            use_flag("fido2", "Enable FIDO2 support", false),
            use_flag("gcrypt", "Enable gcrypt support", true),
            use_flag("homed", "Enable systemd-homed", false),
            use_flag("http", "Enable HTTP support", true),
            use_flag("idn", "Enable IDN support", true),
            use_flag("importd", "Enable systemd-importd", false),
            use_flag("kmod", "Enable kmod support", true),
            use_flag("lz4", "Enable LZ4 compression", true),
            use_flag("lzma", "Enable LZMA compression", true),
            use_flag("networkd", "Enable systemd-networkd", true),
            use_flag("openssl", "Enable OpenSSL support", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("pcre", "Enable PCRE support", true),
            use_flag("policykit", "Enable PolicyKit support", true),
            use_flag("resolvconf", "Enable resolvconf compatibility", true),
            use_flag("seccomp", "Enable seccomp support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("sysv-utils", "Enable SysV compatibility utilities", true),
            use_flag("tpm", "Enable TPM support", false),
            use_flag("xkb", "Enable XKB support", true),
            use_flag("zstd", "Enable zstd compression", true),
        ],
        dependencies: vec![
            dep("sys-libs", "libcap"),
            dep("sys-libs", "pam"),
            dep("sys-libs", "acl"),
            dep("sys-libs", "libseccomp"),
            dep("dev-libs", "openssl"),
            dep("sys-libs", "zlib"),
            dep("app-arch", "xz-utils"),
            dep("app-arch", "zstd"),
            dep("app-arch", "lz4"),
            dep("sys-apps", "dbus"),
            dep("sys-apps", "kmod"),
            dep("dev-libs", "libgcrypt"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
            dep_build("sys-devel", "gettext"),
            dep_build("dev-util", "pkgconf"),
            dep_build("dev-lang", "python"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/systemd/systemd/archive/v256.tar.gz".to_string()),
        source_hash: Some("b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3".to_string()),
        buck_target: "//sys-apps/systemd:systemd-256".to_string(),
        size: 16_000_000,
        installed_size: 48_000_000,
    }
}

fn openrc_0_54() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "openrc"),
        version: Version::new(0, 54, 0),
        slot: "0".to_string(),
        description: "OpenRC manages the services, startup and shutdown of a host".to_string(),
        homepage: Some("https://github.com/OpenRC/openrc".to_string()),
        license: "BSD-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("audit", "Enable audit support", false),
            use_flag("bash", "Install bash completion", true),
            use_flag("debug", "Enable debug mode", false),
            use_flag("ncurses", "Enable ncurses support", false),
            use_flag("netifrc", "Enable net.* init scripts", true),
            use_flag("newnet", "Enable new network stack", false),
            use_flag("pam", "Enable PAM support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("sysv-utils", "Enable SysV compatibility", true),
            use_flag("unicode", "Enable Unicode support", true),
        ],
        dependencies: vec![
            dep("sys-libs", "pam"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![
            dep_runtime("sys-apps", "sysvinit"),
        ],
        source_url: Some("https://github.com/OpenRC/openrc/archive/refs/tags/0.54.tar.gz".to_string()),
        source_hash: Some("c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4".to_string()),
        buck_target: "//sys-apps/openrc:openrc-0.54".to_string(),
        size: 700_000,
        installed_size: 2_500_000,
    }
}

fn dbus_1_14_10() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "dbus"),
        version: Version::new(1, 14, 10),
        slot: "0".to_string(),
        description: "D-Bus message bus".to_string(),
        homepage: Some("https://www.freedesktop.org/wiki/Software/dbus".to_string()),
        license: "|| ( AFL-2.1 GPL-2 )".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("debug", "Enable debug build", false),
            use_flag("doc", "Build documentation", false),
            use_flag("elogind", "Enable elogind integration", false),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("systemd", "Enable systemd integration", true),
            use_flag("test", "Build tests", false),
            use_flag("user-session", "Enable user session bus", true),
            use_flag("X", "Enable X11 support", false),
        ],
        dependencies: vec![
            dep("dev-libs", "expat"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://dbus.freedesktop.org/releases/dbus/dbus-1.14.10.tar.xz".to_string()),
        source_hash: Some("d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5".to_string()),
        buck_target: "//sys-apps/dbus:dbus-1.14.10".to_string(),
        size: 2_000_000,
        installed_size: 6_000_000,
    }
}

fn eudev_3_2_14() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-fs", "eudev"),
        version: Version::new(3, 2, 14),
        slot: "0".to_string(),
        description: "Linux dynamic and persistent device naming support".to_string(),
        homepage: Some("https://github.com/eudev-project/eudev".to_string()),
        license: "LGPL-2.1 MIT GPL-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("hwdb", "Generate hardware database", true),
            use_flag("kmod", "Enable kmod integration", true),
            use_flag("rule-generator", "Enable persistent rules generator", false),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![
            dep("sys-apps", "kmod"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "autoconf"),
            dep_build("sys-devel", "automake"),
            dep_build("sys-devel", "libtool"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/eudev-project/eudev/archive/v3.2.14.tar.gz".to_string()),
        source_hash: Some("e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6".to_string()),
        buck_target: "//sys-fs/eudev:eudev-3.2.14".to_string(),
        size: 600_000,
        installed_size: 3_000_000,
    }
}

fn polkit_124() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-auth", "polkit"),
        version: Version::new(124, 0, 0),
        slot: "0".to_string(),
        description: "Policy framework for controlling privileges".to_string(),
        homepage: Some("https://www.freedesktop.org/wiki/Software/polkit".to_string()),
        license: "LGPL-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("duktape", "Use Duktape JavaScript engine", true),
            use_flag("examples", "Install examples", false),
            use_flag("gtk-doc", "Build documentation", false),
            use_flag("introspection", "Enable GObject introspection", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("systemd", "Enable systemd integration", true),
            use_flag("test", "Build tests", false),
        ],
        dependencies: vec![
            dep("dev-libs", "glib"),
            dep("sys-libs", "pam"),
            dep("sys-apps", "dbus"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-util", "pkgconf"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/polkit-org/polkit/archive/refs/tags/124.tar.gz".to_string()),
        source_hash: Some("f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7".to_string()),
        buck_target: "//sys-auth/polkit:polkit-124".to_string(),
        size: 700_000,
        installed_size: 3_000_000,
    }
}

fn elogind_252_9() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-auth", "elogind"),
        version: Version::parse("252.9.0").unwrap(),
        slot: "0".to_string(),
        description: "Standalone logind extracted from systemd".to_string(),
        homepage: Some("https://github.com/elogind/elogind".to_string()),
        license: "LGPL-2.1 GPL-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("acl", "Enable ACL support", true),
            use_flag("audit", "Enable audit support", false),
            use_flag("cgroup-hybrid", "Enable hybrid cgroup hierarchy", false),
            use_flag("debug", "Enable debug build", false),
            use_flag("pam", "Enable PAM support", true),
            use_flag("policykit", "Enable PolicyKit support", true),
            use_flag("selinux", "Enable SELinux support", false),
        ],
        dependencies: vec![
            dep("sys-libs", "pam"),
            dep("sys-libs", "libcap"),
            dep("sys-apps", "dbus"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-util", "pkgconf"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/elogind/elogind/archive/refs/tags/v252.9.tar.gz".to_string()),
        source_hash: Some("a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8".to_string()),
        buck_target: "//sys-auth/elogind:elogind-252.9".to_string(),
        size: 1_500_000,
        installed_size: 5_000_000,
    }
}

fn cronie_1_7_2() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-process", "cronie"),
        version: Version::new(1, 7, 2),
        slot: "0".to_string(),
        description: "Cron daemon from Fedora".to_string(),
        homepage: Some("https://github.com/cronie-crond/cronie".to_string()),
        license: "ISC BSD BSD-2 GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("anacron", "Enable anacron compatibility", true),
            use_flag("audit", "Enable audit support", false),
            use_flag("inotify", "Enable inotify support", true),
            use_flag("pam", "Enable PAM support", true),
            use_flag("selinux", "Enable SELinux support", false),
        ],
        dependencies: vec![
            dep("sys-libs", "pam"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "autoconf"),
            dep_build("sys-devel", "automake"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/cronie-crond/cronie/releases/download/cronie-1.7.2/cronie-1.7.2.tar.gz".to_string()),
        source_hash: Some("b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9".to_string()),
        buck_target: "//sys-process/cronie:cronie-1.7.2".to_string(),
        size: 250_000,
        installed_size: 700_000,
    }
}
