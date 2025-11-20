# Copyright (c) BuckOS Authors. All rights reserved.
# Eclass Inheritance System

"""
Eclass Module

This module provides reusable build patterns for common package types,
similar to Gentoo's eclasses. Eclasses define standardized build phases,
dependencies, and configurations.
"""

# Available eclasses and their definitions
ECLASSES = {
    "cmake": {
        "name": "cmake",
        "description": "Support for CMake-based packages",
        "src_configure": """
            cmake -B build \\
                -DCMAKE_INSTALL_PREFIX=/usr \\
                -DCMAKE_BUILD_TYPE=Release \\
                -DCMAKE_INSTALL_LIBDIR=lib \\
                ${CMAKE_ARGS:-}
        """,
        "src_compile": """
            cmake --build build -j${NPROC:-$(nproc)}
        """,
        "src_install": """
            DESTDIR="$DESTDIR" cmake --install build
        """,
        "src_test": """
            cmake --build build --target test
        """,
        "bdepend": ["//packages/linux/dev-util/cmake:cmake", "//packages/linux/dev-util/ninja:ninja"],
        "rdepend": [],
        "exports": ["CMAKE_ARGS", "CMAKE_BUILD_TYPE", "CMAKE_INSTALL_PREFIX"],
    },
    "meson": {
        "name": "meson",
        "description": "Support for Meson-based packages",
        "src_configure": """
            meson setup build \\
                --prefix=/usr \\
                --libdir=lib \\
                --buildtype=release \\
                ${MESON_ARGS:-}
        """,
        "src_compile": """
            meson compile -C build -j${NPROC:-$(nproc)}
        """,
        "src_install": """
            DESTDIR="$DESTDIR" meson install -C build
        """,
        "src_test": """
            meson test -C build
        """,
        "bdepend": ["//packages/linux/dev-util/meson:meson", "//packages/linux/dev-util/ninja:ninja"],
        "rdepend": [],
        "exports": ["MESON_ARGS"],
    },
    "autotools": {
        "name": "autotools",
        "description": "Support for autotools-based packages (configure/make)",
        "src_configure": """
            ./configure \\
                --prefix=/usr \\
                --libdir=/usr/lib \\
                --sysconfdir=/etc \\
                --localstatedir=/var \\
                ${CONFIGURE_ARGS:-}
        """,
        "src_compile": """
            make -j${NPROC:-$(nproc)} ${MAKE_ARGS:-}
        """,
        "src_install": """
            make DESTDIR="$DESTDIR" ${INSTALL_ARGS:-} install
        """,
        "src_test": """
            make check
        """,
        "bdepend": ["//packages/linux/dev-build/autoconf:autoconf", "//packages/linux/dev-build/automake:automake", "//packages/linux/dev-build/libtool:libtool"],
        "rdepend": [],
        "exports": ["CONFIGURE_ARGS", "MAKE_ARGS", "INSTALL_ARGS"],
    },
    "python-single-r1": {
        "name": "python-single-r1",
        "description": "Support for packages using a single Python implementation",
        "src_configure": """
            python setup.py configure ${PYTHON_CONFIGURE_ARGS:-}
        """,
        "src_compile": """
            python setup.py build
        """,
        "src_install": """
            python setup.py install --root="$DESTDIR" --prefix=/usr
        """,
        "src_test": """
            python setup.py test
        """,
        "bdepend": ["//packages/linux/dev-lang/python:python"],
        "rdepend": ["//packages/linux/dev-lang/python:python"],
        "exports": ["PYTHON", "PYTHON_TARGETS", "PYTHON_SINGLE_TARGET"],
    },
    "python-r1": {
        "name": "python-r1",
        "description": "Support for packages with multiple Python implementations",
        "src_configure": """
            for impl in ${PYTHON_TARGETS}; do
                python${impl} setup.py configure ${PYTHON_CONFIGURE_ARGS:-}
            done
        """,
        "src_compile": """
            for impl in ${PYTHON_TARGETS}; do
                python${impl} setup.py build
            done
        """,
        "src_install": """
            for impl in ${PYTHON_TARGETS}; do
                python${impl} setup.py install --root="$DESTDIR" --prefix=/usr
            done
        """,
        "src_test": """
            for impl in ${PYTHON_TARGETS}; do
                python${impl} setup.py test
            done
        """,
        "bdepend": ["//packages/linux/dev-lang/python:python"],
        "rdepend": ["//packages/linux/dev-lang/python:python"],
        "exports": ["PYTHON_TARGETS", "PYTHON_COMPAT"],
    },
    "go-module": {
        "name": "go-module",
        "description": "Support for Go module packages",
        "src_configure": """
            # No configure needed for Go modules
            true
        """,
        "src_compile": """
            go build -v -mod=readonly ${GO_BUILD_ARGS:-} .
        """,
        "src_install": """
            go install -v -mod=readonly ${GO_INSTALL_ARGS:-} .
            if [ -f "$GOPATH/bin/${PN}" ]; then
                install -Dm755 "$GOPATH/bin/${PN}" "$DESTDIR/usr/bin/${PN}"
            fi
        """,
        "src_test": """
            go test -v ./...
        """,
        "bdepend": ["//packages/linux/dev-lang/go:go"],
        "rdepend": [],
        "exports": ["GOPATH", "GOFLAGS", "GO_BUILD_ARGS", "GO_INSTALL_ARGS"],
    },
    "cargo": {
        "name": "cargo",
        "description": "Support for Rust/Cargo packages",
        "src_configure": """
            # No configure needed for Cargo
            true
        """,
        "src_compile": """
            cargo build --release ${CARGO_ARGS:-}
        """,
        "src_install": """
            cargo install --path . --root "$DESTDIR/usr" ${CARGO_INSTALL_ARGS:-}
        """,
        "src_test": """
            cargo test
        """,
        "bdepend": ["//packages/linux/dev-lang/rust:rust"],
        "rdepend": [],
        "exports": ["CARGO_ARGS", "CARGO_INSTALL_ARGS", "CARGO_HOME"],
    },
    "xdg": {
        "name": "xdg",
        "description": "Support for XDG desktop applications",
        "src_configure": "",
        "src_compile": "",
        "src_install": "",
        "pkg_postinst": """
            if [ -x /usr/bin/update-desktop-database ]; then
                update-desktop-database -q /usr/share/applications
            fi
            if [ -x /usr/bin/update-mime-database ]; then
                update-mime-database /usr/share/mime
            fi
            if [ -x /usr/bin/gtk-update-icon-cache ]; then
                gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor
            fi
        """,
        "bdepend": [],
        "rdepend": ["//packages/linux/dev-util/desktop-file-utils:desktop-file-utils"],
        "exports": [],
    },
    "linux-mod": {
        "name": "linux-mod",
        "description": "Support for Linux kernel modules",
        "src_configure": """
            # Kernel modules typically don't need configure
            true
        """,
        "src_compile": """
            make -C /lib/modules/$(uname -r)/build M=$PWD modules
        """,
        "src_install": """
            make -C /lib/modules/$(uname -r)/build M=$PWD INSTALL_MOD_PATH="$DESTDIR" modules_install
        """,
        "pkg_postinst": """
            depmod -a
        """,
        "bdepend": ["//packages/linux/kernel/linux-headers:linux-headers"],
        "rdepend": [],
        "exports": ["KERNEL_DIR", "KERNEL_VERSION"],
    },
    "systemd": {
        "name": "systemd",
        "description": "Support for systemd service units",
        "src_configure": "",
        "src_compile": "",
        "src_install": """
            if [ -d systemd ]; then
                install -d "$DESTDIR/usr/lib/systemd/system"
                install -m 644 systemd/*.service "$DESTDIR/usr/lib/systemd/system/"
            fi
        """,
        "pkg_postinst": """
            systemctl daemon-reload
        """,
        "bdepend": [],
        "rdepend": ["//packages/linux/system/init/systemd:systemd"],
        "exports": [],
    },
    "qt5": {
        "name": "qt5",
        "description": "Support for Qt5 applications",
        "src_configure": """
            qmake ${QMAKE_ARGS:-}
        """,
        "src_compile": """
            make -j${NPROC:-$(nproc)}
        """,
        "src_install": """
            make INSTALL_ROOT="$DESTDIR" install
        """,
        "bdepend": ["//packages/linux/dev-qt/qtbase:qtbase"],
        "rdepend": ["//packages/linux/dev-qt/qtbase:qtbase"],
        "exports": ["QMAKE_ARGS", "QT_SELECT"],
    },
    "qt6": {
        "name": "qt6",
        "description": "Support for Qt6 applications",
        "src_configure": """
            qt6-cmake -B build ${QT6_CMAKE_ARGS:-}
        """,
        "src_compile": """
            cmake --build build -j${NPROC:-$(nproc)}
        """,
        "src_install": """
            DESTDIR="$DESTDIR" cmake --install build
        """,
        "bdepend": ["//packages/linux/dev-qt/qt6-base:qt6-base", "//packages/linux/dev-util/cmake:cmake"],
        "rdepend": ["//packages/linux/dev-qt/qt6-base:qt6-base"],
        "exports": ["QT6_CMAKE_ARGS"],
    },
}


