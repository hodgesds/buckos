//! Package category definitions
//!
//! Categories follow the Gentoo/Portage convention for organizing packages.

use serde::{Deserialize, Serialize};

/// Standard package categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    // App categories
    AppAdmin,
    AppArch,
    AppCrypt,
    AppDoc,
    AppEditors,
    AppEmulation,
    AppMisc,
    AppShells,
    AppText,

    // Dev categories
    DevDb,
    DevLang,
    DevLibs,
    DevPerl,
    DevPython,
    DevRuby,
    DevRust,
    DevUtil,
    DevVcs,

    // Media categories
    MediaGfx,
    MediaLibs,
    MediaSound,
    MediaVideo,

    // Net categories
    NetAnalyzer,
    NetDns,
    NetFirewall,
    NetFs,
    NetLibs,
    NetMisc,
    NetProxy,
    NetVpn,
    NetWireless,

    // Sys categories
    SysApps,
    SysBoot,
    SysDevel,
    SysFs,
    SysKernel,
    SysLibs,
    SysProcess,

    // Virtual category for meta-packages
    Virtual,

    // X11 categories
    X11Apps,
    X11Base,
    X11Libs,
    X11Misc,
    X11Wm,
}

impl Category {
    /// Get the string representation of the category
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::AppAdmin => "app-admin",
            Category::AppArch => "app-arch",
            Category::AppCrypt => "app-crypt",
            Category::AppDoc => "app-doc",
            Category::AppEditors => "app-editors",
            Category::AppEmulation => "app-emulation",
            Category::AppMisc => "app-misc",
            Category::AppShells => "app-shells",
            Category::AppText => "app-text",

            Category::DevDb => "dev-db",
            Category::DevLang => "dev-lang",
            Category::DevLibs => "dev-libs",
            Category::DevPerl => "dev-perl",
            Category::DevPython => "dev-python",
            Category::DevRuby => "dev-ruby",
            Category::DevRust => "dev-rust",
            Category::DevUtil => "dev-util",
            Category::DevVcs => "dev-vcs",

            Category::MediaGfx => "media-gfx",
            Category::MediaLibs => "media-libs",
            Category::MediaSound => "media-sound",
            Category::MediaVideo => "media-video",

            Category::NetAnalyzer => "net-analyzer",
            Category::NetDns => "net-dns",
            Category::NetFirewall => "net-firewall",
            Category::NetFs => "net-fs",
            Category::NetLibs => "net-libs",
            Category::NetMisc => "net-misc",
            Category::NetProxy => "net-proxy",
            Category::NetVpn => "net-vpn",
            Category::NetWireless => "net-wireless",

            Category::SysApps => "sys-apps",
            Category::SysBoot => "sys-boot",
            Category::SysDevel => "sys-devel",
            Category::SysFs => "sys-fs",
            Category::SysKernel => "sys-kernel",
            Category::SysLibs => "sys-libs",
            Category::SysProcess => "sys-process",

            Category::Virtual => "virtual",

