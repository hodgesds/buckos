# Copyright (c) BuckOS Authors. All rights reserved.
# USE Flag Definitions and Management

"""
USE Flag Module

This module defines all available USE flags, their categories, and provides
functions for USE flag management in package builds.
"""

# USE flag categories and their flags
USE_FLAG_CATEGORIES = {
    "build": {
        "description": "Build and compilation options",
        "flags": {
            "debug": "Build with debug symbols and assertions",
            "doc": "Build and install documentation",
            "examples": "Install example code and configurations",
            "static": "Build static binaries",
            "static-libs": "Build static libraries",
            "test": "Build and run test suite",
            "lto": "Enable Link Time Optimization",
            "pgo": "Enable Profile Guided Optimization",
            "native": "Optimize for current CPU architecture",
        },
    },
    "security": {
        "description": "Security-related features",
        "flags": {
            "caps": "Linux capabilities support",
            "hardened": "Enable hardened build flags",
            "pie": "Build Position Independent Executables",
            "seccomp": "Enable seccomp filtering",
            "selinux": "SELinux support",
            "ssp": "Stack Smashing Protection",
            "audit": "Enable audit subsystem support",
        },
    },
    "networking": {
        "description": "Network-related features",
        "flags": {
            "ipv6": "IPv6 protocol support",
            "ssl": "OpenSSL support",
            "gnutls": "GnuTLS support",
            "libressl": "LibreSSL support",
            "http2": "HTTP/2 protocol support",
            "http3": "HTTP/3 (QUIC) protocol support",
            "curl": "libcurl support",
            "tcpd": "TCP wrappers support",
        },
    },
    "compression": {
        "description": "Compression library support",
        "flags": {
            "brotli": "Brotli compression support",
            "bzip2": "Bzip2 compression support",
            "lz4": "LZ4 compression support",
            "lzma": "LZMA/XZ compression support",
            "zlib": "Zlib compression support",
            "zstd": "Zstandard compression support",
        },
    },
    "graphics": {
        "description": "Graphics and display support",
        "flags": {
            "X": "X11 windowing system support",
            "wayland": "Wayland display server support",
            "opengl": "OpenGL support",
            "vulkan": "Vulkan graphics API support",
            "egl": "EGL API support",
            "gles2": "OpenGL ES 2.x support",
        },
    },
    "toolkits": {
        "description": "GUI toolkit support",
        "flags": {
            "gtk": "GTK+ 3 toolkit support",
            "gtk4": "GTK 4 toolkit support",
            "qt5": "Qt5 toolkit support",
            "qt6": "Qt6 toolkit support",
            "ncurses": "Ncurses TUI support",
        },
    },
    "audio": {
        "description": "Audio system support",
        "flags": {
            "alsa": "ALSA audio support",
            "pulseaudio": "PulseAudio support",
            "pipewire": "PipeWire support",
            "jack": "JACK audio support",
            "oss": "OSS audio support",
        },
    },
    "languages": {
        "description": "Programming language bindings",
        "flags": {
            "python": "Python bindings",
            "perl": "Perl bindings",
            "ruby": "Ruby bindings",
            "lua": "Lua bindings",
            "tcl": "Tcl bindings",
            "guile": "Guile Scheme bindings",
        },
    },
    "system": {
        "description": "System integration features",
        "flags": {
            "dbus": "D-Bus message bus support",
            "systemd": "systemd init system support",
            "openrc": "OpenRC init system support",
            "pam": "PAM authentication support",
            "acl": "Access Control List support",
            "xattr": "Extended attributes support",
            "udev": "udev device manager support",
            "polkit": "PolicyKit authorization support",
        },
    },
    "internationalization": {
        "description": "Internationalization support",
        "flags": {
            "nls": "Native Language Support",
            "unicode": "Unicode support",
            "icu": "ICU library support",
            "idn": "Internationalized Domain Names support",
        },
    },
    "database": {
        "description": "Database support",
        "flags": {
            "sqlite": "SQLite database support",
            "mysql": "MySQL/MariaDB support",
            "postgres": "PostgreSQL support",
            "ldap": "LDAP directory support",
            "berkdb": "Berkeley DB support",
        },
    },
}

# USE_EXPAND variables
USE_EXPAND = {
    "CPU_FLAGS_X86": {
        "description": "CPU instruction set extensions",
        "values": [
            "aes", "avx", "avx2", "avx512f", "avx512bw", "avx512cd",
            "avx512dq", "avx512vl", "f16c", "fma3", "mmx", "mmxext",
            "pclmul", "popcnt", "rdrand", "sha", "sse", "sse2", "sse3",
            "sse4_1", "sse4_2", "ssse3", "vpclmulqdq",
        ],
    },
    "VIDEO_CARDS": {
        "description": "Video card drivers",
        "values": [
            "amdgpu", "ast", "dummy", "fbdev", "i915", "i965",
            "intel", "nouveau", "nvidia", "radeon", "radeonsi",
            "vesa", "virtualbox", "virgl", "vmware",
        ],
    },
    "INPUT_DEVICES": {
        "description": "Input device drivers",
        "values": [
            "evdev", "joystick", "keyboard", "libinput", "mouse",
            "synaptics", "vmmouse", "wacom",
        ],
    },
    "L10N": {
        "description": "Localization settings",
        "values": [
            "en", "en-US", "en-GB", "de", "fr", "es", "it", "pt", "pt-BR",
            "ru", "zh-CN", "zh-TW", "ja", "ko",
        ],
    },
    "PYTHON_TARGETS": {
        "description": "Python implementation targets",
        "values": ["python3_10", "python3_11", "python3_12", "python3_13"],
    },
    "RUBY_TARGETS": {
        "description": "Ruby implementation targets",
        "values": ["ruby31", "ruby32", "ruby33"],
    },
}