def list_eclasses():
    """
    List all available eclasses

    Returns:
        List of eclass names
    """
    return sorted(ECLASSES.keys())


def get_eclass(name):
    """
    Get eclass definition by name

    Args:
        name: Eclass name

    Returns:
        Eclass definition dict or None
    """
    return ECLASSES.get(name)


def eclass_has_phase(eclass_name, phase):
    """
    Check if an eclass provides a specific build phase

    Args:
        eclass_name: Eclass name
        phase: Phase name (e.g., "src_configure", "src_compile")

    Returns:
        True if eclass provides the phase
    """
    eclass = ECLASSES.get(eclass_name)
    if not eclass:
        return False
    return phase in eclass and eclass[phase] != ""


def inherit(eclass_names):
    """
    Inherit configuration from multiple eclasses

    Merges configurations from multiple eclasses, with later eclasses
    overriding earlier ones for conflicting values.

    Args:
        eclass_names: List of eclass names to inherit

    Returns:
        Merged configuration dict with all phases and dependencies
    """
    result = {
        "src_configure": "",
        "src_compile": "",
        "src_install": "",
        "src_test": "",
        "pkg_postinst": "",
        "bdepend": [],
        "rdepend": [],
        "exports": [],
    }

    for name in eclass_names:
        eclass = ECLASSES.get(name)
        if not eclass:
            fail("Unknown eclass: " + name)

        # Merge phases (later overrides)
        for phase in ["src_configure", "src_compile", "src_install", "src_test", "pkg_postinst"]:
            if phase in eclass and eclass[phase]:
                result[phase] = eclass[phase]

        # Merge dependencies (additive)
        for dep_type in ["bdepend", "rdepend"]:
            if dep_type in eclass:
                for dep in eclass[dep_type]:
                    if dep not in result[dep_type]:
                        result[dep_type].append(dep)

        # Merge exports (additive)
        if "exports" in eclass:
            for export in eclass["exports"]:
                if export not in result["exports"]:
                    result["exports"].append(export)

    return result


