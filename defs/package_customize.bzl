# Copyright (c) BuckOS Authors. All rights reserved.
# Package Customization

"""
Package Customization Module

This module allows users and distributions to customize package builds
through configuration overlays, patches, and environment overrides.
"""

# User customization storage
_CUSTOMIZATIONS = {}


def package_config(
        profile = None,
        use_flags = {},
        env_overrides = {},
        package_patches = {},
        build_options = {}):
    """
    Create a package customization configuration

    Args:
        profile: System profile name
        use_flags: Dict of package name to USE flags
        env_overrides: Dict of package name to environment overrides
        package_patches: Dict of package name to patch list
        build_options: Dict of package name to build options

    Returns:
        Customization configuration dict
    """
    return {
        "profile": profile,
        "use_flags": use_flags,
        "env_overrides": env_overrides,
        "package_patches": package_patches,
        "build_options": build_options,
    }


def apply_customization(base_config, customization):
    """
    Apply customization to a base configuration

    Args:
        base_config: Base package configuration
        customization: Customization to apply

    Returns:
        Modified configuration
    """
    result = dict(base_config)
    pkg_name = base_config.get("name", "")

    # Apply USE flag overrides
    if pkg_name in customization.get("use_flags", {}):
        pkg_flags = customization["use_flags"][pkg_name]
        current_flags = set(result.get("use_defaults", []))

        for flag in pkg_flags:
            if flag.startswith("-"):
                current_flags.discard(flag[1:])
            else:
                current_flags.add(flag)

        result["use_defaults"] = sorted(current_flags)

    # Apply environment overrides
    if pkg_name in customization.get("env_overrides", {}):
        env = customization["env_overrides"][pkg_name]
        if "env" not in result:
            result["env"] = {}
        result["env"].update(env)

    # Apply patches
    if pkg_name in customization.get("package_patches", {}):
        patches = customization["package_patches"][pkg_name]
        if "patches" not in result:
            result["patches"] = []
        result["patches"].extend(patches)

    # Apply build options
    if pkg_name in customization.get("build_options", {}):
        options = customization["build_options"][pkg_name]
        result.update(options)

    return result


def register_customization(name, customization):
    """
    Register a named customization

    Args:
        name: Customization name
        customization: Customization configuration
    """
    _CUSTOMIZATIONS[name] = customization


def get_customization(name):
    """
    Get a registered customization

    Args:
        name: Customization name

    Returns:
        Customization configuration or None
    """
    return _CUSTOMIZATIONS.get(name)


def list_customizations():
    """
    List all registered customizations

    Returns:
        List of customization names
    """
    return sorted(_CUSTOMIZATIONS.keys())


def merge_customizations(*customizations):
    """
    Merge multiple customizations (later ones override)

    Args:
        *customizations: Customization configurations

    Returns:
        Merged customization
    """
    result = {
        "profile": None,
        "use_flags": {},
        "env_overrides": {},
        "package_patches": {},
        "build_options": {},
    }

    for custom in customizations:
        if custom.get("profile"):
            result["profile"] = custom["profile"]

        for pkg, flags in custom.get("use_flags", {}).items():
            if pkg not in result["use_flags"]:
                result["use_flags"][pkg] = []
            result["use_flags"][pkg].extend(flags)

        for pkg, env in custom.get("env_overrides", {}).items():
            if pkg not in result["env_overrides"]:
                result["env_overrides"][pkg] = {}
            result["env_overrides"][pkg].update(env)

        for pkg, patches in custom.get("package_patches", {}).items():
            if pkg not in result["package_patches"]:
                result["package_patches"][pkg] = []
            result["package_patches"][pkg].extend(patches)

        for pkg, options in custom.get("build_options", {}).items():
            if pkg not in result["build_options"]:
                result["build_options"][pkg] = {}
            result["build_options"][pkg].update(options)

    return result


# Predefined customization profiles

HARDENED_CUSTOMIZATION = package_config(
    profile = "hardened",
    use_flags = {
        "glibc": ["hardened", "ssp", "pie"],
        "gcc": ["hardened", "ssp", "pie"],
        "openssh": ["hardened"],
    },
    env_overrides = {
        "*": {
            "CFLAGS": "-O2 -pipe -fstack-protector-strong -fPIE",
            "LDFLAGS": "-Wl,-z,relro -Wl,-z,now -pie",
        },
    },
    package_patches = {
        "glibc": ["//patches/profiles/hardened:glibc-hardened.patch"],
    },
)

MUSL_CUSTOMIZATION = package_config(
    profile = "musl",
    use_flags = {
        "*": ["-glibc", "musl", "static-libs"],
    },
    env_overrides = {
        "*": {
            "CHOST": "x86_64-pc-linux-musl",
        },
    },
    package_patches = {
        "python": ["//patches/profiles/musl:python-musl.patch"],
        "rust": ["//patches/profiles/musl:rust-musl.patch"],
    },
)

CROSS_COMPILE_CUSTOMIZATION = package_config(
    build_options = {
        "*": {
            "cross_compile": True,
        },
    },
)


def get_package_env(package_name, customization = None):
    """
    Get environment variables for a package build

    Args:
        package_name: Package name
        customization: Optional customization to apply

    Returns:
        Dict of environment variables
    """
    env = {}

    if customization:
        # Apply wildcard overrides
        if "*" in customization.get("env_overrides", {}):
            env.update(customization["env_overrides"]["*"])

        # Apply package-specific overrides
        if package_name in customization.get("env_overrides", {}):
            env.update(customization["env_overrides"][package_name])

    return env


def get_package_patches(package_name, customization = None):
    """
    Get patches for a package

    Args:
        package_name: Package name
        customization: Optional customization to apply

    Returns:
        List of patch targets
    """
    patches = []

    if customization:
        if package_name in customization.get("package_patches", {}):
            patches.extend(customization["package_patches"][package_name])

    return patches
