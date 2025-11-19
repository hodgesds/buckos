# sideros-package

A scalable, Buck-based package manager for Sideros, inspired by Gentoo's Portage.

## Overview

`sideros-package` is a modern package manager that combines the flexibility and power of Gentoo's Portage with the speed and scalability of the Buck2 build system. It features intelligent dependency resolution using a SAT solver, parallel execution, and full transaction support with rollback capabilities.

## Features

- **Buck2 Build System**: Modern, fast, and scalable build system integration
- **SAT Solver Resolution**: Intelligent dependency resolution using varisat
- **Parallel Execution**: Multi-threaded package operations for maximum performance
- **Emerge-Compatible CLI**: Familiar interface for Gentoo users
- **Transaction Support**: Atomic operations with full rollback capabilities
- **SQLite Database**: Reliable local package database
- **Binary Package Support**: Build and use binary packages
- **Multiple Compression**: Support for gzip, zstd, and xz compression

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sideros-package = { path = "../package" }
```

Or use the CLI binary:

```bash
cargo install --path sideros/package
```

## CLI Usage

The `sideros-pkg` binary provides an emerge-compatible command-line interface.

### Basic Commands

```bash
# Sync package repositories
sideros-pkg sync

# Search for packages
sideros-pkg search firefox

# Show package information
sideros-pkg info www-client/firefox

# Install packages
sideros-pkg install www-client/firefox

# Remove packages
sideros-pkg remove www-client/firefox

# Clean build artifacts
sideros-pkg clean

# Remove unused dependencies
sideros-pkg depclean
```

### Installation Options

```bash
# Pretend (dry run)
sideros-pkg install -p www-client/firefox

# Ask for confirmation
sideros-pkg install -a www-client/firefox

# Verbose output
sideros-pkg install -v www-client/firefox

# Install without adding to @world set
sideros-pkg install --oneshot sys-apps/temporary-tool

# Fetch only (download without installing)
sideros-pkg install --fetchonly www-client/firefox

# Specify number of parallel jobs
sideros-pkg install -j 8 www-client/firefox
```

### Update Operations

```bash
# Update a single package
sideros-pkg install www-client/firefox

# Update entire system
sideros-pkg install -uDN @world

# Deep update (include dependencies)
sideros-pkg install -u --deep @world

# Rebuild packages with USE flag changes
sideros-pkg install -N @world

# Show dependency tree
sideros-pkg install -t www-client/firefox
```

### Build Operations

```bash
# Build a package
sideros-pkg build www-client/firefox

# Build with specific configuration
sideros-pkg build --config /path/to/config.toml www-client/firefox
```

## Architecture

### Core Components

```
sideros-package/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── main.rs          # CLI binary
│   ├── config.rs        # Configuration management
│   ├── types.rs         # Core type definitions
│   ├── error.rs         # Error types
│   ├── buck/            # Buck2 integration
│   ├── cache/           # Download and build caching
│   ├── catalog/         # Package catalog
│   ├── db/              # SQLite database
│   ├── executor/        # Parallel execution engine
│   ├── repository/      # Repository management
│   ├── resolver/        # Dependency resolution
│   ├── transaction/     # Transaction management
│   └── validation/      # Package validation
```

## Modules

### Database (`db`)

SQLite-based local package database for tracking installed packages.

```rust
use sideros_package::db::Database;

let db = Database::open("/var/db/sideros/packages.db")?;

// Query installed packages
let installed = db.list_installed()?;

// Check if package is installed
let is_installed = db.is_installed("www-client/firefox", "120.0")?;

// Get package metadata
let pkg = db.get_package("www-client/firefox")?;
```

### Buck Integration (`buck`)

Integration with the Buck2 build system for fast, reproducible builds.

```rust
use sideros_package::buck::BuckBuilder;

let builder = BuckBuilder::new("/var/db/repos/gentoo")?;

// Build a package
let result = builder.build("www-client/firefox", "120.0")?;

// Build with specific targets
let result = builder.build_targets(vec!["//www-client/firefox:main"])?;
```

### Resolver (`resolver`)

SAT solver-based dependency resolution using varisat.

```rust
use sideros_package::resolver::Resolver;

let resolver = Resolver::new(&catalog, &db)?;

// Resolve dependencies for a package
let solution = resolver.resolve("www-client/firefox")?;

// Resolve with specific constraints
let solution = resolver.resolve_with_constraints(
    "www-client/firefox",
    &constraints,
)?;

// Get installation order
let install_order = solution.installation_order();
```

**Resolution Features:**
- Handles version conflicts
- Respects USE flag dependencies
- Considers slot conflicts
- Supports blockers

### Executor (`executor`)

Parallel execution engine for scalable package operations.

```rust
use sideros_package::executor::Executor;

let executor = Executor::new(8)?; // 8 parallel jobs

// Execute package operations
let results = executor.execute(operations)?;

// Execute with progress callback
let results = executor.execute_with_progress(
    operations,
    |progress| println!("{:?}", progress),
)?;
```

### Transaction (`transaction`)

Atomic package operations with rollback support.

```rust
use sideros_package::transaction::Transaction;

let mut transaction = Transaction::begin(&db)?;

// Add operations
transaction.install("www-client/firefox", "120.0")?;
transaction.remove("www-client/chromium", "119.0")?;

// Commit or rollback
match transaction.commit() {
    Ok(_) => println!("Transaction successful"),
    Err(e) => {
        println!("Error: {}, rolling back", e);
        transaction.rollback()?;
    }
}
```

### Cache (`cache`)

Download and build artifact caching.

```rust
use sideros_package::cache::Cache;

let cache = Cache::new("/var/cache/sideros")?;

