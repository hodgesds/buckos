# Buckos build definitions
# Compatible with buckos-build buck definitions
#
# This file provides macros and rules for building buckos components.
# It can be extended by buckos-build to provide additional functionality.

load("@prelude//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")

def buckos_rust_library(
    name,
    srcs = None,
    deps = None,
    edition = "2021",
    features = None,
    visibility = None,
    **kwargs
):
    """Create a Rust library for the buckos project.

    This macro wraps rust_library with buckos-specific defaults and
    conventions. It automatically handles:
    - Default edition (2021)
    - Common dependencies
    - Visibility patterns

    Args:
        name: Target name
        srcs: Source files (defaults to glob)
        deps: Dependencies
        edition: Rust edition (default: 2021)
        features: Crate features to enable
        visibility: Visibility specification
        **kwargs: Additional arguments passed to rust_library
    """
    if srcs == None:
        srcs = native.glob(["src/**/*.rs"])

    if deps == None:
        deps = []

    if visibility == None:
        visibility = ["PUBLIC"]

    if features == None:
        features = []

    rust_library(
        name = name,
        srcs = srcs,
        deps = deps,
        edition = edition,
        features = features,
        visibility = visibility,
        **kwargs
    )

def buckos_rust_binary(
    name,
    srcs = None,
    deps = None,
    edition = "2021",
    visibility = None,
    **kwargs
):
    """Create a Rust binary for the buckos project.

    This macro wraps rust_binary with buckos-specific defaults.

    Args:
        name: Target name
        srcs: Source files
        deps: Dependencies
        edition: Rust edition (default: 2021)
        visibility: Visibility specification
        **kwargs: Additional arguments passed to rust_binary
    """
    if deps == None:
        deps = []

    if visibility == None:
        visibility = ["PUBLIC"]

    rust_binary(
        name = name,
        srcs = srcs,
        deps = deps,
        edition = edition,
        visibility = visibility,
        **kwargs
    )

def buckos_package(
    name,
    category,
    version,
    description = None,
    homepage = None,
    license = None,
    deps = None,
    build_deps = None,
    use_flags = None,
    slot = "0",
    keywords = None,
    **kwargs
):
    """Define a buckos package for the package manager.

    This macro creates the necessary targets for a package that can be
    managed by buckos-pkg. It generates:
    - Package metadata
    - Build target
    - Install target

    Args:
        name: Package name
        category: Package category (e.g., "sys-libs", "dev-util")
        version: Package version
        description: Package description
        homepage: Package homepage URL
        license: Package license
        deps: Runtime dependencies
        build_deps: Build-time dependencies
        use_flags: Available USE flags
        slot: Package slot (default: "0")
        keywords: Architecture keywords
        **kwargs: Additional arguments
    """
    if deps == None:
        deps = []

    if build_deps == None:
        build_deps = []

    if use_flags == None:
        use_flags = []

    if keywords == None:
        keywords = ["~amd64", "~arm64"]

    # Create package metadata target
    native.genrule(
        name = "{}-metadata".format(name),
        out = "metadata.json",
        cmd = """
            echo '{
                "name": "%s",
                "category": "%s",
                "version": "%s",
                "description": "%s",
                "homepage": "%s",
                "license": "%s",
                "slot": "%s",
                "keywords": %s,
                "use_flags": %s,
                "deps": %s,
                "build_deps": %s
            }' > $OUT
        """ % (
            name,
            category,
            version,
            description or "",
            homepage or "",
            license or "",
            slot,
            repr(keywords),
            repr(use_flags),
            repr(deps),
            repr(build_deps),
        ),
        visibility = ["PUBLIC"],
    )

    # Create package target alias
    native.alias(
        name = name,
        actual = ":{}-metadata".format(name),
        visibility = ["PUBLIC"],
    )

def buckos_crate_deps(deps_list):
    """Convert a list of crate names to third-party targets.

    Args:
        deps_list: List of crate names

    Returns:
        List of Buck targets for the crates
    """
    return ["//third-party:{}".format(dep) for dep in deps_list]

# Package target naming convention
def package_target(category, name):
    """Generate a Buck target for a buckos package.

    Args:
        category: Package category
        name: Package name

    Returns:
        Buck target string
    """
    return "//packages/{}/{}:package".format(category, name)
