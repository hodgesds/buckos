# Copyright (c) BuckOS Authors. All rights reserved.
# Maintainer Registry

"""
Maintainer Registry Module

This module defines package maintainers and teams for the BuckOS project.
"""

# Maintainer registry
MAINTAINERS = {
    "core-team": {
        "name": "BuckOS Core Team",
        "email": "core@buckos.dev",
        "github": "buckos-core",
        "description": "Maintains core system packages",
        "packages": [
            "core/bash",
            "core/zlib",
            "core/glibc",
            "core/linux-headers",
            "core/busybox",
            "core/musl",
        ],
    },
    "security-team": {
        "name": "BuckOS Security Team",
        "email": "security@buckos.dev",
        "github": "buckos-security",
        "description": "Maintains security-critical packages",
        "packages": [
            "core/openssl",
            "network/openssh",
            "security/audit",
            "security/libcap",
        ],
    },
    "network-team": {
        "name": "BuckOS Network Team",
        "email": "network@buckos.dev",
        "github": "buckos-network",
        "description": "Maintains networking packages",
        "packages": [
            "network/curl",
            "network/wget",
            "network/iproute2",
        ],
    },
    "toolchain-team": {
        "name": "BuckOS Toolchain Team",
        "email": "toolchain@buckos.dev",
        "github": "buckos-toolchain",
        "description": "Maintains compilers and build tools",
        "packages": [
            "dev-tools/gcc",
            "dev-tools/clang",
            "dev-tools/cmake",
            "dev-tools/ninja",
        ],
    },
    "web-team": {
        "name": "BuckOS Web Team",
        "email": "web@buckos.dev",
        "github": "buckos-web",
        "description": "Maintains web server packages",
        "packages": [
            "www/nginx",
            "www/apache",
        ],
    },
    "kernel-team": {
        "name": "BuckOS Kernel Team",
        "email": "kernel@buckos.dev",
        "github": "buckos-kernel",
        "description": "Maintains kernel and kernel-related packages",
        "packages": [
            "core/linux-headers",
            "kernel/linux",
            "kernel/linux-firmware",
        ],
    },
    "desktop-team": {
        "name": "BuckOS Desktop Team",
        "email": "desktop@buckos.dev",
        "github": "buckos-desktop",
        "description": "Maintains desktop environment packages",
        "packages": [
            "desktop/gnome-shell",
            "desktop/plasma-desktop",
            "desktop/xfce4-panel",
            "desktop/sway",
        ],
    },
}


def get_maintainer_info(maintainer_id):
    """
    Get maintainer information

    Args:
        maintainer_id: Maintainer identifier

    Returns:
        Maintainer info dict or None
    """
    return MAINTAINERS.get(maintainer_id)


def get_maintainer_packages(maintainer_id):
    """
    Get packages maintained by a maintainer

    Args:
        maintainer_id: Maintainer identifier

    Returns:
        List of package identifiers
    """
    if maintainer_id in MAINTAINERS:
        return MAINTAINERS[maintainer_id].get("packages", [])
    return []


def get_package_maintainers(pkg_id):
    """
    Get maintainers for a package

    Args:
        pkg_id: Package identifier

    Returns:
        List of maintainer identifiers
    """
    result = []
    for maintainer_id, info in MAINTAINERS.items():
        if pkg_id in info.get("packages", []):
            result.append(maintainer_id)
    return result


def list_all_maintainers():
    """
    List all maintainers

    Returns:
        List of maintainer identifiers
    """
    return sorted(MAINTAINERS.keys())


def get_maintainer_email(maintainer_id):
    """
    Get maintainer email

    Args:
        maintainer_id: Maintainer identifier

    Returns:
        Email string or None
    """
    if maintainer_id in MAINTAINERS:
        return MAINTAINERS[maintainer_id].get("email")
    return None


def get_maintainer_name(maintainer_id):
    """
    Get maintainer display name

    Args:
        maintainer_id: Maintainer identifier

    Returns:
        Name string or None
    """
    if maintainer_id in MAINTAINERS:
        return MAINTAINERS[maintainer_id].get("name")
    return None


def search_maintainers(pattern):
    """
    Search maintainers by pattern

    Args:
        pattern: Search pattern

    Returns:
        List of matching maintainer identifiers
    """
    result = []
    pattern_lower = pattern.lower()
    for maintainer_id, info in MAINTAINERS.items():
        if pattern_lower in maintainer_id.lower():
            result.append(maintainer_id)
        elif pattern_lower in info.get("name", "").lower():
            result.append(maintainer_id)
        elif pattern_lower in info.get("description", "").lower():
            result.append(maintainer_id)
    return sorted(result)


def register_maintainer(
        maintainer_id,
        name,
        email,
        github = "",
        description = "",
        packages = []):
    """
    Register a new maintainer

    Args:
        maintainer_id: Unique identifier
        name: Display name
        email: Contact email
        github: GitHub username
        description: Description
        packages: List of maintained packages
    """
    MAINTAINERS[maintainer_id] = {
        "name": name,
        "email": email,
        "github": github,
        "description": description,
        "packages": packages,
    }
