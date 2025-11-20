# Copyright (c) BuckOS Authors. All rights reserved.
# Package Build Definitions

"""
Package Definitions Module

This module provides build rules for defining packages in the BuckOS
build system. It includes rules for standard configure/make packages,
ebuild-style packages, and custom build rules.
"""

load("//defs:use_flags.bzl", "resolve_use_flags")

# Package info provider
PackageInfo = provider(fields = [
    "name",           # string: Package name
    "version",        # string: Package version
    "slot",           # string: Version slot
    "description",    # string: Package description
    "homepage",       # string: Project homepage
    "license",        # string: License identifier
    "src_uri",        # string: Source URI
    "sha256",         # string: SHA256 checksum
    "deps",           # list: Runtime dependencies
    "build_deps",     # list: Build dependencies
    "iuse",           # list: Available USE flags
    "use_enabled",    # list: Currently enabled flags
    "maintainers",    # list: Maintainer identifiers
    "installed_files",# list: Files installed by package
])


def configure_make_package(
        name,
        version,
        src_uri,
        sha256,
        configure_args = [],
        make_args = [],
        install_args = ["DESTDIR=$DESTDIR"],
        pre_configure = "",
        post_install = "",
        deps = [],
        build_deps = [],
        iuse = [],
        use_defaults = [],
        use_deps = {},
        use_configure = {},
        description = "",
        homepage = "",
        license = "",
        slot = "0",
        maintainers = [],
        visibility = ["PUBLIC"]):
    """
    Define a standard configure/make package

    Args:
        name: Package name
        version: Package version
        src_uri: Source download URI
        sha256: Source checksum
        configure_args: Arguments for ./configure
        make_args: Arguments for make
        install_args: Arguments for make install
        pre_configure: Shell commands before configure
        post_install: Shell commands after install
        deps: Runtime dependencies
        build_deps: Build-time dependencies
        iuse: Available USE flags
        use_defaults: Default enabled flags
        use_deps: USE-conditional dependencies
        use_configure: USE-conditional configure args
        description: Package description
        homepage: Project homepage
        license: License identifier
        slot: Package slot
        maintainers: Maintainer identifiers
        visibility: Buck visibility
    """

    metadata = {
        "name": name,
        "version": version,
        "src_uri": src_uri,
        "sha256": sha256,
        "configure_args": configure_args,
        "make_args": make_args,
        "install_args": install_args,
        "pre_configure": pre_configure,
        "post_install": post_install,
        "deps": deps,
        "build_deps": build_deps,
        "iuse": iuse,
        "use_defaults": use_defaults,
        "use_deps": use_deps,
        "use_configure": use_configure,
        "description": description,
        "homepage": homepage,
        "license": license,
        "slot": slot,
        "maintainers": maintainers,
        "build_type": "configure_make",
    }

    native.genrule(
        name = name + "_metadata",
        out = name + "_metadata.json",
        cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
        visibility = visibility,
    )

    native.filegroup(
        name = name,
        srcs = [":" + name + "_metadata"],
        visibility = visibility,
    )


def cmake_package(
        name,
        version,
        src_uri,
        sha256,
        cmake_args = [],
        make_args = [],
        install_args = [],
        pre_configure = "",
        post_install = "",
        deps = [],
        build_deps = [],
        iuse = [],
        use_defaults = [],
        use_deps = {},
        use_cmake = {},
        description = "",
        homepage = "",
        license = "",
        slot = "0",
        maintainers = [],
        visibility = ["PUBLIC"]):
    """
    Define a CMake-based package

    Args:
        name: Package name
        version: Package version
        src_uri: Source download URI
        sha256: Source checksum
        cmake_args: Arguments for cmake
        make_args: Arguments for make/ninja
        install_args: Arguments for install
        pre_configure: Shell commands before cmake
        post_install: Shell commands after install
        deps: Runtime dependencies
        build_deps: Build-time dependencies
        iuse: Available USE flags
        use_defaults: Default enabled flags
        use_deps: USE-conditional dependencies
        use_cmake: USE-conditional cmake args
        description: Package description
        homepage: Project homepage
        license: License identifier
        slot: Package slot
        maintainers: Maintainer identifiers
        visibility: Buck visibility
    """

    metadata = {
        "name": name,
        "version": version,
        "src_uri": src_uri,
        "sha256": sha256,
        "cmake_args": cmake_args,
        "make_args": make_args,
        "install_args": install_args,
        "pre_configure": pre_configure,
        "post_install": post_install,
        "deps": deps,
        "build_deps": build_deps,
        "iuse": iuse,
        "use_defaults": use_defaults,
        "use_deps": use_deps,
        "use_cmake": use_cmake,
        "description": description,
        "homepage": homepage,
        "license": license,
        "slot": slot,
        "maintainers": maintainers,
        "build_type": "cmake",
    }

    native.genrule(
        name = name + "_metadata",
        out = name + "_metadata.json",
        cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
        visibility = visibility,
    )

    native.filegroup(
        name = name,
        srcs = [":" + name + "_metadata"],
        visibility = visibility,
    )