# Profile USE flag defaults
PROFILE_USE_DEFAULTS = {
    "minimal": ["ipv6"],
    "server": ["ssl", "ipv6", "threads", "caps"],
    "desktop": ["X", "dbus", "pulseaudio", "gtk", "ssl", "ipv6", "threads"],
    "developer": ["debug", "doc", "test", "ssl", "ipv6", "threads", "X", "dbus"],
    "hardened": ["hardened", "pie", "ssp", "caps", "ssl", "ipv6"],
    "embedded": ["static", "-ipv6"],
    "container": ["static", "-pam", "-systemd"],
}


def get_all_use_flags():
    """
    Get all available USE flags

    Returns:
        Dict mapping flag name to description
    """
    result = {}
    for category_info in USE_FLAG_CATEGORIES.values():
        result.update(category_info.get("flags", {}))
    return result


def get_use_flags_by_category(category):
    """
    Get USE flags for a specific category

    Args:
        category: Category name

    Returns:
        Dict mapping flag name to description, or empty dict
    """
    if category in USE_FLAG_CATEGORIES:
        return USE_FLAG_CATEGORIES[category].get("flags", {})
    return {}


def get_use_flag_description(flag):
    """
    Get description for a USE flag

    Args:
        flag: Flag name

    Returns:
        Description string or None
    """
    for category_info in USE_FLAG_CATEGORIES.values():
        flags = category_info.get("flags", {})
        if flag in flags:
            return flags[flag]
    return None


def get_use_flag_category(flag):
    """
    Get category for a USE flag

    Args:
        flag: Flag name

    Returns:
        Category name or None
    """
    for category, category_info in USE_FLAG_CATEGORIES.items():
        if flag in category_info.get("flags", {}):
            return category
    return None


def get_profile_use_defaults(profile):
    """
    Get default USE flags for a profile

    Args:
        profile: Profile name

    Returns:
        List of USE flags
    """
    return PROFILE_USE_DEFAULTS.get(profile, [])


def get_use_expand_values(variable):
    """
    Get possible values for a USE_EXPAND variable

    Args:
        variable: USE_EXPAND variable name

    Returns:
        List of possible values
    """
    if variable in USE_EXPAND:
        return USE_EXPAND[variable].get("values", [])
    return []


def list_use_expand_variables():
    """
    List all USE_EXPAND variables

    Returns:
        List of variable names
    """
    return sorted(USE_EXPAND.keys())


def validate_use_flag(flag):
    """
    Check if a USE flag is valid

    Args:
        flag: Flag name (may include - prefix)

    Returns:
        True if valid, False otherwise
    """
    clean_flag = flag.lstrip("-")
    return get_use_flag_description(clean_flag) != None


def use_package(
        name,
        version,
        src_uri,
        sha256,
        iuse = [],
        use_defaults = [],
        use_deps = {},
        use_configure = {},
        configure_args = [],
        make_args = [],
        install_args = [],
        post_install = "",
        maintainers = [],
        deps = [],
        build_deps = [],
        description = "",
        homepage = "",
        license = "",
        slot = "0",
        visibility = ["PUBLIC"]):
    """
    Define a package with USE flag support

    Args:
        name: Package name
        version: Package version
        src_uri: Source download URI
        sha256: Source checksum
        iuse: Available USE flags for this package
        use_defaults: Default enabled USE flags
        use_deps: Dict mapping USE flag to list of dependencies
        use_configure: Dict mapping USE flag to configure arguments
        configure_args: Base configure arguments
        make_args: Base make arguments
        install_args: Base install arguments
        post_install: Post-install shell commands
        maintainers: List of maintainer identifiers
        deps: Runtime dependencies
        build_deps: Build-time dependencies
        description: Package description
        homepage: Project homepage
        license: License identifier
        slot: Package slot
        visibility: Buck visibility
    """

    # Generate package metadata
    metadata = {
        "name": name,
        "version": version,
        "src_uri": src_uri,
        "sha256": sha256,
        "iuse": iuse,
        "use_defaults": use_defaults,
        "use_deps": use_deps,
        "use_configure": use_configure,
        "configure_args": configure_args,
        "make_args": make_args,
        "install_args": install_args,
        "post_install": post_install,
        "maintainers": maintainers,
        "deps": deps,
        "build_deps": build_deps,
        "description": description,
        "homepage": homepage,
        "license": license,
        "slot": slot,
    }

    # Create genrule for package metadata
    native.genrule(
        name = name + "_metadata",
        out = name + "_metadata.json",
        cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
        visibility = visibility,
    )

    # Create the actual package target
    native.filegroup(
        name = name,
        srcs = [":" + name + "_metadata"],
        visibility = visibility,
    )


def resolve_use_flags(package_use, global_use, profile_use, package_defaults):
    """
    Resolve final USE flags for a package

    Args:
        package_use: User's per-package USE settings
        global_use: User's global USE settings
        profile_use: Profile default USE flags
        package_defaults: Package's default USE flags

    Returns:
        Set of enabled USE flags
    """
    flags = set()

    # 1. Profile defaults
    for flag in profile_use:
        if flag.startswith("-"):
            flags.discard(flag[1:])
        else:
            flags.add(flag)

    # 2. Global user flags
    for flag in global_use:
        if flag.startswith("-"):
            flags.discard(flag[1:])
        else:
            flags.add(flag)

    # 3. Package defaults
    for flag in package_defaults:
        if flag.startswith("-"):
            flags.discard(flag[1:])
        else:
            flags.add(flag)

    # 4. Package user flags
    for flag in package_use:
        if flag.startswith("-"):
            flags.discard(flag[1:])
        else:
            flags.add(flag)

    return flags
