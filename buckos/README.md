# Buckos

A modern, scalable operating system built with Rust, featuring a Buck-based package manager, systemd-like init system, and comprehensive system configuration management.

## Overview

Buckos is a next-generation operating system that combines the power of Rust's safety guarantees with battle-tested concepts from Gentoo Linux. It provides a complete system management solution including package management, service supervision, and system configuration.

## Architecture

Buckos is organized as a Rust workspace with the following crates:

| Crate | Description |
|-------|-------------|
| `buckos` | Main entry point and CLI |
| `buckos-model` | Core data models and types |
| `buckos-config` | System configuration management |
| `buckos-package` | Buck-based package manager |
| `buckos-start` | Init system (PID 1) |
| `buckos-assist` | Help and assistance utilities |
| `buckos-tools` | Utility tool collection |
| `buckos-web` | Official website and documentation |

## Features

### Package Management (`buckos-package`)
- **Buck2 Integration**: Modern, fast build system for scalable package builds
- **SAT Solver**: Intelligent dependency resolution using varisat
- **Parallel Execution**: Multi-threaded package operations
- **Emerge-Compatible CLI**: Familiar interface for Gentoo users
- **Transaction Support**: Atomic operations with rollback capabilities

### Init System (`buckos-start`)
- **PID 1 Service Manager**: systemd-like service supervision
- **Process Supervision**: Automatic restart with configurable policies
- **Dependency Management**: Service dependency ordering
- **Multiple Service Types**: Simple, Forking, Oneshot, Notify, Idle

### Configuration Management (`buckos-config`)
- **Portage-Inspired**: Familiar `/etc/portage` style configuration
- **USE Flags**: Fine-grained package customization
- **Package Sets**: Organized package groups
- **Profile System**: System-wide configuration profiles

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo
- Buck2 (for package building)
- SQLite (for package database)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/hodgesds/buckos.git
cd buckos/buckos

# Build all crates
cargo build --release

# Run tests
cargo test

# Install binaries
cargo install --path .
```

### Binary Locations

After building, the following binaries will be available:

- `buckos` - Package manager (main CLI)
- `start` - Init system
- `buckos-web` - Documentation server

## Quick Start

### Package Management

```bash
# Sync package repositories
buckos sync

# Search for packages
buckos search firefox

# Install a package
buckos install www-client/firefox

# Remove a package
buckos remove www-client/firefox

# Update system
buckos install -uDN @world
```

### Service Management

```bash
# Start a service
start start nginx

# Stop a service
start stop nginx

# Check service status
start status nginx

# List all services
start list
```

## Configuration

Buckos uses a configuration structure similar to Gentoo's `/etc/portage`:

```
/etc/buckos/
├── make.conf              # Global build settings
├── repos.conf/            # Repository configuration
├── package.use/           # Per-package USE flags
├── package.accept_keywords/  # Keyword acceptance
├── package.license/       # License acceptance
├── package.mask/          # Package masks
├── package.unmask/        # Package unmasks
├── package.env/           # Per-package environment
├── env/                   # Environment definitions
├── sets/                  # Custom package sets
└── world                  # User-selected packages
```

### Example make.conf

```bash
# Compiler flags
CFLAGS="-O2 -pipe -march=native"
CXXFLAGS="${CFLAGS}"

# USE flags
USE="X wayland pulseaudio -systemd"

# Build jobs
MAKEOPTS="-j8"

# Features
FEATURES="parallel-fetch ccache"
```

## Crate Documentation

### buckos-model

Core data models used throughout the system:

- `Package` - Package metadata and definitions
- `User/Group` - User and group management
- `Application` - Application definitions
- `Permission` - Access control models

### buckos-config

Configuration parsing and management:

- Atom parsing (e.g., `>=sys-apps/portage-2.3`)
- USE flag management
- Keyword acceptance
- Package masking/unmasking

### buckos-package

Package management operations:

- Repository synchronization
- Dependency resolution
- Package building and installation
- Transaction management

### buckos-start

Init system operations:

- Service lifecycle management
- Process supervision
- Signal handling
- System shutdown

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p buckos-package

# Run with verbose output
cargo test -- --nocapture
```

### Code Style

The project uses standard Rust formatting:

```bash
# Format code
cargo fmt

# Check for common issues
cargo clippy
```

### Documentation

Generate and view documentation:

```bash
# Generate docs
cargo doc --no-deps

# Open in browser
cargo doc --no-deps --open
```

## Dependencies

Key external dependencies:

- **clap** - CLI argument parsing
- **tokio** - Async runtime
- **serde** - Serialization/deserialization
- **rusqlite** - SQLite database
- **varisat** - SAT solver for dependency resolution
- **nix** - Unix system calls
- **axum** - Web framework

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Guidelines

- Write tests for new functionality
- Follow Rust best practices and idioms
- Update documentation as needed
- Keep commits focused and atomic

## License

This project is licensed under the terms specified in the LICENSE file.

## Acknowledgments

- Inspired by [Gentoo Linux](https://gentoo.org/) and its Portage package manager
- Built with the [Rust](https://www.rust-lang.org/) programming language
- Uses [Buck2](https://buck2.build/) for build system integration

## Related Projects

- [Gentoo Portage](https://wiki.gentoo.org/wiki/Portage)
- [systemd](https://systemd.io/)
- [Buck2](https://buck2.build/)