// Check if package is cached
let cached = cache.has("www-client/firefox", "120.0")?;

// Get cached package path
let path = cache.get_path("www-client/firefox", "120.0")?;

// Clean old cache entries
let freed = cache.clean_older_than(Duration::days(30))?;
```

### Repository (`repository`)

Package repository management.

```rust
use sideros_package::repository::Repository;

let repo = Repository::open("/var/db/repos/gentoo")?;

// Sync repository
repo.sync()?;

// List available packages
let packages = repo.list_packages("www-client")?;

// Get package metadata
let metadata = repo.get_metadata("www-client/firefox", "120.0")?;
```

### Catalog (`catalog`)

Package catalog for querying available packages.

```rust
use sideros_package::catalog::Catalog;

let catalog = Catalog::load(&repos)?;

// Search for packages
let results = catalog.search("firefox")?;

// Get all versions of a package
let versions = catalog.get_versions("www-client/firefox")?;

// Get package dependencies
let deps = catalog.get_dependencies("www-client/firefox", "120.0")?;
```

### Validation (`validation`)

Package validation before installation.

```rust
use sideros_package::validation::Validator;

let validator = Validator::new()?;

// Validate package integrity
let result = validator.validate_package(&package_path)?;

// Verify checksums
let valid = validator.verify_checksums(&package_path, &expected)?;
```

## Configuration

### Package Manager Configuration

```toml
# /etc/sideros/package.toml

[general]
# Number of parallel jobs
jobs = 8

# Package database location
database = "/var/db/sideros/packages.db"

# Cache directory
cache_dir = "/var/cache/sideros"

[download]
# Download timeout (seconds)
timeout = 300

# Retry attempts
retries = 3

# Use binary packages
binpkg = true

[build]
# Build directory
builddir = "/var/tmp/sideros"

# Keep build logs
keep_logs = true

# Build in tmpfs if available
tmpfs = true
```

## Library Usage

### Basic Package Installation

```rust
use sideros_package::{PackageManager, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load("/etc/sideros/package.toml")?;

    // Create package manager
    let pm = PackageManager::new(config)?;

    // Install a package
    pm.install("www-client/firefox", &InstallOptions::default()).await?;

    Ok(())
}
```

### Custom Dependency Resolution

```rust
use sideros_package::resolver::{Resolver, Constraints};

let resolver = Resolver::new(&catalog, &db)?;

// Add custom constraints
let mut constraints = Constraints::new();
constraints.require(">=www-client/firefox-120.0");
constraints.block("www-client/chromium");
constraints.use_flag("www-client/firefox", "wayland");

// Resolve with constraints
let solution = resolver.resolve_with_constraints("www-client/firefox", &constraints)?;

// Print solution
for pkg in solution.packages() {
    println!("Install: {}-{}", pkg.name, pkg.version);
}
```

### Transaction Management

```rust
use sideros_package::transaction::{Transaction, Operation};

let mut tx = Transaction::begin(&db)?;

// Queue multiple operations
tx.add(Operation::Install {
    package: "www-client/firefox".into(),
    version: "120.0".into(),
});
tx.add(Operation::Remove {
    package: "www-client/chromium".into(),
    version: "119.0".into(),
});

// Execute atomically
tx.commit()?;
```

### Progress Monitoring

```rust
use sideros_package::executor::{Executor, Progress};

let executor = Executor::new(8)?;

executor.execute_with_progress(operations, |progress| {
    match progress {
        Progress::Downloading { package, percent } => {
            println!("Downloading {}: {}%", package, percent);
        }
        Progress::Building { package, phase } => {
            println!("Building {}: {}", package, phase);
        }
        Progress::Installing { package } => {
            println!("Installing {}", package);
        }
        Progress::Complete { package } => {
            println!("Completed {}", package);
        }
    }
})?;
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.0 | Async runtime |
| `rusqlite` | Latest | SQLite database |
| `varisat` | Latest | SAT solver |
| `reqwest` | Latest | HTTP client |
| `sha2`, `blake3` | Latest | Checksums |
| `flate2`, `zstd`, `xz2` | Latest | Compression |
| `clap` | 4.4 | CLI parsing |
| `rayon` | Latest | Parallelism |
| `petgraph` | Latest | Dependency graphs |

## Comparison with Portage

| Feature | Portage | sideros-package |
|---------|---------|-----------------|
| Build System | Make/CMake/etc | Buck2 |
| Dependency Resolution | Custom | SAT Solver (varisat) |
| Parallelism | Limited | Full parallel execution |
| Database | Flat files | SQLite |
| Transactions | No | Full rollback support |
| CLI Compatibility | - | Emerge-compatible |

## Testing

```bash
# Run all tests
cargo test -p sideros-package

# Run specific test
cargo test -p sideros-package resolver

# Run with logging
RUST_LOG=debug cargo test -p sideros-package
```

## Performance

`sideros-package` is designed for high performance:

- **Parallel Downloads**: Multiple packages downloaded simultaneously
- **Parallel Builds**: Concurrent package compilation
- **Efficient Resolution**: SAT solver provides optimal solutions
- **Caching**: Aggressive caching of downloads and build artifacts
- **Incremental Updates**: Only rebuild what's necessary

## Contributing

When contributing to the package manager:

1. Follow the existing code patterns
2. Add tests for new functionality
3. Update documentation
4. Consider backward compatibility with Portage
5. Benchmark performance-critical changes

## License

This crate is part of the Sideros project and is licensed under the same terms.

## See Also

- [Buck2 Build System](https://buck2.build/)
- [Gentoo Portage](https://wiki.gentoo.org/wiki/Portage)
- [SAT Solving](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
