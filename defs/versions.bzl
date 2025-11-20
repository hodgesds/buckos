# Copyright (c) BuckOS Authors. All rights reserved.
# Version Management

"""
Version Management Module

This module provides version comparison, selection, and multi-version
package support for the BuckOS build system.
"""

# Version comparison operators
VERSION_OPERATORS = ["=", ">=", ">", "<=", "<", "~>", "*"]


def parse_version(version_str):
    """
    Parse a version string into comparable components

    Args:
        version_str: Version string (e.g., "1.2.3", "1.2.3_p1", "1.2.3-r1")

    Returns:
        Tuple of (major, minor, patch, suffix, revision)
    """
    # Handle revision suffix (-r1, -r2, etc.)
    revision = 0
    if "-r" in version_str:
        parts = version_str.rsplit("-r", 1)
        version_str = parts[0]
        if len(parts) > 1 and parts[1].isdigit():
            revision = int(parts[1])

    # Handle patch level suffix (_p1, _p2, etc.)
    patch_level = 0
    if "_p" in version_str:
        parts = version_str.rsplit("_p", 1)
        version_str = parts[0]
        if len(parts) > 1 and parts[1].isdigit():
            patch_level = int(parts[1])

    # Handle alpha/beta/rc suffixes
    suffix = ""
    for s in ["_alpha", "_beta", "_rc"]:
        if s in version_str:
            idx = version_str.find(s)
            suffix = version_str[idx:]
            version_str = version_str[:idx]
            break

    # Parse numeric components
    components = version_str.split(".")
    major = 0
    minor = 0
    patch = 0

    if len(components) >= 1 and components[0].isdigit():
        major = int(components[0])
    if len(components) >= 2 and components[1].isdigit():
        minor = int(components[1])
    if len(components) >= 3:
        # Handle things like "21" in "5.2.21"
        patch_str = components[2]
        patch_num = ""
        for c in patch_str:
            if c.isdigit():
                patch_num += c
            else:
                break
        if patch_num:
            patch = int(patch_num)

    return (major, minor, patch, suffix, patch_level, revision)


def compare_versions(v1, v2):
    """
    Compare two version strings

    Args:
        v1: First version
        v2: Second version

    Returns:
        -1 if v1 < v2, 0 if equal, 1 if v1 > v2
    """
    p1 = parse_version(v1)
    p2 = parse_version(v2)

    # Compare major, minor, patch
    for i in range(3):
        if p1[i] < p2[i]:
            return -1
        elif p1[i] > p2[i]:
            return 1

    # Compare suffix (alpha < beta < rc < "")
    suffix_order = {"_alpha": 0, "_beta": 1, "_rc": 2, "": 3}
    s1 = suffix_order.get(p1[3], 3)
    s2 = suffix_order.get(p2[3], 3)
    if s1 < s2:
        return -1
    elif s1 > s2:
        return 1

    # Compare patch level
    if p1[4] < p2[4]:
        return -1
    elif p1[4] > p2[4]:
        return 1

    # Compare revision
    if p1[5] < p2[5]:
        return -1
    elif p1[5] > p2[5]:
        return 1

    return 0


def version_satisfies(version, constraint):
    """
    Check if a version satisfies a constraint

    Args:
        version: Version string to check
        constraint: Version constraint (e.g., ">=1.0.0", "~>1.5")

    Returns:
        True if version satisfies constraint
    """
    # No constraint means any version matches
    if not constraint:
        return True

    # Parse operator from constraint
    op = ""
    target = constraint
    for operator in VERSION_OPERATORS:
        if constraint.startswith(operator):
            op = operator
            target = constraint[len(operator):]
            break

    # Handle wildcard
    if op == "*" or target.endswith("*"):
        prefix = target.rstrip("*").rstrip(".")
        return version.startswith(prefix)

    cmp = compare_versions(version, target)

    if op == "" or op == "=":
        return cmp == 0
    elif op == ">=":
        return cmp >= 0
    elif op == ">":
        return cmp > 0
    elif op == "<=":
        return cmp <= 0
    elif op == "<":
        return cmp < 0
    elif op == "~>":
        # Pessimistic version constraint (e.g., ~>1.5 matches 1.5.x)
        p1 = parse_version(version)
        p2 = parse_version(target)
        # Must match major and minor, patch can be anything >= target
        return p1[0] == p2[0] and p1[1] == p2[1] and p1[2] >= p2[2]

    return False


