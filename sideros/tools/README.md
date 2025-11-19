# sideros-tools

A collection of utility tools for Sideros system administration and development.

## Overview

`sideros-tools` is a collection of command-line utilities that assist with various system administration, development, and maintenance tasks on Sideros. It serves as a central location for tools that don't warrant their own dedicated crate.

## Features

- **System Utilities**: Tools for system information and maintenance
- **Development Tools**: Utilities for package and system development
- **Diagnostic Utilities**: Tools for system analysis and debugging
- **Migration Helpers**: Tools for migrating from other systems

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sideros-tools = { path = "../tools" }
```

Or install the binary:

```bash
cargo install --path sideros/tools
```

## Planned Tools

### System Information

```bash
# Display system information
sideros-tools sysinfo

# Show hardware information
sideros-tools hwinfo

# Display boot information
sideros-tools bootinfo

# Show loaded kernel modules
sideros-tools lsmod
```

### Package Utilities

```bash
# Check package integrity
sideros-tools pkg-check www-client/firefox

# List package files
sideros-tools pkg-files www-client/firefox

# Find which package owns a file
sideros-tools pkg-owner /usr/bin/firefox

# Verify all installed packages
sideros-tools pkg-verify
```

### Configuration Utilities

```bash
# Validate configuration
sideros-tools config-check

# Diff configurations
sideros-tools config-diff /etc/sideros/make.conf.old /etc/sideros/make.conf

# Generate configuration from template
sideros-tools config-gen --template desktop

# Migrate Gentoo configuration
sideros-tools config-migrate /etc/portage
```

### Development Utilities

```bash
# Generate ebuild skeleton
sideros-tools ebuild-new category/package

# Check ebuild syntax
sideros-tools ebuild-lint mypackage.ebuild

# Test build in sandbox
sideros-tools build-test category/package

# Generate package manifest
sideros-tools manifest category/package
```

### Diagnostic Tools

```bash
# Analyze system logs
sideros-tools log-analyze

# Check for security issues
sideros-tools security-scan

# Find orphaned files
sideros-tools orphans

# Check for broken symlinks
sideros-tools broken-links
```

### Migration Tools

```bash
# Migrate from Gentoo
sideros-tools migrate-gentoo

# Import package database
sideros-tools import-pkgdb /var/db/pkg

# Convert Portage config to Sideros
sideros-tools convert-portage-config /etc/portage
```

## Planned Library Usage

### System Information

```rust
use sideros_tools::sysinfo::SystemInfo;

let info = SystemInfo::collect()?;

println!("Hostname: {}", info.hostname);
println!("Kernel: {}", info.kernel_version);
println!("Architecture: {}", info.architecture);
println!("CPU: {}", info.cpu_info);
println!("Memory: {} / {}", info.memory_used, info.memory_total);
```

### Package Utilities

```rust
use sideros_tools::package::{PackageChecker, FileOwner};

// Check package integrity
let checker = PackageChecker::new(&db)?;
let result = checker.check("www-client/firefox")?;

if !result.is_ok() {
    for issue in result.issues {
        println!("Issue: {}", issue);
    }
}

// Find file owner
let owner = FileOwner::new(&db)?;
let package = owner.find("/usr/bin/firefox")?;
println!("Owned by: {}", package);
```

### Configuration Utilities

```rust
use sideros_tools::config::{ConfigValidator, ConfigMigrator};

// Validate configuration
let validator = ConfigValidator::new()?;
let result = validator.validate("/etc/sideros")?;

for warning in result.warnings {
    println!("Warning: {}", warning);
}

// Migrate configuration
let migrator = ConfigMigrator::new()?;
migrator.migrate_from_gentoo("/etc/portage", "/etc/sideros")?;
```

### Development Utilities

```rust
use sideros_tools::dev::{EbuildGenerator, EbuildLinter};

// Generate ebuild
let generator = EbuildGenerator::new()?;
generator.generate("category/package", &options)?;

// Lint ebuild
let linter = EbuildLinter::new()?;
let result = linter.lint("mypackage.ebuild")?;

for issue in result.issues {
    println!("{}: {}", issue.severity, issue.message);
}
```

## Tool Index

| Tool | Description | Status |
|------|-------------|--------|
| `sysinfo` | System information display | Planned |
| `hwinfo` | Hardware information | Planned |
| `pkg-check` | Package integrity check | Planned |
| `pkg-files` | List package files | Planned |
| `pkg-owner` | Find file owner | Planned |
| `config-check` | Configuration validation | Planned |
| `config-migrate` | Configuration migration | Planned |
| `ebuild-new` | Ebuild generator | Planned |
| `ebuild-lint` | Ebuild linter | Planned |
| `log-analyze` | Log analysis | Planned |
| `security-scan` | Security scanner | Planned |
| `migrate-gentoo` | Gentoo migration | Planned |

## Status

This crate is currently in early development. It serves as a collection point for various utility tools that will be developed as needed.

### Roadmap

1. **Phase 1**: Basic system information tools
2. **Phase 2**: Package verification utilities
3. **Phase 3**: Configuration migration tools
4. **Phase 4**: Development utilities
5. **Phase 5**: Advanced diagnostic tools

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.0 | CLI parsing (planned) |
| `serde` | 1.0 | Data serialization (planned) |
| `walkdir` | Latest | Directory traversal (planned) |
| `sha2` | Latest | Checksums (planned) |
| `sysinfo` | Latest | System information (planned) |

## Contributing

Contributions are welcome! When adding a new tool:

1. Consider if it should be a separate crate or part of this collection
2. Follow the existing code patterns
3. Add comprehensive documentation
4. Include unit and integration tests
5. Add the tool to this README

### Adding a New Tool

1. Create a new module under `src/tools/`
2. Implement the tool's functionality
3. Add CLI integration in `src/main.rs`
4. Document the tool in this README
5. Add tests

## Testing

```bash
# Run all tests
cargo test -p sideros-tools

# Run specific tool tests
cargo test -p sideros-tools sysinfo

# Run integration tests
cargo test -p sideros-tools --features integration
```

## License

This crate is part of the Sideros project and is licensed under the same terms.

## See Also

- [Gentoo eutils](https://wiki.gentoo.org/wiki/Eutils) - Similar utility collection
- [Portage Utils](https://wiki.gentoo.org/wiki/Portage-utils) - C-based Portage utilities
