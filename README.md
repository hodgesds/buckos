# Buckos

**Buckos** is a modern Linux distribution built on top of [Buck2](https://buck.build/), Facebook's fast and scalable build system. Inspired by [Gentoo Linux](https://gentoo.org/) and its powerful Portage package manager, Buckos brings source-based package management to a new level with deterministic builds, fine-grained configuration, and modern Rust-based tooling.

## Philosophy

Buckos combines the best of both worlds:

- **Gentoo's Flexibility**: USE flag-style configuration for fine-grained control over package features
- **Buck2's Performance**: Hermetic, reproducible builds with aggressive caching and parallelization
- **Modern Architecture**: Written entirely in Rust for safety, performance, and reliability

## Key Features

- **Source-based Distribution**: Build packages from source with your exact specifications
- **USE Flag Configuration**: Enable/disable package features through build flags
- **SAT Solver Dependency Resolution**: Intelligent handling of complex dependency graphs
- **Parallel Everything**: Multi-threaded downloads, builds, and package operations
- **Transaction Support**: Atomic operations with rollback capability
- **Buck2 Integration**: Fast, reproducible builds with remote caching support
- **Portage Compatibility**: Familiar interface for Gentoo users

## Project Structure

```
buckos/
├── .buckconfig           # Buck2 build configuration
├── .buckroot             # Buck2 workspace root marker
├── BUCK                  # Root build definitions
├── README.md             # This file
├── buckos/               # Main workspace
│   ├── Cargo.toml        # Rust workspace configuration
│   ├── model/            # Core data models
│   ├── package/          # Package manager (buckos-pkg)
│   ├── assist/           # System diagnostics
│   ├── config/           # Configuration management
│   ├── start/            # Init system (PID 1)
│   ├── web/              # Documentation website
│   └── tools/            # System utilities
├── build/                # Build artifacts
├── platforms/            # Platform definitions
├── third-party/          # Third-party dependencies
└── toolchains/           # Build toolchain configurations
```

## Crates Overview

### buckos-package (Package Manager)

The core package manager with a Portage-like interface. Handles package installation, dependency resolution, and Buck2 integration.

**Binary**: `buckos-pkg`

**Key Commands**:
```bash
buckos-pkg install <package>     # Install a package
buckos-pkg remove <package>      # Remove a package
buckos-pkg update                # Update installed packages
buckos-pkg sync                  # Sync package repositories
buckos-pkg search <query>        # Search for packages
buckos-pkg info <package>        # Show package information
buckos-pkg depclean              # Remove unused dependencies
buckos-pkg build <package>       # Build without installing
buckos-pkg verify                # Verify installed packages
buckos-pkg audit                 # Security audit
```

### buckos-config (Configuration Management)

Manages system configuration with full Portage compatibility.

**Supported Configurations**:
- `make.conf` - Global build settings (CFLAGS, USE, FEATURES, MAKEOPTS)
- `package.use/` - Per-package USE flags
- `package.accept_keywords/` - Architecture/stability keywords
- `package.license/` - License acceptance
- `package.mask/` - Package masks/unmasks
- `repos.conf/` - Repository configuration
- Custom package sets (@world, @system, etc.)

### buckos-start (Init System)

A modern init system and service manager designed to run as PID 1.

**Binary**: `start`

**Features**:
- Service dependency management
- Multiple service types (simple, forking, oneshot, notify, idle)
- Automatic restart with configurable policies
- Signal handling and zombie reaping

### buckos-assist (System Diagnostics)

System diagnostic and troubleshooting assistant with privacy controls.

**Binary**: `buckos-assist`

**Commands**:
```bash
buckos-assist collect            # Gather system diagnostics
buckos-assist summary            # Quick system overview
buckos-assist privacy            # Configure privacy settings
```

### buckos-model (Data Models)

Core data types used throughout the project including Package, User, License, and system entities.

### buckos-web (Documentation Website)

Official website and documentation server built with Axum.

### buckos-tools (Utilities)

Collection of system administration and development utilities.

## Package Ecosystem

### Build Repository

Package definitions (Buck targets) are maintained in a separate repository:

**Repository**: [buckos-build](https://github.com/hodgesds/buckos-build)

This repository contains the equivalent of Gentoo's ebuilds as Buck build targets. Each package definition includes:
- Source URLs and checksums
- Dependencies and build requirements
- USE flag definitions and conditional dependencies
- Build instructions and phases
- Installation rules

### Package Categories

Packages are organized into categories similar to Portage:
- `core/` - Core system utilities
- `build/` - Build tools and compilers
- `compression/` - Compression libraries and tools
- `network/` - Networking tools and libraries
- `toolchain/` - Compiler toolchains
- `services/` - System services and daemons
- `utils/` - General utilities

## Configuration

### make.conf

The main configuration file located at `/etc/buckos/make.conf`:

```bash
# Compiler flags
CFLAGS="-O2 -pipe -march=native"
CXXFLAGS="${CFLAGS}"

# Number of parallel jobs
MAKEOPTS="-j$(nproc)"

# Global USE flags
USE="wayland pulseaudio -systemd"

# Accept licenses
ACCEPT_LICENSE="*"

# Features
FEATURES="parallel-fetch buildpkg ccache"

# Buck2 specific
BUCK_JOBS="auto"
BUCK_CACHE_MODE="readwrite"
```

### USE Flags (Build Flags)

USE flags control optional features in packages. They can be set globally or per-package:

**Global** (`/etc/buckos/make.conf`):
```bash
USE="wayland pulseaudio bluetooth -X"
```

**Per-package** (`/etc/buckos/package.use/`):
```bash
# Enable specific flags for firefox
www-client/firefox wayland webrtc -dbus

# Enable all audio support for vlc
media-video/vlc alsa pulseaudio jack
```

### Package Masking

Control which package versions are available:

```bash
# /etc/buckos/package.mask/custom
# Mask unstable version
>=dev-lang/rust-1.80.0

# /etc/buckos/package.unmask/custom
# Unmask specific package
=sys-kernel/linux-6.8.0
```

### Keywords

Accept testing/unstable packages:

```bash
# /etc/buckos/package.accept_keywords/testing
# Accept testing packages
dev-util/buck ~amd64
sys-devel/llvm ~amd64
```

## Building Packages

Buckos uses Buck2 for building packages. The build process is:

1. **Resolution**: Resolve dependencies using SAT solver
2. **Fetch**: Download sources in parallel
3. **Build**: Execute Buck2 build with specified flags
4. **Install**: Install built artifacts to system

### Buck2 Integration

Each package maps to a Buck2 target in the build repository:

```python
# Example Buck target for a package
rust_binary(
    name = "ripgrep",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        "//third-party:regex",
        "//third-party:clap",
    ],
    features = select({
        "//config:use_pcre2": ["pcre2"],
        "DEFAULT": [],
    }),
)
```

Build flags are translated to Buck2 configuration:

```bash
# Install with specific features
buckos-pkg install ripgrep --use="pcre2"

# This translates to Buck2 config
buck2 build //packages/ripgrep:ripgrep --config //config:use_pcre2=True
```

## Installation

### Requirements

- Rust 1.70+ (for building from source)
- Buck2 (for package building)
- SQLite 3.x
- Linux kernel 5.x+

### Building from Source

```bash
# Clone the repository
git clone https://github.com/hodgesds/buckos.git
cd buckos

# Build all crates
cargo build --release

# Install binaries
cargo install --path buckos/package
cargo install --path buckos/start
cargo install --path buckos/assist
```

### Initial Setup

```bash
# Create configuration directory
sudo mkdir -p /etc/buckos

# Copy default configuration
sudo cp -r config/defaults/* /etc/buckos/

# Initialize package database
buckos-pkg --init

# Sync repositories
buckos-pkg sync
```

## Usage Examples

### Package Management

```bash
# Sync package repository
buckos-pkg sync

# Search for a package
buckos-pkg search "web browser"

# Get package information
buckos-pkg info www-client/firefox

# Install with specific USE flags
buckos-pkg install www-client/firefox --use="wayland -dbus"

# Update all packages
buckos-pkg update @world

# Remove a package and unused dependencies
buckos-pkg remove www-client/firefox
buckos-pkg depclean

# Verify installed packages
buckos-pkg verify
```

### System Diagnostics

```bash
# Collect full system diagnostics
buckos-assist collect --format json --output report.json

# Quick system summary
buckos-assist summary

# Collect with privacy controls
buckos-assist collect --privacy minimal --redact-usernames --redact-ips
```

### Service Management

```bash
# Start a service
buckos-start start nginx

# Enable service at boot
buckos-start enable nginx

# Check service status
buckos-start status nginx

# View service logs
buckos-start logs nginx
```

## Comparison with Gentoo

| Feature | Gentoo/Portage | Buckos |
|---------|----------------|--------|
| Build System | Custom (ebuild) | Buck2 |
| Package Definitions | Shell scripts | Buck targets |
| Dependency Resolution | Custom | SAT solver |
| Configuration | Bash-like | Portage-compatible |
| Language | Python/Bash | Rust |
| Parallelism | Limited | Full parallel |
| Build Cache | ccache/sccache | Buck2 RE |
| Reproducibility | Partial | Full hermetic |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    User Interface                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  buckos-pkg  │  │buckos-assist │  │ buckos-start │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
├─────────────────────────────────────────────────────────┤
│                    Core Libraries                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │buckos-config │  │ buckos-model │  │ buckos-tools │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
├─────────────────────────────────────────────────────────┤
│                    Build System                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │                    Buck2                          │   │
│  └──────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│                  Package Repository                      │
│  ┌──────────────────────────────────────────────────┐   │
│  │              buckos-build (GitHub)                │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Technical Details

### Dependency Resolution

Buckos uses the Varisat SAT solver for dependency resolution:

- Handles complex version constraints
- Resolves USE flag interactions
- Supports slot-based dependencies
- Detects circular dependencies

### Caching

Multiple layers of caching for performance:

- **Download cache**: Compressed source archives
- **Buck2 cache**: Build artifacts and intermediate outputs
- **Package cache**: Built binary packages

### Database

SQLite-based local package database tracking:

- Installed packages and versions
- File ownership
- Dependencies and reverse dependencies
- USE flags used during build
- Build timestamps

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone and setup
git clone https://github.com/hodgesds/buckos.git
cd buckos

# Install development dependencies
rustup component add rustfmt clippy

# Build and test
cargo build
cargo test
cargo clippy
cargo fmt --check
```

### Areas for Contribution

- Package definitions in [buckos-build](https://github.com/hodgesds/buckos-build)
- Documentation improvements
- New utility tools
- Bug fixes and performance improvements
- Testing and quality assurance

## Roadmap

### Near-term
- [ ] Complete package manager core functionality
- [ ] Implement remaining utility tools
- [ ] Documentation and tutorials
- [ ] Initial package repository population

### Medium-term
- [ ] Binary package distribution
- [ ] Remote build execution
- [ ] Web-based package browser
- [ ] Migration tools from Gentoo

### Long-term
- [ ] Full distribution bootstrap
- [ ] Official installation media
- [ ] Community package repository
- [ ] Enterprise features

## Related Projects

- [Buck2](https://buck2.build/) - Build system
- [Gentoo Linux](https://gentoo.org/) - Inspiration for package management
- [buckos-build](https://github.com/hodgesds/buckos-build) - Package definitions repository

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Community

- **GitHub Issues**: [Report bugs or request features](https://github.com/hodgesds/buckos/issues)
- **Discussions**: [Community discussions](https://github.com/hodgesds/buckos/discussions)

## Acknowledgments

- The Gentoo community for Portage and the USE flag system
- Meta/Facebook for Buck2
- The Rust community for excellent tooling and libraries