            Category::X11Apps => "x11-apps",
            Category::X11Base => "x11-base",
            Category::X11Libs => "x11-libs",
            Category::X11Misc => "x11-misc",
            Category::X11Wm => "x11-wm",
        }
    }

    /// Get the description of the category
    pub fn description(&self) -> &'static str {
        match self {
            Category::AppAdmin => "Administration and system management utilities",
            Category::AppArch => "Archiving and compression utilities",
            Category::AppCrypt => "Cryptography tools and utilities",
            Category::AppDoc => "Documentation tools",
            Category::AppEditors => "Text editors",
            Category::AppEmulation => "Emulation software",
            Category::AppMisc => "Miscellaneous applications",
            Category::AppShells => "Command-line shells",
            Category::AppText => "Text processing utilities",

            Category::DevDb => "Database development libraries",
            Category::DevLang => "Programming languages and compilers",
            Category::DevLibs => "Development libraries",
            Category::DevPerl => "Perl modules and libraries",
            Category::DevPython => "Python modules and libraries",
            Category::DevRuby => "Ruby gems and libraries",
            Category::DevRust => "Rust crates and libraries",
            Category::DevUtil => "Development utilities and tools",
            Category::DevVcs => "Version control systems",

            Category::MediaGfx => "Graphics applications and libraries",
            Category::MediaLibs => "Media processing libraries",
            Category::MediaSound => "Audio applications",
            Category::MediaVideo => "Video applications",

            Category::NetAnalyzer => "Network analysis tools",
            Category::NetDns => "DNS servers and utilities",
            Category::NetFirewall => "Firewall tools and daemons",
            Category::NetFs => "Network filesystems",
            Category::NetLibs => "Network libraries",
            Category::NetMisc => "Miscellaneous network tools",
            Category::NetProxy => "Proxy servers and clients",
            Category::NetVpn => "VPN clients and servers",
            Category::NetWireless => "Wireless networking tools",

            Category::SysApps => "System applications and utilities",
            Category::SysBoot => "Boot loaders and boot utilities",
            Category::SysDevel => "System development tools",
            Category::SysFs => "Filesystem utilities",
            Category::SysKernel => "Linux kernel and modules",
            Category::SysLibs => "System libraries",
            Category::SysProcess => "Process management tools",

            Category::Virtual => "Virtual/meta packages for dependency tracking",

            Category::X11Apps => "X11 applications",
            Category::X11Base => "X11 base system",
            Category::X11Libs => "X11 libraries",
            Category::X11Misc => "Miscellaneous X11 utilities",
            Category::X11Wm => "Window managers",
        }
    }

    /// Parse a category from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "app-admin" => Some(Category::AppAdmin),
            "app-arch" => Some(Category::AppArch),
            "app-crypt" => Some(Category::AppCrypt),
            "app-doc" => Some(Category::AppDoc),
            "app-editors" => Some(Category::AppEditors),
            "app-emulation" => Some(Category::AppEmulation),
            "app-misc" => Some(Category::AppMisc),
            "app-shells" => Some(Category::AppShells),
            "app-text" => Some(Category::AppText),

            "dev-db" => Some(Category::DevDb),
            "dev-lang" => Some(Category::DevLang),
            "dev-libs" => Some(Category::DevLibs),
            "dev-perl" => Some(Category::DevPerl),
            "dev-python" => Some(Category::DevPython),
            "dev-ruby" => Some(Category::DevRuby),
            "dev-rust" => Some(Category::DevRust),
            "dev-util" => Some(Category::DevUtil),
            "dev-vcs" => Some(Category::DevVcs),

            "media-gfx" => Some(Category::MediaGfx),
            "media-libs" => Some(Category::MediaLibs),
            "media-sound" => Some(Category::MediaSound),
            "media-video" => Some(Category::MediaVideo),

            "net-analyzer" => Some(Category::NetAnalyzer),
            "net-dns" => Some(Category::NetDns),
            "net-firewall" => Some(Category::NetFirewall),
            "net-fs" => Some(Category::NetFs),
            "net-libs" => Some(Category::NetLibs),
            "net-misc" => Some(Category::NetMisc),
            "net-proxy" => Some(Category::NetProxy),
            "net-vpn" => Some(Category::NetVpn),
            "net-wireless" => Some(Category::NetWireless),

            "sys-apps" => Some(Category::SysApps),
            "sys-boot" => Some(Category::SysBoot),
            "sys-devel" => Some(Category::SysDevel),
            "sys-fs" => Some(Category::SysFs),
            "sys-kernel" => Some(Category::SysKernel),
            "sys-libs" => Some(Category::SysLibs),
            "sys-process" => Some(Category::SysProcess),

            "virtual" => Some(Category::Virtual),

            "x11-apps" => Some(Category::X11Apps),
            "x11-base" => Some(Category::X11Base),
            "x11-libs" => Some(Category::X11Libs),
            "x11-misc" => Some(Category::X11Misc),
            "x11-wm" => Some(Category::X11Wm),

            _ => None,
        }
    }

    /// Get all categories
    pub fn all() -> Vec<Self> {
        vec![
            Category::AppAdmin,
            Category::AppArch,
            Category::AppCrypt,
            Category::AppDoc,
            Category::AppEditors,
            Category::AppEmulation,
            Category::AppMisc,
            Category::AppShells,
            Category::AppText,
            Category::DevDb,
            Category::DevLang,
            Category::DevLibs,
            Category::DevPerl,
            Category::DevPython,
            Category::DevRuby,
            Category::DevRust,
            Category::DevUtil,
            Category::DevVcs,
            Category::MediaGfx,
            Category::MediaLibs,
            Category::MediaSound,
            Category::MediaVideo,
            Category::NetAnalyzer,
            Category::NetDns,
            Category::NetFirewall,
            Category::NetFs,
            Category::NetLibs,
            Category::NetMisc,
            Category::NetProxy,
            Category::NetVpn,
            Category::NetWireless,
            Category::SysApps,
            Category::SysBoot,
            Category::SysDevel,
            Category::SysFs,
            Category::SysKernel,
            Category::SysLibs,
            Category::SysProcess,
            Category::Virtual,
            Category::X11Apps,
            Category::X11Base,
            Category::X11Libs,
            Category::X11Misc,
            Category::X11Wm,
        ]
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
