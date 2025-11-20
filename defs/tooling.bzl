# Copyright (c) BuckOS Authors. All rights reserved.
# External Tooling Integration

"""
Tooling Integration Module

This module provides the primary integration point for external tools
like package managers. It exposes functions for querying packages,
generating configurations, and building packages.
"""

load("//defs:registry.bzl", "PACKAGE_REGISTRY", "get_package_info", "get_default_version", "get_all_versions", "get_stable_versions", "get_version_status", "list_all_packages", "list_packages_by_category", "search_packages")
load("//defs:use_flags.bzl", "USE_FLAG_CATEGORIES", "PROFILE_USE_DEFAULTS", "USE_EXPAND", "get_all_use_flags", "get_use_flags_by_category", "get_use_flag_description", "get_profile_use_defaults", "resolve_use_flags")
load("//defs:package_sets.bzl", "SYSTEM_SETS", "TASK_SETS", "DESKTOP_SETS", "get_set_packages", "get_set_info", "list_all_sets", "list_sets_by_type", "union_sets", "intersection_sets", "difference_sets", "compare_sets")
load("//defs:versions.bzl", "compare_versions", "version_satisfies", "select_best_version", "get_upgrade_path")
load("//defs:maintainers.bzl", "MAINTAINERS", "get_maintainer_info", "get_maintainer_packages", "get_package_maintainers")

# Profile definitions
PROFILES = {
    "minimal": {
        "description": "Absolute minimum system",
        "parent": None,
        "use_defaults": PROFILE_USE_DEFAULTS.get("minimal", []),
        "package_set": "minimal",
    },
    "server": {
        "description": "Server systems",
        "parent": "minimal",
        "use_defaults": PROFILE_USE_DEFAULTS.get("server", []),
        "package_set": "server",
    },
    "desktop": {
        "description": "Desktop systems",
        "parent": "server",
        "use_defaults": PROFILE_USE_DEFAULTS.get("desktop", []),
        "package_set": "desktop",
    },
    "developer": {
        "description": "Development environment",
        "parent": "desktop",
        "use_defaults": PROFILE_USE_DEFAULTS.get("developer", []),
        "package_set": "developer",
    },
    "hardened": {
        "description": "Security-focused systems",
        "parent": "server",
        "use_defaults": PROFILE_USE_DEFAULTS.get("hardened", []),
        "package_set": "hardened",
    },
    "embedded": {
        "description": "Embedded systems",
        "parent": "minimal",
        "use_defaults": PROFILE_USE_DEFAULTS.get("embedded", []),
        "package_set": "minimal",
    },
    "container": {
        "description": "Container environments",
        "parent": "minimal",
        "use_defaults": PROFILE_USE_DEFAULTS.get("container", []),
        "package_set": "minimal",
    },
}


def generate_system_config(
        profile = "server",
        detected_hardware = [],
        detected_features = [],
        user_use_flags = [],
        package_overrides = {},
        env_preset = None,
        target_arch = "x86_64"):
    """
    Generate system configuration

    Args:
        profile: System profile name
        detected_hardware: Hardware detection results
        detected_features: Feature detection results
        user_use_flags: User-specified USE flags
        package_overrides: Per-package USE overrides
        env_preset: Environment preset name
        target_arch: Target architecture

    Returns:
        Complete system configuration dict
    """

    # Get profile info
    profile_info = PROFILES.get(profile, PROFILES["server"])

    # Calculate USE flags
    use_flags = set()

    # Add profile defaults
    for flag in profile_info.get("use_defaults", []):
        if flag.startswith("-"):
            use_flags.discard(flag[1:])
        else:
            use_flags.add(flag)

    # Add hardware-detected flags
    for flag in detected_hardware:
        use_flags.add(flag)

    # Add feature-detected flags
    for flag in detected_features:
        use_flags.add(flag)

    # Add user flags
    for flag in user_use_flags:
        if flag.startswith("-"):
            use_flags.discard(flag[1:])
        else:
            use_flags.add(flag)

    # Set up environment based on preset
    env = get_env_preset(env_preset, target_arch)

    # Build configuration
    config = {
        "profile": profile,
        "arch": target_arch,
        "use_flags": {
            "global": sorted(use_flags),
            "package": package_overrides,
        },
        "env": env,
        "package_env": {},
        "accept_keywords": [],
        "package_mask": [],
        "package_unmask": [],
    }

    return config


