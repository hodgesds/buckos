# Copyright (c) BuckOS Authors. All rights reserved.
# EAPI Versioning System

"""
EAPI Module

This module provides EAPI (Ebuild API) versioning support for safe evolution
of the build macro API in the BuckOS build system.
"""

# Current EAPI version
CURRENT_EAPI = 8

# Minimum supported EAPI
MIN_SUPPORTED_EAPI = 6

# EAPI feature definitions
EAPI_FEATURES = {
    6: {
        "description": "Base functionality with eapply and user patches",
        "features": {
            "eapply": True,
            "eapply_user": True,
            "dohtml": True,
            "hasv": True,
            "hasq": True,
            "usev": True,
            "useq": True,
            "default_src_prepare": True,
            "default_src_configure": True,
            "default_src_compile": True,
            "default_src_install": True,
            "bdepend": False,
            "subslots": False,
            "selective_fetch": False,
            "strict_use": False,
            "sysroot": False,
            "dot_version": True,
        },
        "deprecated_functions": [],
        "banned_functions": [],
    },
    7: {
        "description": "BDEPEND, version functions, and sysroot support",
        "features": {
            "eapply": True,
            "eapply_user": True,
            "dohtml": False,
            "hasv": False,
            "hasq": False,
            "usev": True,
            "useq": False,
            "default_src_prepare": True,
            "default_src_configure": True,
            "default_src_compile": True,
            "default_src_install": True,
            "bdepend": True,
            "subslots": False,
            "selective_fetch": False,
            "strict_use": False,
            "sysroot": True,
            "ver_cut": True,
            "ver_test": True,
            "ver_rs": True,
            "dot_version": True,
        },
        "deprecated_functions": ["dohtml", "hasv", "hasq", "useq"],
        "banned_functions": [],
    },
    8: {
        "description": "Subslots, selective fetch, and strict USE",
        "features": {
            "eapply": True,
            "eapply_user": True,
            "dohtml": False,
            "hasv": False,
            "hasq": False,
            "usev": True,
            "useq": False,
            "default_src_prepare": True,
            "default_src_configure": True,
            "default_src_compile": True,
            "default_src_install": True,
            "bdepend": True,
            "subslots": True,
            "selective_fetch": True,
            "strict_use": True,
            "sysroot": True,
            "ver_cut": True,
            "ver_test": True,
            "ver_rs": True,
            "idepend": True,
            "usex": True,
            "in_iuse": True,
            "eqawarn": True,
            "compressed_patches": True,
            "dot_version": True,
        },
        "deprecated_functions": ["dohtml", "hasv", "hasq", "useq"],
        "banned_functions": ["dohtml", "hasv", "hasq", "useq"],
    },
}

# Default phase implementations for each EAPI
DEFAULT_PHASES = {
    8: {
        "src_unpack": """
            if [ -n "$A" ]; then
                unpack $A
            fi
        """,
        "src_prepare": """
            if [ -d "${WORKDIR}/patches" ]; then
                eapply "${WORKDIR}/patches"
            fi
            eapply_user
        """,
        "src_configure": """
            if [ -x "${ECONF_SOURCE:-.}/configure" ]; then
                econf
            fi
        """,
        "src_compile": """
            if [ -f Makefile ] || [ -f GNUmakefile ] || [ -f makefile ]; then
                emake
            fi
        """,
        "src_install": """
            if [ -f Makefile ] || [ -f GNUmakefile ] || [ -f makefile ]; then
                emake DESTDIR="${D}" install
            fi
        """,
        "src_test": """
            if [ -f Makefile ] || [ -f GNUmakefile ] || [ -f makefile ]; then
                emake check || emake test
            fi
        """,
    },
}


def validate_eapi(eapi):
    """
    Validate that an EAPI version is supported

    Args:
        eapi: EAPI version number

    Returns:
        True if EAPI is supported
    """
    return eapi >= MIN_SUPPORTED_EAPI and eapi <= CURRENT_EAPI


def require_eapi(min_eapi):
    """
    Require a minimum EAPI version

    Args:
        min_eapi: Minimum required EAPI

    Fails:
        If current EAPI is less than required
    """
    if CURRENT_EAPI < min_eapi:
        fail("EAPI {} required, but current EAPI is {}".format(min_eapi, CURRENT_EAPI))


