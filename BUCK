# Root BUCK file for sideros project
# Compatible with sideros-build buck definitions

# Export all sideros crates
export_file(
    name = "README.md",
    src = "README.md",
    visibility = ["PUBLIC"],
)

# Alias for building all sideros crates
filegroup(
    name = "sideros",
    srcs = [
        "//sideros/model:sideros-model",
        "//sideros/package:sideros-package",
        "//sideros/package:sideros-pkg",
    ],
    visibility = ["PUBLIC"],
)