def get_env_preset(preset, arch):
    """
    Get environment variables for a preset

    Args:
        preset: Preset name (optimize-speed, optimize-size, debug, etc.)
        arch: Target architecture

    Returns:
        Dict of environment variables
    """

    # Default flags
    base_cflags = "-O2 -pipe"
    if arch == "x86_64":
        base_cflags += " -march=x86-64"
    elif arch == "aarch64":
        base_cflags += " -march=armv8-a"

    presets = {
        "optimize-speed": {
            "CFLAGS": "-O3 -pipe -march=native",
            "CXXFLAGS": "-O3 -pipe -march=native",
            "LDFLAGS": "-Wl,-O1 -Wl,--as-needed",
            "MAKEOPTS": "-j$(nproc)",
        },
        "optimize-size": {
            "CFLAGS": "-Os -pipe",
            "CXXFLAGS": "-Os -pipe",
            "LDFLAGS": "-Wl,-O1 -Wl,--as-needed -Wl,--gc-sections",
            "MAKEOPTS": "-j$(nproc)",
        },
        "debug": {
            "CFLAGS": "-O0 -g -pipe",
            "CXXFLAGS": "-O0 -g -pipe",
            "LDFLAGS": "",
            "MAKEOPTS": "-j$(nproc)",
        },
        "default": {
            "CFLAGS": base_cflags,
            "CXXFLAGS": base_cflags,
            "LDFLAGS": "-Wl,-O1 -Wl,--as-needed",
            "MAKEOPTS": "-j$(nproc)",
        },
    }

    return presets.get(preset, presets["default"])


def export_config_json(config):
    """
    Export configuration as JSON

    Args:
        config: Configuration dict

    Returns:
        JSON string
    """
    return json.encode_indent(config, indent = "  ")


def export_config_toml(config):
    """
    Export configuration as TOML

    Args:
        config: Configuration dict

    Returns:
        TOML string
    """
    lines = []
    lines.append("[profile]")
    lines.append('name = "{}"'.format(config.get("profile", "")))
    lines.append('arch = "{}"'.format(config.get("arch", "")))
    lines.append("")

    lines.append("[use_flags]")
    global_flags = config.get("use_flags", {}).get("global", [])
    lines.append("global = {}".format(json.encode(global_flags)))
    lines.append("")

    pkg_flags = config.get("use_flags", {}).get("package", {})
    if pkg_flags:
        lines.append("[use_flags.package]")
        for pkg, flags in pkg_flags.items():
            lines.append('{} = {}'.format(pkg, json.encode(flags)))
        lines.append("")

    lines.append("[env]")
    for key, value in config.get("env", {}).items():
        lines.append('{} = "{}"'.format(key, value))

    return "\n".join(lines)


def export_config_shell(config):
    """
    Export configuration as shell script

    Args:
        config: Configuration dict

    Returns:
        Shell script string
    """
    lines = []
    lines.append("#!/bin/bash")
    lines.append("# BuckOS Configuration")
    lines.append("")

    lines.append('export PROFILE="{}"'.format(config.get("profile", "")))
    lines.append('export ARCH="{}"'.format(config.get("arch", "")))
    lines.append("")

    lines.append("# USE flags")
    global_flags = config.get("use_flags", {}).get("global", [])
    lines.append('export USE="{}"'.format(" ".join(global_flags)))
    lines.append("")

    lines.append("# Environment")
    for key, value in config.get("env", {}).items():
        lines.append('export {}="{}"'.format(key, value))

    return "\n".join(lines)


def export_buck_config(config):
    """
    Export configuration as Buck2 config

    Args:
        config: Configuration dict

    Returns:
        Buck config string
    """
    lines = []
    lines.append("# Generated BuckOS configuration")
    lines.append("")

    lines.append('PROFILE = "{}"'.format(config.get("profile", "")))
    lines.append('ARCH = "{}"'.format(config.get("arch", "")))
    lines.append("")

    global_flags = config.get("use_flags", {}).get("global", [])
    lines.append("USE_FLAGS = {}".format(json.encode(global_flags)))
    lines.append("")

    pkg_flags = config.get("use_flags", {}).get("package", {})
    lines.append("PACKAGE_USE = {}".format(json.encode(pkg_flags)))
    lines.append("")

    lines.append("ENV = {}".format(json.encode(config.get("env", {}))))

    return "\n".join(lines)


def get_profile_info(profile_name):
    """
    Get profile information

    Args:
        profile_name: Profile name

    Returns:
        Profile info dict or None
    """
    return PROFILES.get(profile_name)


def list_profiles():
    """
    List available profiles

    Returns:
        List of profile names
    """
    return sorted(PROFILES.keys())


def get_profile_use_flags(profile_name):
    """
    Get USE flags for a profile, including inherited flags

    Args:
        profile_name: Profile name

    Returns:
        List of USE flags
    """
    if profile_name not in PROFILES:
        return []

    profile = PROFILES[profile_name]
    flags = []

    # Get parent flags first
    parent = profile.get("parent")
    if parent:
        flags.extend(get_profile_use_flags(parent))

    # Add this profile's flags
    flags.extend(profile.get("use_defaults", []))

    return flags


# Re-export commonly used functions for convenience
# These are available directly from this module

# Registry functions
query_package = get_package_info
query_versions = get_all_versions
query_stable_versions = get_stable_versions
query_version_status = get_version_status
query_all_packages = list_all_packages
query_by_category = list_packages_by_category

# USE flag functions
query_use_flags = get_all_use_flags
query_use_by_category = get_use_flags_by_category
query_use_description = get_use_flag_description

# Set functions
query_set = get_set_packages
query_set_info = get_set_info
query_all_sets = list_all_sets

# Maintainer functions
query_maintainer = get_maintainer_info
query_maintainer_packages = get_maintainer_packages