def meson_package(
        name,
        version,
        src_uri,
        sha256,
        meson_args = [],
        ninja_args = [],
        install_args = [],
        pre_configure = "",
        post_install = "",
        deps = [],
        build_deps = [],
        iuse = [],
        use_defaults = [],
        use_deps = {},
        use_meson = {},
        description = "",
        homepage = "",
        license = "",
        slot = "0",
        maintainers = [],
        visibility = ["PUBLIC"]):
    """
    Define a Meson-based package

    Args:
        name: Package name
        version: Package version
        src_uri: Source download URI
        sha256: Source checksum
        meson_args: Arguments for meson
        ninja_args: Arguments for ninja
        install_args: Arguments for install
        pre_configure: Shell commands before meson
        post_install: Shell commands after install
        deps: Runtime dependencies
        build_deps: Build-time dependencies
        iuse: Available USE flags
        use_defaults: Default enabled flags
        use_deps: USE-conditional dependencies
        use_meson: USE-conditional meson args
        description: Package description
        homepage: Project homepage
        license: License identifier
        slot: Package slot
        maintainers: Maintainer identifiers
        visibility: Buck visibility
    """

    metadata = {
        "name": name,
        "version": version,
        "src_uri": src_uri,
        "sha256": sha256,
        "meson_args": meson_args,
        "ninja_args": ninja_args,
        "install_args": install_args,
        "pre_configure": pre_configure,
        "post_install": post_install,
        "deps": deps,
        "build_deps": build_deps,
        "iuse": iuse,
        "use_defaults": use_defaults,
        "use_deps": use_deps,
        "use_meson": use_meson,
        "description": description,
        "homepage": homepage,
        "license": license,
        "slot": slot,
        "maintainers": maintainers,
        "build_type": "meson",
    }

    native.genrule(
        name = name + "_metadata",
        out = name + "_metadata.json",
        cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
        visibility = visibility,
    )

    native.filegroup(
        name = name,
        srcs = [":" + name + "_metadata"],
        visibility = visibility,
    )


def ebuild_package(
        name,
        version,
        src_uri = "",
        sha256 = "",
        phases = {},
        deps = [],
        build_deps = [],
        post_deps = [],
        iuse = [],
        use_defaults = [],
        required_use = "",
        description = "",
        homepage = "",
        license = "",
        slot = "0",
        keywords = [],
        maintainers = [],
        visibility = ["PUBLIC"]):
    """
    Define an ebuild-style package with explicit phases

    Args:
        name: Package name
        version: Package version
        src_uri: Source download URI
        sha256: Source checksum
        phases: Dict of phase name to shell commands
        deps: Runtime dependencies
        build_deps: Build-time dependencies
        post_deps: Post-install dependencies
        iuse: Available USE flags
        use_defaults: Default enabled flags
        required_use: USE flag requirements
        description: Package description
        homepage: Project homepage
        license: License identifier
        slot: Package slot
        keywords: Architecture keywords
        maintainers: Maintainer identifiers
        visibility: Buck visibility
    """

    metadata = {
        "name": name,
        "version": version,
        "src_uri": src_uri,
        "sha256": sha256,
        "phases": phases,
        "deps": deps,
        "build_deps": build_deps,
        "post_deps": post_deps,
        "iuse": iuse,
        "use_defaults": use_defaults,
        "required_use": required_use,
        "description": description,
        "homepage": homepage,
        "license": license,
        "slot": slot,
        "keywords": keywords,
        "maintainers": maintainers,
        "build_type": "ebuild",
    }

    native.genrule(
        name = name + "_metadata",
        out = name + "_metadata.json",
        cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
        visibility = visibility,
    )

    native.filegroup(
        name = name,
        srcs = [":" + name + "_metadata"],
        visibility = visibility,
    )


def binary_package(
        name,
        version,
        src_uri,
        sha256,
        install_commands = [],
        deps = [],
        description = "",
        homepage = "",
        license = "",
        slot = "0",
        maintainers = [],
        visibility = ["PUBLIC"]):
    """
    Define a pre-built binary package

    Args:
        name: Package name
        version: Package version
        src_uri: Binary download URI
        sha256: Binary checksum
        install_commands: Shell commands to install
        deps: Runtime dependencies
        description: Package description
        homepage: Project homepage
        license: License identifier
        slot: Package slot
        maintainers: Maintainer identifiers
        visibility: Buck visibility
    """

    metadata = {
        "name": name,
        "version": version,
        "src_uri": src_uri,
        "sha256": sha256,
        "install_commands": install_commands,
        "deps": deps,
        "description": description,
        "homepage": homepage,
        "license": license,
        "slot": slot,
        "maintainers": maintainers,
        "build_type": "binary",
    }

    native.genrule(
        name = name + "_metadata",
        out = name + "_metadata.json",
        cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
        visibility = visibility,
    )

    native.filegroup(
        name = name,
        srcs = [":" + name + "_metadata"],
        visibility = visibility,
    )


def virtual_package(
        name,
        version = "0",
        providers = [],
        description = "",
        visibility = ["PUBLIC"]):
    """
    Define a virtual package (meta-package with alternatives)

    Args:
        name: Virtual package name
        version: Virtual version
        providers: List of packages that provide this virtual
        description: Description
        visibility: Buck visibility
    """

    metadata = {
        "name": name,
        "version": version,
        "providers": providers,
        "description": description,
        "build_type": "virtual",
    }

    native.genrule(
        name = name + "_metadata",
        out = name + "_metadata.json",
        cmd = "echo '{}' > $OUT".format(json.encode(metadata)),
        visibility = visibility,
    )

    native.filegroup(
        name = name,
        srcs = [":" + name + "_metadata"],
        visibility = visibility,
    )
