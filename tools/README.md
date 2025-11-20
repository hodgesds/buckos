# buckos-tools

A collection of utility tools for Buckos system administration and development.

## Overview

`buckos-tools` is a collection of command-line utilities that assist with various system administration, development, and maintenance tasks on Buckos. It serves as a central location for tools that don't warrant their own dedicated crate.

## Features

- **System Utilities**: Tools for system information and maintenance
- **Development Tools**: Utilities for package and system development
- **Diagnostic Utilities**: Tools for system analysis and debugging
- **Migration Helpers**: Tools for migrating from other systems

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
buckos-tools = { path = "../tools" }
```

Or install the binary:

```bash
cargo install --path tools
```

## Planned Tools

### System Information

```bash
# Display system information
buckos-tools sysinfo

# Show hardware information
buckos-tools hwinfo

# Display boot information
buckos-tools bootinfo

# Show loaded kernel modules
buckos-tools lsmod
```

### Package Utilities

```bash
# Check package integrity
buckos-tools pkg-check www-client/firefox

# List package files
buckos-tools pkg-files www-client/firefox

# Find which package owns a file
buckos-tools pkg-owner /usr/bin/firefox

# Verify all installed packages
buckos-tools pkg-verify
```

### Configuration Utilities

```bash
# Validate configuration
buckos-tools config-check

# Diff configurations
buckos-tools config-diff /etc/buckos/make.conf.old /etc/buckos/make.conf

# Generate configuration from template
buckos-tools config-gen --template desktop

# Migrate Gentoo configuration
buckos-tools config-migrate /etc/portage
```

### Development Utilities

```bash
# Generate ebuild skeleton
buckos-tools ebuild-new category/package

# Check ebuild syntax
buckos-tools ebuild-lint mypackage.ebuild

# Test build in sandbox
buckos-tools build-test category/package

# Generate package manifest
buckos-tools manifest category/package
```

### Diagnostic Tools

```bash
# Analyze system logs
buckos-tools log-analyze

# Check for security issues
buckos-tools security-scan

# Find orphaned files
buckos-tools orphans

# Check for broken symlinks
buckos-tools broken-links
```

### Migration Tools

```bash
# Migrate from Gentoo
buckos-tools migrate-gentoo

# Import package database
buckos-tools import-pkgdb /var/db/pkg

# Convert Portage config to Buckos
buckos-tools convert-portage-config /etc/portage
```

## Planned Library Usage

### System Information

```rust
use buckos_tools::sysinfo::SystemInfo;

let info = SystemInfo::collect()?;

println!("Hostname: {}", info.hostname);
println!("Kernel: {}", info.kernel_version);
println!("Architecture: {}", info.architecture);
println!("CPU: {}", info.cpu_info);
println!("Memory: {} / {}", info.memory_used, info.memory_total);
```

### Package Utilities

```rust
use buckos_tools::package::{PackageChecker, FileOwner};

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
use buckos_tools::config::{ConfigValidator, ConfigMigrator};

// Validate configuration
let validator = ConfigValidator::new()?;
let result = validator.validate("/etc/buckos")?;

for warning in result.warnings {
    println!("Warning: {}", warning);
}

// Migrate configuration
let migrator = ConfigMigrator::new()?;
migrator.migrate_from_gentoo("/etc/portage", "/etc/buckos")?;
```

### Development Utilities

```rust
use buckos_tools::dev::{EbuildGenerator, EbuildLinter};

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
cargo test -p buckos-tools

# Run specific tool tests
cargo test -p buckos-tools sysinfo

# Run integration tests
cargo test -p buckos-tools --features integration
```

## License

This crate is part of the Buckos project and is licensed under the same terms.

## See Also

- [Gentoo eutils](https://wiki.gentoo.org/wiki/Eutils) - Similar utility collection
- [Portage Utils](https://wiki.gentoo.org/wiki/Portage-utils) - C-based Portage utilities
