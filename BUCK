# Root BUCK file for buckos project
# Compatible with buckos-build buck definitions

# Export all buckos crates
export_file(
    name = "README.md",
    src = "README.md",
    visibility = ["PUBLIC"],
)

# Alias for building all buckos crates
filegroup(
    name = "buckos",
    srcs = [
        "//buckos/model:buckos-model",
        "//buckos/package:buckos-package",
        "//buckos/package:buckos-pkg",
    ],
    visibility = ["PUBLIC"],
)