def eclass_package(
        name,
        version,
        eclasses,
        src_uri = "",
        sha256 = "",
        description = "",
        homepage = "",
        license = "",
        slot = "0",
        iuse = [],
        deps = [],
        build_deps = [],
        maintainers = [],
        phase_overrides = {},
        visibility = ["PUBLIC"]):
    """
    Define a package using eclass inheritance

    Args:
        name: Package name
        version: Package version
        eclasses: List of eclasses to inherit
        src_uri: Source download URI
        sha256: Source checksum
        description: Package description
        homepage: Project homepage
        license: License identifier
        slot: Package slot
        iuse: USE flags specific to this package
        deps: Additional runtime dependencies
        build_deps: Additional build dependencies
        maintainers: Package maintainers
        phase_overrides: Dict of phase names to custom implementations
        visibility: Buck visibility
    """
    # Get merged eclass configuration
    eclass_config = inherit(eclasses)

    # Merge dependencies
    all_deps = deps + eclass_config["rdepend"]
    all_build_deps = build_deps + eclass_config["bdepend"]

    # Apply phase overrides
    phases = {}
    for phase in ["src_configure", "src_compile", "src_install", "src_test", "pkg_postinst"]:
        if phase in phase_overrides:
            phases[phase] = phase_overrides[phase]
        elif phase in eclass_config and eclass_config[phase]:
            phases[phase] = eclass_config[phase]

    # Generate package metadata
    metadata = {
        "name": name,
        "version": version,
        "eclasses": eclasses,
        "src_uri": src_uri,
        "sha256": sha256,
        "description": description,
        "homepage": homepage,
        "license": license,
        "slot": slot,
        "iuse": iuse,
        "deps": all_deps,
        "build_deps": all_build_deps,
        "maintainers": maintainers,
        "phases": phases,
        "exports": eclass_config["exports"],
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


def get_eclass_bdepend(eclass_names):
    """
    Get all build dependencies for a list of eclasses

    Args:
        eclass_names: List of eclass names

    Returns:
        List of build dependencies
    """
    deps = []
    for name in eclass_names:
        eclass = ECLASSES.get(name)
        if eclass and "bdepend" in eclass:
            for dep in eclass["bdepend"]:
                if dep not in deps:
                    deps.append(dep)
    return deps


def get_eclass_rdepend(eclass_names):
    """
    Get all runtime dependencies for a list of eclasses

    Args:
        eclass_names: List of eclass names

    Returns:
        List of runtime dependencies
    """
    deps = []
    for name in eclass_names:
        eclass = ECLASSES.get(name)
        if eclass and "rdepend" in eclass:
            for dep in eclass["rdepend"]:
                if dep not in deps:
                    deps.append(dep)
    return deps
