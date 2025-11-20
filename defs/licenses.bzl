# Copyright (c) BuckOS Authors. All rights reserved.
# License Management System

"""
License Module

This module provides license tracking, validation, and compliance checking
for the BuckOS build system.
"""

# License definitions with metadata
LICENSES = {
    # Permissive Licenses
    "MIT": {
        "name": "MIT License",
        "url": "https://opensource.org/licenses/MIT",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "BSD": {
        "name": "BSD License",
        "url": "https://opensource.org/licenses/BSD-3-Clause",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "BSD-2": {
        "name": "BSD 2-Clause License",
        "url": "https://opensource.org/licenses/BSD-2-Clause",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "BSD-3": {
        "name": "BSD 3-Clause License",
        "url": "https://opensource.org/licenses/BSD-3-Clause",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "ISC": {
        "name": "ISC License",
        "url": "https://opensource.org/licenses/ISC",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "Apache-2.0": {
        "name": "Apache License 2.0",
        "url": "https://www.apache.org/licenses/LICENSE-2.0",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "Zlib": {
        "name": "zlib License",
        "url": "https://opensource.org/licenses/Zlib",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "Unlicense": {
        "name": "The Unlicense",
        "url": "https://unlicense.org/",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "WTFPL-2": {
        "name": "WTFPL Version 2",
        "url": "http://www.wtfpl.net/",
        "free": True,
        "osi": False,
        "copyleft": False,
    },
    "0BSD": {
        "name": "Zero-Clause BSD",
        "url": "https://opensource.org/licenses/0BSD",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "CC0-1.0": {
        "name": "Creative Commons Zero v1.0 Universal",
        "url": "https://creativecommons.org/publicdomain/zero/1.0/",
        "free": True,
        "osi": False,
        "copyleft": False,
    },

    # GPL Family
    "GPL-2": {
        "name": "GNU General Public License v2",
        "url": "https://www.gnu.org/licenses/old-licenses/gpl-2.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "GPL-2+": {
        "name": "GNU General Public License v2 or later",
        "url": "https://www.gnu.org/licenses/old-licenses/gpl-2.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "GPL-3": {
        "name": "GNU General Public License v3",
        "url": "https://www.gnu.org/licenses/gpl-3.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "GPL-3+": {
        "name": "GNU General Public License v3 or later",
        "url": "https://www.gnu.org/licenses/gpl-3.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "LGPL-2": {
        "name": "GNU Lesser General Public License v2",
        "url": "https://www.gnu.org/licenses/old-licenses/lgpl-2.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "LGPL-2+": {
        "name": "GNU Lesser General Public License v2 or later",
        "url": "https://www.gnu.org/licenses/old-licenses/lgpl-2.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "LGPL-2.1": {
        "name": "GNU Lesser General Public License v2.1",
        "url": "https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "LGPL-2.1+": {
        "name": "GNU Lesser General Public License v2.1 or later",
        "url": "https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "LGPL-3": {
        "name": "GNU Lesser General Public License v3",
        "url": "https://www.gnu.org/licenses/lgpl-3.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "LGPL-3+": {
        "name": "GNU Lesser General Public License v3 or later",
        "url": "https://www.gnu.org/licenses/lgpl-3.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "AGPL-3": {
        "name": "GNU Affero General Public License v3",
        "url": "https://www.gnu.org/licenses/agpl-3.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "AGPL-3+": {
        "name": "GNU Affero General Public License v3 or later",
        "url": "https://www.gnu.org/licenses/agpl-3.0.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },

    # Other Copyleft
    "MPL-2.0": {
        "name": "Mozilla Public License 2.0",
        "url": "https://www.mozilla.org/en-US/MPL/2.0/",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "EPL-1.0": {
        "name": "Eclipse Public License 1.0",
        "url": "https://www.eclipse.org/legal/epl-v10.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "EPL-2.0": {
        "name": "Eclipse Public License 2.0",
        "url": "https://www.eclipse.org/legal/epl-v20.html",
        "free": True,
        "osi": True,
        "copyleft": True,
    },
    "CDDL": {
        "name": "Common Development and Distribution License",
        "url": "https://opensource.org/licenses/CDDL-1.0",
        "free": True,
        "osi": True,
        "copyleft": True,
    },

    # Creative Commons
    "CC-BY-3.0": {
        "name": "Creative Commons Attribution 3.0",
        "url": "https://creativecommons.org/licenses/by/3.0/",
        "free": True,
        "osi": False,
        "copyleft": False,
    },
    "CC-BY-4.0": {
        "name": "Creative Commons Attribution 4.0",
        "url": "https://creativecommons.org/licenses/by/4.0/",
        "free": True,
        "osi": False,
        "copyleft": False,
    },
    "CC-BY-SA-3.0": {
        "name": "Creative Commons Attribution-ShareAlike 3.0",
        "url": "https://creativecommons.org/licenses/by-sa/3.0/",
        "free": True,
        "osi": False,
        "copyleft": True,
    },
    "CC-BY-SA-4.0": {
        "name": "Creative Commons Attribution-ShareAlike 4.0",
        "url": "https://creativecommons.org/licenses/by-sa/4.0/",
        "free": True,
        "osi": False,
        "copyleft": True,
    },

    # Documentation
    "FDL-1.1": {
        "name": "GNU Free Documentation License v1.1",
        "url": "https://www.gnu.org/licenses/old-licenses/fdl-1.1.html",
        "free": True,
        "osi": False,
        "copyleft": True,
    },
    "FDL-1.2": {
        "name": "GNU Free Documentation License v1.2",
        "url": "https://www.gnu.org/licenses/old-licenses/fdl-1.2.html",
        "free": True,
        "osi": False,
        "copyleft": True,
    },
    "FDL-1.3": {
        "name": "GNU Free Documentation License v1.3",
        "url": "https://www.gnu.org/licenses/fdl-1.3.html",
        "free": True,
        "osi": False,
        "copyleft": True,
    },

    # Other Free Software
    "Artistic": {
        "name": "Artistic License",
        "url": "https://opensource.org/licenses/Artistic-1.0",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "Artistic-2": {
        "name": "Artistic License 2.0",
        "url": "https://opensource.org/licenses/Artistic-2.0",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "OFL-1.1": {
        "name": "SIL Open Font License 1.1",
        "url": "https://scripts.sil.org/OFL",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "PSF-2": {
        "name": "Python Software Foundation License 2.0",
        "url": "https://www.python.org/psf/license/",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "Ruby": {
        "name": "Ruby License",
        "url": "https://www.ruby-lang.org/en/about/license.txt",
        "free": True,
        "osi": False,
        "copyleft": False,
    },
    "Boost-1.0": {
        "name": "Boost Software License 1.0",
        "url": "https://www.boost.org/LICENSE_1_0.txt",
        "free": True,
        "osi": True,
        "copyleft": False,
    },
    "OpenSSL": {
        "name": "OpenSSL License",
        "url": "https://www.openssl.org/source/license.html",
        "free": True,
        "osi": False,
        "copyleft": False,
    },
    "SSLeay": {
        "name": "Original SSLeay License",
        "url": "https://www.openssl.org/source/license.html",
        "free": True,
        "osi": False,
        "copyleft": False,
    },
    "Unicode-DFS-2016": {
        "name": "Unicode License Agreement - Data Files and Software",
        "url": "https://www.unicode.org/copyright.html",
        "free": True,
        "osi": True,
        "copyleft": False,
    },

    # Non-Free
    "EULA": {
        "name": "End-User License Agreement",
        "url": "",
        "free": False,
        "osi": False,
        "copyleft": False,
    },
    "Proprietary": {
        "name": "Proprietary License",
        "url": "",
        "free": False,
        "osi": False,
        "copyleft": False,
    },
    "NVIDIA": {
        "name": "NVIDIA License",
        "url": "https://www.nvidia.com/content/DriverDownload-March2009/licence.php",
        "free": False,
        "osi": False,
        "copyleft": False,
    },
    "AMD-GPU-PRO-EULA": {
        "name": "AMD GPU-PRO EULA",
        "url": "https://www.amd.com/en/support",
        "free": False,
        "osi": False,
        "copyleft": False,
    },
    "Intel-SDP": {
        "name": "Intel Simplified Software License",
        "url": "https://software.intel.com/license",
        "free": False,
        "osi": False,
        "copyleft": False,
    },

    # Firmware
    "linux-firmware": {
        "name": "Linux Firmware License",
        "url": "https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git",
        "free": False,
        "osi": False,
        "copyleft": False,
    },
    "radeon-ucode": {
        "name": "AMD/ATI Firmware License",
        "url": "https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git",
        "free": False,
        "osi": False,
        "copyleft": False,
    },

    # Special
    "public-domain": {
        "name": "Public Domain",
        "url": "",
        "free": True,
        "osi": False,
        "copyleft": False,
    },
    "all-rights-reserved": {
        "name": "All Rights Reserved",
        "url": "",
        "free": False,
        "osi": False,
        "copyleft": False,
    },
    "unknown": {
        "name": "Unknown License",
        "url": "",
        "free": False,
        "osi": False,
        "copyleft": False,
    },
}

# License groups for ACCEPT_LICENSE
LICENSE_GROUPS = {
    "@FREE": {
        "description": "All free software licenses",
        "licenses": [k for k, v in LICENSES.items() if v.get("free", False)],
    },
    "@OSI-APPROVED": {
        "description": "OSI-approved licenses",
        "licenses": [k for k, v in LICENSES.items() if v.get("osi", False)],
    },
    "@GPL-COMPATIBLE": {
        "description": "GPL-compatible licenses",
        "licenses": [
            "MIT", "BSD", "BSD-2", "BSD-3", "ISC", "0BSD", "Zlib", "Unlicense",
            "CC0-1.0", "public-domain", "LGPL-2.1", "LGPL-2.1+", "LGPL-3", "LGPL-3+",
        ],
    },
    "@COPYLEFT": {
        "description": "Copyleft licenses",
        "licenses": [k for k, v in LICENSES.items() if v.get("copyleft", False)],
    },
    "@PERMISSIVE": {
        "description": "Permissive licenses",
        "licenses": [
            "MIT", "BSD", "BSD-2", "BSD-3", "ISC", "Apache-2.0", "Zlib",
            "Unlicense", "0BSD", "CC0-1.0", "Boost-1.0", "PSF-2",
        ],
    },
    "@BINARY-REDISTRIBUTABLE": {
        "description": "Licenses allowing binary redistribution",
        "licenses": [k for k, v in LICENSES.items() if v.get("free", False)] + [
            "NVIDIA", "AMD-GPU-PRO-EULA", "Intel-SDP", "linux-firmware", "radeon-ucode",
        ],
    },
    "@FIRMWARE": {
        "description": "Firmware licenses",
        "licenses": ["linux-firmware", "radeon-ucode"],
    },
    "@EULA": {
        "description": "End-user license agreements",
        "licenses": ["EULA", "NVIDIA", "AMD-GPU-PRO-EULA", "Intel-SDP"],
    },
}

# Default license acceptance configurations
DEFAULT_ACCEPT_LICENSE = ["@FREE"]
SERVER_ACCEPT_LICENSE = ["@FREE", "@FIRMWARE"]
DESKTOP_ACCEPT_LICENSE = ["@FREE", "@FIRMWARE", "@BINARY-REDISTRIBUTABLE"]
DEVELOPER_ACCEPT_LICENSE = ["*", "-unknown"]


def get_license_info(license_id):
    """
    Get license information

    Args:
        license_id: License identifier

    Returns:
        License info dict or None
    """
    return LICENSES.get(license_id)


def is_free_license(license_id):
    """
    Check if a license is free software

    Args:
        license_id: License identifier

    Returns:
        True if license is free
    """
    info = LICENSES.get(license_id)
    return info.get("free", False) if info else False


def is_osi_approved(license_id):
    """
    Check if a license is OSI approved

    Args:
        license_id: License identifier

    Returns:
        True if OSI approved
    """
    info = LICENSES.get(license_id)
    return info.get("osi", False) if info else False


def is_copyleft(license_id):
    """
    Check if a license is copyleft

    Args:
        license_id: License identifier

    Returns:
        True if copyleft
    """
    info = LICENSES.get(license_id)
    return info.get("copyleft", False) if info else False


def expand_license_group(group):
    """
    Expand a license group to its constituent licenses

    Args:
        group: Group name (e.g., "@FREE")

    Returns:
        List of license identifiers
    """
    if group in LICENSE_GROUPS:
        return LICENSE_GROUPS[group]["licenses"]
    return []


def check_license(license_id, accept_list):
    """
    Check if a license is accepted

    Args:
        license_id: License identifier
        accept_list: List of accepted licenses/groups

    Returns:
        True if license is accepted
    """
    for accept in accept_list:
        # Check for wildcard
        if accept == "*":
            return True

        # Check for negation
        if accept.startswith("-"):
            neg_license = accept[1:]
            if neg_license == license_id:
                return False
            if neg_license.startswith("@"):
                if license_id in expand_license_group(neg_license):
                    return False
            continue

        # Check for group
        if accept.startswith("@"):
            if license_id in expand_license_group(accept):
                return True
        # Direct match
        elif accept == license_id:
            return True

    return False


def parse_license_expression(expression):
    """
    Parse a license expression (e.g., "GPL-2 || MIT")

    Args:
        expression: License expression string

    Returns:
        Parsed expression dict
    """
    expression = expression.strip()

    # Check for OR expression
    if " || " in expression:
        parts = [p.strip() for p in expression.split(" || ")]
        return {
            "type": "or",
            "licenses": parts,
        }

    # Check for AND expression
    if " && " in expression:
        parts = [p.strip() for p in expression.split(" && ")]
        return {
            "type": "and",
            "licenses": parts,
        }

    # Check for conditional
    if "?" in expression:
        parts = expression.split("?")
        return {
            "type": "conditional",
            "condition": parts[0].strip(),
            "license": parts[1].strip() if len(parts) > 1 else "",
        }

    # Single license
    return {
        "type": "single",
        "licenses": [expression],
    }


def check_license_expression(expression, accept_list):
    """
    Check if a license expression is satisfied

    Args:
        expression: License expression string
        accept_list: List of accepted licenses/groups

    Returns:
        True if expression is satisfied
    """
    parsed = parse_license_expression(expression)

    if parsed["type"] == "or":
        # Any license in OR must be accepted
        for license_id in parsed["licenses"]:
            if check_license(license_id, accept_list):
                return True
        return False

    if parsed["type"] == "and":
        # All licenses in AND must be accepted
        for license_id in parsed["licenses"]:
            if not check_license(license_id, accept_list):
                return False
        return True

    if parsed["type"] == "single":
        return check_license(parsed["licenses"][0], accept_list)

    return False


def generate_license_report(packages):
    """
    Generate a license report for a list of packages

    Args:
        packages: List of package info dicts with 'name' and 'license' keys

    Returns:
        License report dict
    """
    by_license = {}
    free_count = 0
    non_free_count = 0

    for pkg in packages:
        license_id = pkg.get("license", "unknown")

        if license_id not in by_license:
            by_license[license_id] = []
        by_license[license_id].append(pkg.get("name", "unknown"))

        if is_free_license(license_id):
            free_count += 1
        else:
            non_free_count += 1

    return {
        "by_license": by_license,
        "free_count": free_count,
        "non_free_count": non_free_count,
        "total_count": len(packages),
    }


def list_all_licenses():
    """
    List all known licenses

    Returns:
        List of license identifiers
    """
    return sorted(LICENSES.keys())


def list_license_groups():
    """
    List all license groups

    Returns:
        List of group names
    """
    return sorted(LICENSE_GROUPS.keys())


def get_license_group_info(group):
    """
    Get information about a license group

    Args:
        group: Group name

    Returns:
        Group info dict or None
    """
    return LICENSE_GROUPS.get(group)
