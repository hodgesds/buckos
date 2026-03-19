# Copyright (c) BuckOS Authors. All rights reserved.
# Package Set Definitions

"""
Package Sets Module

This module defines package collections (sets) for easy installation of
related packages. Sets can inherit from other sets and be combined.
"""

# System set definitions
SYSTEM_SETS = {
    "minimal": {
        "description": "Minimal bootable system",
        "inherits": None,
        "packages": [
            "linux/core/bash",
            "linux/core/busybox",
            "linux/core/musl",
            "linux/core/linux-headers",
        ],
    },
    "server": {
        "description": "Server base system",
        "inherits": "minimal",
        "packages": [
            "linux/network/openssl",
            "linux/core/zlib",
            "linux/core/glibc",
            "linux/network/openssh",
            "linux/system/init/systemd",
        ],
    },
    "desktop": {
        "description": "Desktop base system",
        "inherits": "server",
        "packages": [
            "linux/graphics/mesa",
            "linux/graphics/xorg-server",
            "linux/audio/pipewire",
            "linux/desktop/dbus",
        ],
    },
    "developer": {
        "description": "Development environment",
        "inherits": "desktop",
        "packages": [
            "linux/dev-tools/gcc",
            "linux/dev-tools/clang",
            "linux/dev-tools/cmake",
            "linux/dev-tools/git",
            "linux/dev-tools/gdb",
        ],
    },
    "hardened": {
        "description": "Security-hardened system",
        "inherits": "server",
        "packages": [
            "linux/security/audit",
            "linux/security/libcap",
        ],
    },
}

# Task-specific sets
TASK_SETS = {
    "web-server": {
        "description": "Web server packages",
        "packages": [
            "linux/www/servers/nginx",
            "linux/www/servers/apache",
            "linux/network/curl",
        ],
    },
    "database": {
        "description": "Database packages",
        "packages": [
            "linux/database/postgresql",
            "linux/database/mariadb",
            "linux/database/sqlite",
        ],
    },
    "container": {
        "description": "Container runtime packages",
        "packages": [
            "linux/app-containers/docker",
            "linux/app-containers/podman",
            "linux/app-containers/containerd",
        ],
    },
    "virtualization": {
        "description": "Virtualization packages",
        "packages": [
            "linux/app-emulation/qemu",
            "linux/app-emulation/libvirt",
        ],
    },
    "monitoring": {
        "description": "System monitoring packages",
        "packages": [
            "linux/monitoring/prometheus",
            "linux/monitoring/grafana",
            "linux/monitoring/node-exporter",
        ],
    },
}

# Desktop environment sets
DESKTOP_SETS = {
    "gnome": {
        "description": "GNOME desktop environment",
        "packages": [
            "linux/desktop/gnome-shell",
            "linux/desktop/gnome-terminal",
            "linux/desktop/nautilus",
            "linux/desktop/gnome-control-center",
        ],
    },
    "kde": {
        "description": "KDE Plasma desktop environment",
        "packages": [
            "linux/desktop/plasma-desktop",
            "linux/desktop/konsole",
            "linux/desktop/dolphin",
            "linux/desktop/systemsettings",
        ],
    },
    "xfce": {
        "description": "Xfce desktop environment",
        "packages": [
            "linux/desktop/xfce4-panel",
            "linux/desktop/xfce4-terminal",
            "linux/desktop/thunar",
            "linux/desktop/xfce4-settings",
        ],
    },
    "sway": {
        "description": "Sway Wayland compositor",
        "packages": [
            "linux/desktop/sway",
            "linux/desktop/foot",
            "linux/desktop/waybar",
            "linux/desktop/wofi",
        ],
    },
}


def get_set_packages(set_name):
    """
    Get packages in a set, including inherited packages

    Args:
        set_name: Name of the set

    Returns:
        List of package identifiers
    """

    # Check all set collections
    set_info = None
    if set_name in SYSTEM_SETS:
        set_info = SYSTEM_SETS[set_name]
    elif set_name in TASK_SETS:
        set_info = TASK_SETS[set_name]
    elif set_name in DESKTOP_SETS:
        set_info = DESKTOP_SETS[set_name]

    if not set_info:
        return []

    packages = list(set_info.get("packages", []))

    # Handle inheritance
    inherits = set_info.get("inherits")
    if inherits:
        inherited = get_set_packages(inherits)
        packages = inherited + packages

    # Remove duplicates while preserving order
    seen = {}
    result = []
    for pkg in packages:
        if pkg not in seen:
            seen[pkg] = True
            result.append(pkg)

    return result