def eapi_has_feature(feature, eapi = None):
    """
    Check if a feature is available in an EAPI

    Args:
        feature: Feature name
        eapi: EAPI version (defaults to current)

    Returns:
        True if feature is available
    """
    if eapi == None:
        eapi = CURRENT_EAPI

    if eapi not in EAPI_FEATURES:
        return False

    features = EAPI_FEATURES[eapi].get("features", {})
    return features.get(feature, False)


def get_eapi_features(eapi):
    """
    Get all features for an EAPI version

    Args:
        eapi: EAPI version

    Returns:
        Dict of feature name to availability
    """
    if eapi not in EAPI_FEATURES:
        return {}

    return EAPI_FEATURES[eapi].get("features", {})


def is_deprecated(function_name, eapi = None):
    """
    Check if a function is deprecated in an EAPI

    Args:
        function_name: Function name
        eapi: EAPI version (defaults to current)

    Returns:
        True if function is deprecated
    """
    if eapi == None:
        eapi = CURRENT_EAPI

    if eapi not in EAPI_FEATURES:
        return False

    deprecated = EAPI_FEATURES[eapi].get("deprecated_functions", [])
    return function_name in deprecated


def is_banned(function_name, eapi = None):
    """
    Check if a function is banned in an EAPI

    Args:
        function_name: Function name
        eapi: EAPI version (defaults to current)

    Returns:
        True if function is banned
    """
    if eapi == None:
        eapi = CURRENT_EAPI

    if eapi not in EAPI_FEATURES:
        return False

    banned = EAPI_FEATURES[eapi].get("banned_functions", [])
    return function_name in banned


def get_default_phase(phase, eapi = None):
    """
    Get default implementation for a build phase

    Args:
        phase: Phase name (e.g., "src_compile")
        eapi: EAPI version (defaults to current)

    Returns:
        Shell script for default phase implementation
    """
    if eapi == None:
        eapi = CURRENT_EAPI

    if eapi not in DEFAULT_PHASES:
        return ""

    return DEFAULT_PHASES[eapi].get(phase, "")


def migration_guide(from_eapi, to_eapi):
    """
    Get migration guide between EAPI versions

    Args:
        from_eapi: Source EAPI version
        to_eapi: Target EAPI version

    Returns:
        List of migration steps
    """
    steps = []

    # EAPI 6 -> 7
    if from_eapi <= 6 and to_eapi >= 7:
        steps.extend([
            "Convert DEPEND to BDEPEND for build-time only dependencies",
            "Replace dohtml with dodoc for HTML documentation",
            "Replace hasv/hasq with has_version/best_version",
            "Replace useq with use",
            "Update to use ver_cut/ver_test/ver_rs for version manipulation",
            "Utilize SYSROOT for cross-compilation scenarios",
        ])

    # EAPI 7 -> 8
    if from_eapi <= 7 and to_eapi >= 8:
        steps.extend([
            "Add subslots for packages with ABI-sensitive libraries",
            "Use IDEPEND for install-time dependencies",
            "Remove any remaining uses of banned functions",
            "Update USE flag handling for strict USE semantics",
            "Consider using selective fetch for large sources",
            "Support compressed patches (.xz, .zst)",
        ])

    return steps


def check_eapi_compatibility(package_eapi, system_eapi = None):
    """
    Check compatibility between package EAPI and system EAPI

    Args:
        package_eapi: Package's EAPI
        system_eapi: System's EAPI (defaults to current)

    Returns:
        Compatibility result dict
    """
    if system_eapi == None:
        system_eapi = CURRENT_EAPI

    result = {
        "compatible": True,
        "warnings": [],
        "errors": [],
    }

    # Check if package EAPI is supported
    if not validate_eapi(package_eapi):
        result["compatible"] = False
        result["errors"].append(
            "Package EAPI {} is not supported (supported: {}-{})".format(
                package_eapi, MIN_SUPPORTED_EAPI, CURRENT_EAPI
            )
        )
        return result

    # Check for deprecated features
    if package_eapi < system_eapi:
        deprecated = EAPI_FEATURES[system_eapi].get("deprecated_functions", [])
        if deprecated:
            result["warnings"].append(
                "Consider updating to EAPI {} to avoid deprecated functions: {}".format(
                    system_eapi, ", ".join(deprecated)
                )
            )

    # Check for banned features if package uses old EAPI
    if package_eapi < 8:
        if eapi_has_feature("subslots", 8):
            result["warnings"].append(
                "Consider EAPI 8 to support subslots for ABI tracking"
            )

    return result


