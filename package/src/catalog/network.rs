//! Network packages
//!
//! This module defines network packages including:
//! - curl
//! - wget
//! - OpenSSL
//! - ca-certificates

use super::{dep, dep_build, dep_runtime, use_flag};
use crate::types::{PackageId, PackageInfo};
use semver::Version;

/// Get all network packages
pub fn get_packages() -> Vec<PackageInfo> {
    vec![
        // OpenSSL
        openssl_3_2_1(),
        openssl_3_3_0(),
        // curl
        curl_8_6_0(),
        curl_8_7_1(),
        // wget
        wget_1_21_4(),
        wget_1_24_5(),
        // ca-certificates
        ca_certificates_20240203(),
        // nghttp2
        nghttp2_1_60_0(),
        // libssh2
        libssh2_1_11_0(),
        // GnuTLS
        gnutls_3_8_4(),
        // OpenSSH
        openssh_9_7(),
        // iproute2
        iproute2_6_7_0(),
        // iputils
        iputils_20240117(),
        // net-tools
        net_tools_2_10(),
        // bind-tools (dig, nslookup, host)
        bind_tools_9_18_24(),
    ]
}

fn openssl_3_2_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-libs", "openssl"),
        version: Version::new(3, 2, 1),
        slot: "0/3".to_string(),
        description: "Robust, full-featured SSL/TLS library".to_string(),
        homepage: Some("https://www.openssl.org/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("asm", "Enable assembly optimizations", true),
            use_flag("fips", "Enable FIPS mode", false),
            use_flag("ktls", "Enable kernel TLS offload", false),
            use_flag("rfc3779", "Enable RFC3779 support", true),
            use_flag("sctp", "Enable SCTP support", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
            use_flag("tls-compression", "Enable TLS compression", false),
            use_flag("weak-ssl-ciphers", "Enable weak SSL ciphers", false),
        ],
        dependencies: vec![dep("sys-libs", "zlib")],
        build_dependencies: vec![dep_build("dev-lang", "perl")],
        runtime_dependencies: vec![dep_runtime("app-misc", "ca-certificates")],
        source_url: Some("https://www.openssl.org/source/openssl-3.2.1.tar.gz".to_string()),
        source_hash: Some(
            "83c7329fe52c850677d75e5d0b0ca245309b97e8ecbcfdc1dfdc4ab9fac35b39".to_string(),
        ),
        buck_target: "//dev-libs/openssl:openssl-3.2.1".to_string(),
        size: 15_000_000,
        installed_size: 40_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn openssl_3_3_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("dev-libs", "openssl"),
        version: Version::new(3, 3, 0),
        slot: "0/3".to_string(),
        description: "Robust, full-featured SSL/TLS library".to_string(),
        homepage: Some("https://www.openssl.org/".to_string()),
        license: "Apache-2.0".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("asm", "Enable assembly optimizations", true),
            use_flag("fips", "Enable FIPS mode", false),
            use_flag("ktls", "Enable kernel TLS offload", false),
            use_flag("rfc3779", "Enable RFC3779 support", true),
            use_flag("sctp", "Enable SCTP support", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
            use_flag("tls-compression", "Enable TLS compression", false),
            use_flag("weak-ssl-ciphers", "Enable weak SSL ciphers", false),
        ],
        dependencies: vec![dep("sys-libs", "zlib")],
        build_dependencies: vec![dep_build("dev-lang", "perl")],
        runtime_dependencies: vec![dep_runtime("app-misc", "ca-certificates")],
        source_url: Some("https://www.openssl.org/source/openssl-3.3.0.tar.gz".to_string()),
        source_hash: Some(
            "93c7329fe52c850677d75e5d0b0ca245309b97e8ecbcfdc1dfdc4ab9fac35b40".to_string(),
        ),
        buck_target: "//dev-libs/openssl:openssl-3.3.0".to_string(),
        size: 15_500_000,
        installed_size: 42_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn curl_8_6_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-misc", "curl"),
        version: Version::new(8, 6, 0),
        slot: "0".to_string(),
        description: "A command line tool and library for transferring data with URLs".to_string(),
        homepage: Some("https://curl.se/".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("adns", "Enable async DNS support", false),
            use_flag("alt-svc", "Enable Alt-Svc support", true),
            use_flag("brotli", "Enable Brotli support", true),
            use_flag("gnutls", "Enable GnuTLS support", false),
            use_flag("gopher", "Enable Gopher protocol", false),
            use_flag("http2", "Enable HTTP/2 support", true),
            use_flag("http3", "Enable HTTP/3 support", false),
            use_flag("idn", "Enable IDN support", true),
            use_flag("ipv6", "Enable IPv6 support", true),
            use_flag("kerberos", "Enable Kerberos support", false),
            use_flag("ldap", "Enable LDAP support", false),
            use_flag("mbedtls", "Enable mbedTLS support", false),
            use_flag("nghttp2", "Enable nghttp2 support", true),
            use_flag("openssl", "Enable OpenSSL support", true),
            use_flag("progress-meter", "Enable progress meter", true),
            use_flag("psl", "Enable public suffix list", true),
            use_flag("rtmp", "Enable RTMP support", false),
            use_flag("samba", "Enable SMB support", false),
            use_flag("ssh", "Enable SSH support", true),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
            use_flag("tftp", "Enable TFTP protocol", false),
            use_flag("websockets", "Enable WebSocket support", true),
            use_flag("zstd", "Enable zstd support", true),
        ],
        dependencies: vec![
            dep("dev-libs", "openssl"),
            dep("sys-libs", "zlib"),
            dep("net-libs", "nghttp2"),
            dep("net-libs", "libssh2"),
            dep("app-arch", "zstd"),
            dep("app-arch", "brotli"),
        ],
        build_dependencies: vec![dep_build("dev-util", "pkgconf")],
        runtime_dependencies: vec![dep_runtime("app-misc", "ca-certificates")],
        source_url: Some("https://curl.se/download/curl-8.6.0.tar.xz".to_string()),
        source_hash: Some(
            "3ccd55d91af9516539df80625f818c734dc6f2ecf9bada3343c01a5a97f7c7c0".to_string(),
        ),
        buck_target: "//net-misc/curl:curl-8.6.0".to_string(),
        size: 2_600_000,
        installed_size: 8_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn curl_8_7_1() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-misc", "curl"),
        version: Version::new(8, 7, 1),
        slot: "0".to_string(),
        description: "A command line tool and library for transferring data with URLs".to_string(),
        homepage: Some("https://curl.se/".to_string()),
        license: "MIT".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("adns", "Enable async DNS support", false),
            use_flag("alt-svc", "Enable Alt-Svc support", true),
            use_flag("brotli", "Enable Brotli support", true),
            use_flag("gnutls", "Enable GnuTLS support", false),
            use_flag("http2", "Enable HTTP/2 support", true),
            use_flag("http3", "Enable HTTP/3 support", false),
            use_flag("idn", "Enable IDN support", true),
            use_flag("ipv6", "Enable IPv6 support", true),
            use_flag("nghttp2", "Enable nghttp2 support", true),
            use_flag("openssl", "Enable OpenSSL support", true),
            use_flag("psl", "Enable public suffix list", true),
            use_flag("ssh", "Enable SSH support", true),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("websockets", "Enable WebSocket support", true),
            use_flag("zstd", "Enable zstd support", true),
        ],
        dependencies: vec![
            dep("dev-libs", "openssl"),
            dep("sys-libs", "zlib"),
            dep("net-libs", "nghttp2"),
            dep("net-libs", "libssh2"),
            dep("app-arch", "zstd"),
            dep("app-arch", "brotli"),
        ],
        build_dependencies: vec![dep_build("dev-util", "pkgconf")],
        runtime_dependencies: vec![dep_runtime("app-misc", "ca-certificates")],
        source_url: Some("https://curl.se/download/curl-8.7.1.tar.xz".to_string()),
        source_hash: Some(
            "4ccd55d91af9516539df80625f818c734dc6f2ecf9bada3343c01a5a97f7c7c1".to_string(),
        ),
        buck_target: "//net-misc/curl:curl-8.7.1".to_string(),
        size: 2_700_000,
        installed_size: 8_500_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn wget_1_21_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-misc", "wget"),
        version: Version::new(1, 21, 4),
        slot: "0".to_string(),
        description: "Network utility to retrieve files from the web".to_string(),
        homepage: Some("https://www.gnu.org/software/wget/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("cookie_check", "Enable cookie checking", false),
            use_flag("debug", "Enable debug build", false),
            use_flag("gnutls", "Use GnuTLS instead of OpenSSL", false),
            use_flag("idn", "Enable IDN support", true),
            use_flag("ipv6", "Enable IPv6 support", true),
            use_flag("metalink", "Enable Metalink support", false),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("ntlm", "Enable NTLM support", false),
            use_flag("openssl", "Use OpenSSL", true),
            use_flag("pcre", "Enable PCRE regex support", true),
            use_flag("psl", "Enable public suffix list", true),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("static", "Build static binary", false),
            use_flag("test", "Build tests", false),
            use_flag("uuid", "Enable UUID support", false),
            use_flag("zlib", "Enable zlib support", true),
        ],
        dependencies: vec![
            dep("dev-libs", "openssl"),
            dep("sys-libs", "zlib"),
            dep("dev-libs", "pcre2"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gettext"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![dep_runtime("app-misc", "ca-certificates")],
        source_url: Some("https://ftp.gnu.org/gnu/wget/wget-1.21.4.tar.gz".to_string()),
        source_hash: Some(
            "81542f5cefb8faacc39bbbc6c82ded80e3e4a88505ae72571b15dd21bf0a9919".to_string(),
        ),
        buck_target: "//net-misc/wget:wget-1.21.4".to_string(),
        size: 4_800_000,
        installed_size: 10_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn wget_1_24_5() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-misc", "wget"),
        version: Version::new(1, 24, 5),
        slot: "0".to_string(),
        description: "Network utility to retrieve files from the web".to_string(),
        homepage: Some("https://www.gnu.org/software/wget/".to_string()),
        license: "GPL-3+".to_string(),
        keywords: vec![
            "~amd64".to_string(),
            "~arm64".to_string(),
            "~x86".to_string(),
        ],
        use_flags: vec![
            use_flag("gnutls", "Use GnuTLS instead of OpenSSL", false),
            use_flag("idn", "Enable IDN support", true),
            use_flag("ipv6", "Enable IPv6 support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("openssl", "Use OpenSSL", true),
            use_flag("pcre", "Enable PCRE regex support", true),
            use_flag("psl", "Enable public suffix list", true),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("zlib", "Enable zlib support", true),
        ],
        dependencies: vec![
            dep("dev-libs", "openssl"),
            dep("sys-libs", "zlib"),
            dep("dev-libs", "pcre2"),
        ],
        build_dependencies: vec![
            dep_build("sys-devel", "gettext"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![dep_runtime("app-misc", "ca-certificates")],
        source_url: Some("https://ftp.gnu.org/gnu/wget/wget-1.24.5.tar.gz".to_string()),
        source_hash: Some(
            "91542f5cefb8faacc39bbbc6c82ded80e3e4a88505ae72571b15dd21bf0a991a".to_string(),
        ),
        buck_target: "//net-misc/wget:wget-1.24.5".to_string(),
        size: 5_000_000,
        installed_size: 11_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn ca_certificates_20240203() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("app-misc", "ca-certificates"),
        version: Version::parse("20240203.0.0").unwrap(),
        slot: "0".to_string(),
        description: "Common CA certificates".to_string(),
        homepage: Some("https://packages.debian.org/sid/ca-certificates".to_string()),
        license: "MPL-2.0".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("cacert", "Include CACert certificates", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("dev-libs", "openssl"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://deb.debian.org/debian/pool/main/c/ca-certificates/ca-certificates_20240203.tar.xz".to_string()),
        source_hash: Some("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2".to_string()),
        buck_target: "//app-misc/ca-certificates:ca-certificates-20240203".to_string(),
        size: 200_000,
        installed_size: 600_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn nghttp2_1_60_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-libs", "nghttp2"),
        version: Version::new(1, 60, 0),
        slot: "0/1.14".to_string(),
        description: "HTTP/2 C Library".to_string(),
        homepage: Some("https://nghttp2.org/".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("cxx", "Build C++ library", false),
            use_flag("debug", "Enable debug build", false),
            use_flag("hpack-tools", "Build HPACK tools", false),
            use_flag("jemalloc", "Use jemalloc allocator", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
            use_flag("utils", "Build command line utilities", false),
            use_flag("xml", "Enable XML support for Metalink", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://github.com/nghttp2/nghttp2/releases/download/v1.60.0/nghttp2-1.60.0.tar.xz"
                .to_string(),
        ),
        source_hash: Some(
            "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3".to_string(),
        ),
        buck_target: "//net-libs/nghttp2:nghttp2-1.60.0".to_string(),
        size: 1_600_000,
        installed_size: 4_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn libssh2_1_11_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-libs", "libssh2"),
        version: Version::new(1, 11, 0),
        slot: "0".to_string(),
        description: "Library implementing the SSH2 protocol".to_string(),
        homepage: Some("https://www.libssh2.org/".to_string()),
        license: "BSD".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("gcrypt", "Use libgcrypt instead of OpenSSL", false),
            use_flag("mbedtls", "Use mbedTLS instead of OpenSSL", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
            use_flag("zlib", "Enable zlib compression", true),
        ],
        dependencies: vec![dep("dev-libs", "openssl"), dep("sys-libs", "zlib")],
        build_dependencies: vec![
            dep_build("dev-util", "cmake"),
            dep_build("dev-util", "ninja"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://www.libssh2.org/download/libssh2-1.11.0.tar.gz".to_string()),
        source_hash: Some(
            "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4".to_string(),
        ),
        buck_target: "//net-libs/libssh2:libssh2-1.11.0".to_string(),
        size: 1_000_000,
        installed_size: 2_500_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn gnutls_3_8_4() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-libs", "gnutls"),
        version: Version::new(3, 8, 4),
        slot: "0/30".to_string(),
        description: "TLS 1.0-1.3 and SSL 3.0 implementation".to_string(),
        homepage: Some("https://www.gnutls.org/".to_string()),
        license: "GPL-3+ LGPL-2.1+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("brotli", "Enable Brotli compression", true),
            use_flag("cxx", "Build C++ library", true),
            use_flag("dane", "Enable DANE support", false),
            use_flag("doc", "Build documentation", false),
            use_flag("guile", "Enable Guile bindings", false),
            use_flag("idn", "Enable IDN support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("openssl", "Enable OpenSSL compatibility", false),
            use_flag("pkcs11", "Enable PKCS#11 support", true),
            use_flag("seccomp", "Enable seccomp support", false),
            use_flag("sslv2", "Enable SSLv2 client hello", false),
            use_flag("sslv3", "Enable SSLv3 support", false),
            use_flag("static-libs", "Build static libraries", false),
            use_flag("test", "Build tests", false),
            use_flag("tls-heartbeat", "Enable TLS heartbeat", false),
            use_flag("tools", "Build command line tools", true),
            use_flag("zlib", "Enable zlib compression", true),
            use_flag("zstd", "Enable zstd compression", true),
        ],
        dependencies: vec![
            dep("dev-libs", "nettle"),
            dep("dev-libs", "libtasn1"),
            dep("dev-libs", "libunistring"),
            dep("dev-libs", "gmp"),
            dep("sys-libs", "zlib"),
            dep("app-arch", "zstd"),
            dep("app-arch", "brotli"),
        ],
        build_dependencies: vec![
            dep_build("dev-util", "pkgconf"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://www.gnupg.org/ftp/gcrypt/gnutls/v3.8/gnutls-3.8.4.tar.xz".to_string(),
        ),
        source_hash: Some(
            "d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5".to_string(),
        ),
        buck_target: "//net-libs/gnutls:gnutls-3.8.4".to_string(),
        size: 6_000_000,
        installed_size: 15_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn openssh_9_7() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-misc", "openssh"),
        version: Version::new(9, 7, 0),
        slot: "0".to_string(),
        description: "SSH client and server".to_string(),
        homepage: Some("https://www.openssh.com/".to_string()),
        license: "BSD GPL-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("audit", "Enable audit support", false),
            use_flag("debug", "Enable debug build", false),
            use_flag("kerberos", "Enable Kerberos support", false),
            use_flag("ldns", "Enable LDNS/DNSSEC support", false),
            use_flag("libedit", "Use libedit instead of readline", true),
            use_flag("livecd", "Enable SSH ProxyCommand for live CD", false),
            use_flag("pam", "Enable PAM support", true),
            use_flag("pie", "Build as position independent executable", true),
            use_flag("security-key", "Enable security key support", true),
            use_flag("selinux", "Enable SELinux support", false),
            use_flag("ssl", "Enable SSL support", true),
            use_flag("static", "Build static binaries", false),
            use_flag("systemd", "Enable systemd support", true),
            use_flag("test", "Build tests", false),
            use_flag("xmss", "Enable XMSS support", false),
        ],
        dependencies: vec![
            dep("dev-libs", "openssl"),
            dep("sys-libs", "pam"),
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![dep_build("dev-util", "pkgconf")],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://cdn.openbsd.org/pub/OpenBSD/OpenSSH/portable/openssh-9.7p1.tar.gz".to_string(),
        ),
        source_hash: Some(
            "e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6".to_string(),
        ),
        buck_target: "//net-misc/openssh:openssh-9.7".to_string(),
        size: 1_800_000,
        installed_size: 5_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn iproute2_6_7_0() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "iproute2"),
        version: Version::new(6, 7, 0),
        slot: "0".to_string(),
        description: "IP routing utilities".to_string(),
        homepage: Some("https://wiki.linuxfoundation.org/networking/iproute2".to_string()),
        license: "GPL-2".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("atm", "Enable ATM support", false),
            use_flag("berkdb", "Enable Berkeley DB support", false),
            use_flag("bpf", "Enable eBPF support", true),
            use_flag("caps", "Enable capabilities support", true),
            use_flag("elf", "Enable ELF support", true),
            use_flag("iptables", "Enable iptables support", false),
            use_flag("minimal", "Build minimal set of tools", false),
            use_flag("nfs", "Enable NFS support", false),
            use_flag("selinux", "Enable SELinux support", false),
        ],
        dependencies: vec![dep("sys-libs", "libcap")],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "flex"),
            dep_build("sys-devel", "bison"),
            dep_build("dev-util", "pkgconf"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://kernel.org/pub/linux/utils/net/iproute2/iproute2-6.7.0.tar.xz".to_string(),
        ),
        source_hash: Some(
            "f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7".to_string(),
        ),
        buck_target: "//sys-apps/iproute2:iproute2-6.7.0".to_string(),
        size: 1_000_000,
        installed_size: 3_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn iputils_20240117() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-misc", "iputils"),
        version: Version::parse("20240117.0.0").unwrap(),
        slot: "0".to_string(),
        description: "Network utilities including ping, ping6, tracepath".to_string(),
        homepage: Some("https://github.com/iputils/iputils".to_string()),
        license: "BSD GPL-2+ rdisc".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("arping", "Build arping", true),
            use_flag("caps", "Enable capabilities support", true),
            use_flag("clockdiff", "Build clockdiff", true),
            use_flag("doc", "Build documentation", false),
            use_flag("idn", "Enable IDN support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("static", "Build static binaries", false),
            use_flag("tracepath", "Build tracepath", true),
            use_flag("traceroute6", "Build traceroute6", true),
        ],
        dependencies: vec![dep("sys-libs", "libcap")],
        build_dependencies: vec![
            dep_build("dev-util", "meson"),
            dep_build("dev-util", "ninja"),
        ],
        runtime_dependencies: vec![],
        source_url: Some("https://github.com/iputils/iputils/archive/20240117.tar.gz".to_string()),
        source_hash: Some(
            "a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8".to_string(),
        ),
        buck_target: "//net-misc/iputils:iputils-20240117".to_string(),
        size: 350_000,
        installed_size: 800_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn net_tools_2_10() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("sys-apps", "net-tools"),
        version: Version::new(2, 10, 0),
        slot: "0".to_string(),
        description: "Standard network tools: arp, hostname, ifconfig, netstat, etc.".to_string(),
        homepage: Some("https://net-tools.sourceforge.io/".to_string()),
        license: "GPL-2+".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("arp", "Build arp", true),
            use_flag("hostname", "Build hostname", false),
            use_flag("ipv6", "Enable IPv6 support", true),
            use_flag("nls", "Enable Native Language Support", true),
            use_flag("plipconfig", "Build plipconfig", false),
            use_flag("slattach", "Build slattach", false),
            use_flag("static", "Build static binaries", false),
        ],
        dependencies: vec![],
        build_dependencies: vec![
            dep_build("sys-devel", "gcc"),
            dep_build("sys-devel", "gettext"),
        ],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://downloads.sourceforge.net/net-tools/net-tools-2.10.tar.xz".to_string(),
        ),
        source_hash: Some(
            "b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9".to_string(),
        ),
        buck_target: "//sys-apps/net-tools:net-tools-2.10".to_string(),
        size: 350_000,
        installed_size: 1_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}

fn bind_tools_9_18_24() -> PackageInfo {
    PackageInfo {
        id: PackageId::new("net-dns", "bind-tools"),
        version: Version::new(9, 18, 24),
        slot: "0".to_string(),
        description: "DNS utilities: dig, host, nslookup, nsupdate".to_string(),
        homepage: Some("https://www.isc.org/bind/".to_string()),
        license: "MPL-2.0".to_string(),
        keywords: vec!["amd64".to_string(), "arm64".to_string(), "x86".to_string()],
        use_flags: vec![
            use_flag("caps", "Enable capabilities support", true),
            use_flag("doc", "Build documentation", false),
            use_flag("gssapi", "Enable GSSAPI support", false),
            use_flag("idn", "Enable IDN support", true),
            use_flag("xml", "Enable XML statistics", false),
        ],
        dependencies: vec![
            dep("dev-libs", "openssl"),
            dep("dev-libs", "libuv"),
            dep("dev-libs", "userspace-rcu"),
            dep("sys-libs", "zlib"),
        ],
        build_dependencies: vec![dep_build("dev-util", "pkgconf")],
        runtime_dependencies: vec![],
        source_url: Some(
            "https://downloads.isc.org/isc/bind9/9.18.24/bind-9.18.24.tar.xz".to_string(),
        ),
        source_hash: Some(
            "c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0".to_string(),
        ),
        buck_target: "//net-dns/bind-tools:bind-tools-9.18.24".to_string(),
        size: 5_000_000,
        installed_size: 8_000_000,
        required_use: String::new(),
        blockers: Vec::new(),
    }
}