def get_set_info(set_name):
    """
    Get metadata for a set

    Args:
        set_name: Name of the set

    Returns:
        Set info dict or None
    """
    if set_name in SYSTEM_SETS:
        info = dict(SYSTEM_SETS[set_name])
        info["type"] = "system"
        return info
    elif set_name in TASK_SETS:
        info = dict(TASK_SETS[set_name])
        info["type"] = "task"
        return info
    elif set_name in DESKTOP_SETS:
        info = dict(DESKTOP_SETS[set_name])
        info["type"] = "desktop"
        return info
    return None


def list_all_sets():
    """
    List all available sets

    Returns:
        List of set names
    """
    all_sets = []
    all_sets.extend(SYSTEM_SETS.keys())
    all_sets.extend(TASK_SETS.keys())
    all_sets.extend(DESKTOP_SETS.keys())
    return sorted(all_sets)


def list_sets_by_type(set_type):
    """
    List sets of a specific type

    Args:
        set_type: Type of set (system, task, desktop)

    Returns:
        List of set names
    """
    if set_type == "system":
        return sorted(SYSTEM_SETS.keys())
    elif set_type == "task":
        return sorted(TASK_SETS.keys())
    elif set_type == "desktop":
        return sorted(DESKTOP_SETS.keys())
    return []


def union_sets(*set_names):
    """
    Get union of multiple sets

    Args:
        *set_names: Names of sets to union

    Returns:
        List of unique packages
    """
    all_packages = []
    for set_name in set_names:
        all_packages.extend(get_set_packages(set_name))

    # Remove duplicates while preserving order
    seen = {}
    result = []
    for pkg in all_packages:
        if pkg not in seen:
            seen[pkg] = True
            result.append(pkg)

    return result


def intersection_sets(*set_names):
    """
    Get intersection of multiple sets

    Args:
        *set_names: Names of sets to intersect

    Returns:
        List of packages in all sets
    """
    if not set_names:
        return []

    # Start with first set
    result = set(get_set_packages(set_names[0]))

    # Intersect with remaining sets
    for set_name in set_names[1:]:
        result = result & set(get_set_packages(set_name))

    return sorted(result)


def difference_sets(set1, set2):
    """
    Get packages in set1 but not in set2

    Args:
        set1: First set name
        set2: Second set name

    Returns:
        List of packages
    """
    packages1 = set(get_set_packages(set1))
    packages2 = set(get_set_packages(set2))
    return sorted(packages1 - packages2)


def compare_sets(set1, set2):
    """
    Compare two sets

    Args:
        set1: First set name
        set2: Second set name

    Returns:
        Dict with added, removed, and common packages
    """
    packages1 = set(get_set_packages(set1))
    packages2 = set(get_set_packages(set2))

    return {
        "added": sorted(packages2 - packages1),
        "removed": sorted(packages1 - packages2),
        "common": sorted(packages1 & packages2),
    }


def system_set(name, packages, inherits = None, description = ""):
    """
    Define a system set

    Args:
        name: Set name
        packages: List of packages
        inherits: Parent set to inherit from
        description: Set description
    """
    SYSTEM_SETS[name] = {
        "description": description,
        "inherits": inherits,
        "packages": packages,
    }


def task_set(name, packages, description = ""):
    """
    Define a task set

    Args:
        name: Set name
        packages: List of packages
        description: Set description
    """
    TASK_SETS[name] = {
        "description": description,
        "packages": packages,
    }


def desktop_set(name, packages, description = ""):
    """
    Define a desktop set

    Args:
        name: Set name
        packages: List of packages
        description: Set description
    """
    DESKTOP_SETS[name] = {
        "description": description,
        "packages": packages,
    }