def get_eapi_description(eapi):
    """
    Get description for an EAPI version

    Args:
        eapi: EAPI version

    Returns:
        Description string
    """
    if eapi not in EAPI_FEATURES:
        return "Unknown EAPI"

    return EAPI_FEATURES[eapi].get("description", "")


def list_supported_eapis():
    """
    List all supported EAPI versions

    Returns:
        List of supported EAPI version numbers
    """
    return list(range(MIN_SUPPORTED_EAPI, CURRENT_EAPI + 1))


def get_eapi_diff(from_eapi, to_eapi):
    """
    Get differences between two EAPI versions

    Args:
        from_eapi: Source EAPI version
        to_eapi: Target EAPI version

    Returns:
        Dict with added, removed, and changed features
    """
    if from_eapi not in EAPI_FEATURES or to_eapi not in EAPI_FEATURES:
        return {"added": [], "removed": [], "changed": []}

    from_features = EAPI_FEATURES[from_eapi].get("features", {})
    to_features = EAPI_FEATURES[to_eapi].get("features", {})

    added = []
    removed = []
    changed = []

    all_features = set(from_features.keys()) | set(to_features.keys())

    for feature in all_features:
        from_val = from_features.get(feature)
        to_val = to_features.get(feature)

        if from_val == None and to_val != None:
            added.append(feature)
        elif from_val != None and to_val == None:
            removed.append(feature)
        elif from_val != to_val:
            changed.append(feature)

    return {
        "added": sorted(added),
        "removed": sorted(removed),
        "changed": sorted(changed),
    }


# Helper functions for version manipulation (EAPI 7+)

def ver_cut(range_spec, version):
    """
    Cut version components

    Args:
        range_spec: Range specification (e.g., "1-2", "3")
        version: Version string

    Returns:
        Cut version string
    """
    if not eapi_has_feature("ver_cut"):
        fail("ver_cut requires EAPI 7 or later")

    # Parse version components
    components = []
    current = ""
    for char in version:
        if char in ".-_":
            if current:
                components.append(current)
            components.append(char)
            current = ""
        else:
            current += char
    if current:
        components.append(current)

    # Parse range
    if "-" in range_spec:
        parts = range_spec.split("-")
        start = int(parts[0]) if parts[0] else 1
        end = int(parts[1]) if parts[1] else len(components)
    else:
        start = int(range_spec)
        end = start

    # Convert to zero-based indices for version parts only
    result = []
    part_index = 0
    for comp in components:
        if comp not in ".-_":
            part_index += 1
            if part_index >= start and part_index <= end:
                result.append(comp)
        elif part_index >= start and part_index < end:
            result.append(comp)

    return "".join(result)


def ver_test(v1, op, v2):
    """
    Compare version strings

    Args:
        v1: First version
        op: Comparison operator (-eq, -ne, -lt, -le, -gt, -ge)
        v2: Second version

    Returns:
        True if comparison is satisfied
    """
    if not eapi_has_feature("ver_test"):
        fail("ver_test requires EAPI 7 or later")

    # Import from versions.bzl
    # This is a simplified implementation
    cmp = _ver_compare(v1, v2)

    if op == "-eq":
        return cmp == 0
    elif op == "-ne":
        return cmp != 0
    elif op == "-lt":
        return cmp < 0
    elif op == "-le":
        return cmp <= 0
    elif op == "-gt":
        return cmp > 0
    elif op == "-ge":
        return cmp >= 0

    return False


def _ver_compare(v1, v2):
    """Simple version comparison helper"""
    p1 = _parse_version_simple(v1)
    p2 = _parse_version_simple(v2)

    for i in range(max(len(p1), len(p2))):
        c1 = p1[i] if i < len(p1) else 0
        c2 = p2[i] if i < len(p2) else 0
        if c1 < c2:
            return -1
        elif c1 > c2:
            return 1
    return 0


def _parse_version_simple(version):
    """Parse version into numeric components"""
    components = []
    current = ""
    for char in version:
        if char.isdigit():
            current += char
        else:
            if current:
                components.append(int(current))
                current = ""
    if current:
        components.append(int(current))
    return components
