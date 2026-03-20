# Copyright (c) BuckOS Authors. All rights reserved.
# Central Package Registry - Source of truth for all package versions and metadata

"""
Package Registry Module

This module serves as the central registry for all package versions, status,
and metadata. Package managers should query this registry for version information.
"""

# Package status constants
STATUS_STABLE = "stable"
STATUS_TESTING = "testing"
STATUS_DEPRECATED = "deprecated"
STATUS_MASKED = "masked"

# Central package registry
# Structure: category/package-name -> version info
PACKAGE_REGISTRY = {
    "linux/core/bash": {
        "default": "5.3",
        "description": "GNU Bourne Again SHell",
        "homepage": "https://www.gnu.org/software/bash/",
        "license": "GPL-3.0",
        "buck_target": "//packages/linux/core/bash:bash",
        "versions": {
            "5.3": {
                "slot": "0",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["core-team"],
    },
    "linux/system/libs/crypto/openssl": {
        "default": "3.6.0",
        "description": "TLS/SSL and crypto library",
        "homepage": "https://www.openssl.org/",
        "license": "Apache-2.0",
        "buck_target": "//packages/linux/system/libs/crypto/openssl:openssl",
        "versions": {
            "3.6.0": {
                "slot": "3",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["security-team"],
    },
    "linux/core/zlib": {
        "default": "1.3.1",
        "description": "Standard compression library",
        "homepage": "https://www.zlib.net/",
        "license": "Zlib",
        "buck_target": "//packages/linux/core/zlib:zlib",
        "versions": {
            "1.3.1": {
                "slot": "0",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["core-team"],
    },
    "linux/core/glibc": {
        "default": "2.42",
        "description": "GNU C Library",
        "homepage": "https://www.gnu.org/software/libc/",
        "license": "LGPL-2.1",
        "buck_target": "//packages/linux/core/glibc:glibc",
        "versions": {
            "2.42": {
                "slot": "2.2",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["toolchain-team"],
    },
    "linux/kernel/buckos-kernel/linux-headers": {
        "default": "6.6",
        "description": "Linux kernel headers",
        "homepage": "https://www.kernel.org/",
        "license": "GPL-2.0",
        "buck_target": "//packages/linux/kernel/buckos-kernel:linux-headers",
        "versions": {
            "6.6": {
                "slot": "0",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["kernel-team"],
    },
    "linux/system/libs/network/curl": {
        "default": "8.10.1",
        "description": "Command line tool and library for transferring data with URLs",
        "homepage": "https://curl.se/",
        "license": "MIT",
        "buck_target": "//packages/linux/system/libs/network/curl:curl",
        "versions": {
            "8.10.1": {
                "slot": "0",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["network-team"],
    },
    "linux/network/openssh": {
        "default": "9.9p1",
        "description": "Port of OpenBSD's free SSH release",
        "homepage": "https://www.openssh.com/",
        "license": "BSD-2-Clause",
        "buck_target": "//packages/linux/network/openssh:openssh",
        "versions": {
            "9.9p1": {
                "slot": "0",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["security-team"],
    },
    "linux/www/servers/nginx": {
        "default": "1.26.2",
        "description": "High performance HTTP and reverse proxy server",
        "homepage": "https://nginx.org/",
        "license": "BSD-2-Clause",
        "buck_target": "//packages/linux/www/servers/nginx:nginx",
        "versions": {
            "1.26.2": {
                "slot": "0",
                "status": STATUS_STABLE,
                "keywords": ["amd64", "arm64"],
                "eapi": "8",
            },
        },
        "maintainers": ["web-team"],
    },
}


def get_default_version(pkg_id):
    """
    Get default version for a package

    Args:
        pkg_id: Package identifier (category/name)

    Returns:
        Default version string or None if not found
    """
    if pkg_id in PACKAGE_REGISTRY:
        return PACKAGE_REGISTRY[pkg_id].get("default")
    return None


def get_all_versions(pkg_id):
    """
    Get all versions for a package

    Args:
        pkg_id: Package identifier (category/name)

    Returns:
        List of version strings, sorted newest first
    """
    if pkg_id in PACKAGE_REGISTRY:
        versions = list(PACKAGE_REGISTRY[pkg_id].get("versions", {}).keys())
        return sorted(versions, reverse = True)
    return []


def get_versions_in_slot(pkg_id, slot):
    """
    Get versions in a specific slot

    Args:
        pkg_id: Package identifier
        slot: Slot identifier

    Returns:
        List of versions in the slot
    """
    if pkg_id not in PACKAGE_REGISTRY:
        return []

    result = []
    for version, info in PACKAGE_REGISTRY[pkg_id].get("versions", {}).items():
        if info.get("slot") == slot:
            result.append(version)
    return sorted(result, reverse = True)


def get_stable_versions(pkg_id):
    """
    Get only stable versions for a package

    Args:
        pkg_id: Package identifier

    Returns:
        List of stable version strings
    """
    if pkg_id not in PACKAGE_REGISTRY:
        return []

    result = []
    for version, info in PACKAGE_REGISTRY[pkg_id].get("versions", {}).items():
        if info.get("status") == STATUS_STABLE:
            result.append(version)
    return sorted(result, reverse = True)


def get_version_status(pkg_id, version):
    """
    Get status for a specific version

    Args:
        pkg_id: Package identifier
        version: Version string

    Returns:
        Status string (stable, testing, deprecated, masked)
    """
    if pkg_id not in PACKAGE_REGISTRY:
        return None

    versions = PACKAGE_REGISTRY[pkg_id].get("versions", {})
    if version in versions:
        return versions[version].get("status")
    return None


def get_version_info(pkg_id, version):
    """
    Get full version info

    Args:
        pkg_id: Package identifier
        version: Version string

    Returns:
        Version info dict or None
    """
    if pkg_id not in PACKAGE_REGISTRY:
        return None

    return PACKAGE_REGISTRY[pkg_id].get("versions", {}).get(version)


def get_package_info(pkg_id):
    """
    Get package metadata

    Args:
        pkg_id: Package identifier

    Returns:
        Package info dict or None
    """
    return PACKAGE_REGISTRY.get(pkg_id)


def list_all_packages():
    """
    List all packages in registry

    Returns:
        List of package identifiers
    """
    return sorted(PACKAGE_REGISTRY.keys())


def list_packages_by_category(category):
    """
    List packages in a category (supports multi-level paths like "linux/core")

    Args:
        category: Category path (e.g., "linux/core", "linux/network")

    Returns:
        List of package names (last path component only)
    """
    result = []
    prefix = category + "/"
    for pkg_id in PACKAGE_REGISTRY.keys():
        if pkg_id.startswith(prefix):
            # The package name is the last path component
            remainder = pkg_id[len(prefix):]
            if "/" not in remainder:
                result.append(remainder)
    return sorted(result)


def get_package_maintainers(pkg_id):
    """
    Get maintainers for a package

    Args:
        pkg_id: Package identifier

    Returns:
        List of maintainer identifiers
    """
    if pkg_id in PACKAGE_REGISTRY:
        return PACKAGE_REGISTRY[pkg_id].get("maintainers", [])
    return []


def search_packages(pattern):
    """
    Search packages by pattern

    Args:
        pattern: Search pattern (substring match)

    Returns:
        List of matching package identifiers
    """
    result = []
    pattern_lower = pattern.lower()
    for pkg_id, info in PACKAGE_REGISTRY.items():
        if pattern_lower in pkg_id.lower():
            result.append(pkg_id)
        elif pattern_lower in info.get("description", "").lower():
            result.append(pkg_id)
    return sorted(result)


def get_all_categories():
    """
    Get all package categories (returns full category paths without package name)

    Returns:
        List of category paths (e.g., "linux/core", "linux/network")
    """
    categories = set()
    for pkg_id in PACKAGE_REGISTRY.keys():
        # Category is everything except the last path component (the package name)
        last_slash = pkg_id.rfind("/")
        if last_slash > 0:
            categories.add(pkg_id[:last_slash])
    return sorted(categories)