def select_best_version(versions, constraint = None):
    """
    Select the best version from a list

    Args:
        versions: List of version strings
        constraint: Optional version constraint

    Returns:
        Best matching version or None
    """
    if not versions:
        return None

    # Filter by constraint
    if constraint:
        matching = [v for v in versions if version_satisfies(v, constraint)]
    else:
        matching = versions

    if not matching:
        return None

    # Sort and return highest
    sorted_versions = sorted(matching, key = lambda v: parse_version(v), reverse = True)
    return sorted_versions[0]


def multi_version_package(
        name,
        versions,
        default_version,
        description = "",
        homepage = "",
        license = "",
        maintainers = [],
        visibility = ["PUBLIC"]):
    """
    Define a package with multiple installable versions

    Args:
        name: Package name
        versions: Dict mapping version to version info
        default_version: Default version to install
        description: Package description
        homepage: Project homepage
        license: License identifier
        maintainers: List of maintainers
        visibility: Buck visibility
    """

    # Create individual targets for each version
    for version, info in versions.items():
        target_name = "{}-{}".format(name, version.replace(".", "_"))

        metadata = {
            "name": name,
            "version": version,
            "slot": info.get("slot", "0"),
            "status": info.get("status", "stable"),
            "src_uri": info.get("src_uri", ""),
            "sha256": info.get("sha256", ""),
            "description": description,
            "homepage": homepage,
            "license": license,
            "maintainers": maintainers,
        }

        native.genrule(
            name = target_name + "_metadata",
            out = target_name + "_metadata.json",
            cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
            visibility = visibility,
        )

        native.filegroup(
            name = target_name,
            srcs = [":" + target_name + "_metadata"],
            visibility = visibility,
        )

    # Create alias for default version
    default_target = "{}-{}".format(name, default_version.replace(".", "_"))
    native.alias(
        name = name,
        actual = ":" + default_target,
        visibility = visibility,
    )


def get_upgrade_path(package, current_version, target_version, all_versions):
    """
    Calculate safe upgrade path between versions

    Args:
        package: Package name
        current_version: Current installed version
        target_version: Target version
        all_versions: List of all available versions

    Returns:
        List of versions to install in order
    """
    # Sort versions
    sorted_versions = sorted(all_versions, key = lambda v: parse_version(v))

    try:
        current_idx = sorted_versions.index(current_version)
        target_idx = sorted_versions.index(target_version)
    except:
        return [target_version]

    if current_idx >= target_idx:
        # Downgrade - just go directly
        return [target_version]

    # Find breaking changes in path
    path = []
    for i in range(current_idx + 1, target_idx + 1):
        version = sorted_versions[i]
        # In a real implementation, check for breaking changes
        path.append(version)

    return path if path else [target_version]


def version_in_slot(version, slot, versions_info):
    """
    Check if a version is in a specific slot

    Args:
        version: Version string
        slot: Slot identifier
        versions_info: Dict mapping version to info

    Returns:
        True if version is in slot
    """
    if version in versions_info:
        return versions_info[version].get("slot") == slot
    return False


def get_latest_in_slot(slot, versions_info):
    """
    Get latest version in a slot

    Args:
        slot: Slot identifier
        versions_info: Dict mapping version to info

    Returns:
        Latest version in slot or None
    """
    slot_versions = []
    for version, info in versions_info.items():
        if info.get("slot") == slot:
            slot_versions.append(version)

    return select_best_version(slot_versions)
