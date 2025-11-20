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

## Current Status

| Component | Status | Description |
|-----------|--------|-------------|
| **buckos-package** | ✅ Complete | Core package manager with full CLI |
| **buckos-config** | ✅ Complete | Portage-compatible configuration system |
| **buckos-boss** | ✅ Complete | Init system and service manager |
| **buckos-model** | ✅ Complete | Core data types and structures |
| **buckos-installer** | ✅ Complete | GUI installer with hardware detection |
| **Build System (defs/)** | ✅ Complete | Eclasses, licenses, EAPI, subslots |
| **buckos-assist** | 🔄 In Progress | System diagnostics and help |
| **buckos-tools** | 🔄 In Progress | System utilities |
| **buckos-web** | 🔄 In Progress | Documentation website |

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
│   ├── package/          # Package manager (buckos)
│   ├── assist/           # System diagnostics
│   ├── config/           # Configuration management
│   ├── boss/             # Init system (PID 1)
│   ├── installer/        # GUI system installer
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

**Binary**: `buckos`

#### Core Commands

```bash
buckos install <package>     # Install a package
buckos remove <package>      # Remove a package (alias: unmerge)
buckos update                # Update installed packages
buckos sync                  # Sync package repositories
buckos search <query>        # Search for packages
buckos info <package>        # Show package information with USE flags
buckos list                  # List installed packages
buckos build <package>       # Build without installing
buckos clean                 # Clean cache (eclean equivalent)
buckos verify                # Verify installed packages
```

#### Advanced Commands

```bash
buckos query deps <pkg>      # Show package dependencies
buckos query rdeps <pkg>     # Show reverse dependencies
buckos query files <pkg>     # List files owned by package
buckos owner <file>          # Find file owner (equery belongs)
buckos depgraph <pkg>        # Show dependency tree visualization
buckos depclean              # Remove unused packages
buckos resume                # Resume interrupted operations
buckos newuse                # Rebuild packages with changed USE flags
buckos audit                 # Security vulnerability check
```

**Shortcuts**:
```bash
buckos deps <pkg>            # Shortcut for query deps
buckos rdeps <pkg>           # Shortcut for query rdeps
```

#### USE Flag Management

```bash
buckos useflags list         # List available USE flags
buckos useflags info <flag>  # Show flag information and description
buckos useflags set <flags>  # Set global USE flags
buckos useflags get          # Get current USE flag configuration
buckos useflags package      # Set per-package USE flags
buckos useflags expand       # Show USE_EXPAND variables
buckos useflags validate     # Validate USE configuration
```

#### System Configuration

```bash
buckos detect                # Detect system hardware and capabilities
buckos configure             # Generate system configuration from templates
buckos config                # Show configuration
buckos set                   # Manage package sets (@world, @system, custom)
buckos patch                 # Manage patches
buckos profile               # Manage system profiles
buckos export                # Export configuration in various formats
```

#### Global Options

| Option | Description |
|--------|-------------|
| `-c, --config <path>` | Configuration file path |
| `-v, --verbose` | Verbose output (stackable: -vv, -vvv) |
| `-q, --quiet` | Quiet output |
| `-p, --pretend` | Dry run mode - show what would be done |
| `-a, --ask` | Ask for confirmation before proceeding |
| `--fetchonly` | Download sources only, don't install |
| `--oneshot` | Don't add package to @world set |
| `-D, --deep` | Update dependencies recursively |
| `-N, --newuse` | Rebuild packages when USE flags change |
| `-t, --tree` | Show dependency tree |
| `-j, --jobs <n>` | Number of parallel jobs |

#### Install Command Options

| Option | Description |
|--------|-------------|
| `--use-flags <flags>` | Enable specific USE flags (comma-separated) |
| `--disable-use <flags>` | Disable specific USE flags |
| `-f, --force` | Force reinstall even if already installed |
| `--nodeps` | Don't install dependencies |
| `-b, --build` | Build from source instead of binary |
| `--noreplace` | Skip if package is already installed |
| `-e, --emptytree` | Empty dependency tree before install |

#### Security Audit

The `audit` command checks installed packages against a comprehensive vulnerability database including:
- CVEs for common packages (OpenSSL, curl, glibc, Linux kernel, OpenSSH, etc.)
- Severity classification (critical, high, medium, low)
- Version range checking with fix recommendations
- Sorted output by severity for prioritization

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

### buckos-boss (Init System)

A modern init system and service manager designed to run as PID 1.

**Binary**: `boss`

**Commands**:
```bash
boss init                   # Run as PID 1
boss start <service>        # Start a service
boss stop <service>         # Stop a service
boss restart <service>      # Restart a service
boss status [service]       # Show service status
boss list                   # List all services
boss enable <service>       # Enable service at boot
boss disable <service>      # Disable service at boot
boss logs <service>         # Show service logs
```

**Features**:
- Service dependency management with parallel startup
- Multiple service types (simple, forking, oneshot, notify, idle)
- Automatic restart with configurable policies
- Smart rate limiting (prevents restart loops)
- Real-time memory/CPU monitoring per service
- Health check support with configurable intervals
- Watchdog support for service monitoring
- Signal handling and zombie reaping
- Boot timing analysis (similar to systemd-analyze)

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

### buckos-installer (GUI Installer)

A beginner-friendly graphical installer for Buckos with hardware detection.

**Binary**: `buckos-installer`

**Commands**:
```bash
buckos-installer                    # Launch GUI installer
buckos-installer --text-mode        # Text-based installation guide
buckos-installer --target /mnt/os   # Set target directory
buckos-installer --dry-run          # Simulate installation
```

**Features**:
- Automatic hardware detection (GPU, network, audio, storage)
- Multiple installation profiles (Minimal, Desktop, Server, Handheld)
- Desktop environment selection (GNOME, KDE, Xfce, i3, Sway, Hyprland, etc.)
- Gaming handheld support (Steam Deck, ROG Ally, Legion Go, etc.)
- Disk layout presets (Simple, Standard, Btrfs subvolumes, Custom)
- LUKS encryption options with TPM support
- Multiple bootloader choices (GRUB, systemd-boot, rEFInd, Limine)
- User and network configuration
- Step-by-step guided installation

### buckos-tools (System Utilities)

A comprehensive collection of system administration and development utilities.

**Binary**: `buckos-tools`

**Commands**:
```bash
buckos-tools lsblk       # List block devices
buckos-tools hwinfo      # Show hardware information
buckos-tools tree        # Display directory tree
buckos-tools envinfo     # Show environment information
buckos-tools netinfo     # Show network interfaces
buckos-tools meminfo     # Show memory information
buckos-tools cpuinfo     # Show CPU information
buckos-tools syscheck    # System health check
buckos-tools diskfree    # Show disk usage
buckos-tools ps          # Show process information
buckos-tools report      # Generate system report
```

**Features**:
- Visual progress bars for memory/CPU usage
- Color-coded health status indicators
- Multiple output formats (text, JSON)
- Process sorting by CPU, memory, or PID

## Build Definition System

### Core Definition Files

Buckos uses Starlark (`.bzl`) files to define build rules and metadata. These are located in the `defs/` directory:

| File | Purpose |
|------|---------|
| `package_defs.bzl` | Package build rules and PackageInfo provider |
| `use_flags.bzl` | USE flag definitions and resolution |
| `registry.bzl` | Central package version registry |
| `versions.bzl` | Version comparison and subslot system |
| `eclasses.bzl` | Eclass inheritance system |
| `licenses.bzl` | License management and groups |
| `eapi.bzl` | EAPI versioning support |
| `package_sets.bzl` | Package set definitions |
| `maintainers.bzl` | Maintainer registry |
| `package_customize.bzl` | Per-package customization |
| `tooling.bzl` | External tool integration and profiles |

### Eclasses

Eclasses provide reusable build patterns (similar to Gentoo eclasses):

```python
load("//defs:eclasses.bzl", "inherit", "eclass_package")

# Inherit from multiple eclasses
config = inherit(["cmake", "xdg"])

# Use in package definition
eclass_package(
    name = "my-app",
    version = "1.0.0",
    eclasses = ["cmake", "xdg"],
    # Phases inherited from eclasses
)
```

**Available Eclasses**:
- `cmake` - CMake-based packages
- `meson` - Meson-based packages
- `autotools` - Traditional configure/make
- `python-single-r1` - Single Python implementation
- `python-r1` - Multiple Python versions
- `go-module` - Go module packages
- `cargo` - Rust/Cargo packages
- `xdg` - Desktop applications
- `linux-mod` - Kernel modules
- `systemd` - Systemd services
- `qt5` / `qt6` - Qt applications

### License System

License management with groups and compliance checking:

```python
load("//defs:licenses.bzl", "check_license", "expand_license_group")

# Check if license is accepted
check_license("GPL-2", ["@FREE"])  # Returns True

# Expand license group
expand_license_group("@FREE")  # Returns list of free licenses
```

**License Groups**:
- `@FREE` - All free software licenses
- `@OSI-APPROVED` - OSI-approved licenses
- `@GPL-COMPATIBLE` - GPL-compatible licenses
- `@COPYLEFT` - Copyleft licenses
- `@PERMISSIVE` - Permissive licenses
- `@BINARY-REDISTRIBUTABLE` - Binary redistribution allowed
- `@FIRMWARE` - Firmware licenses

### EAPI Versioning

Safe evolution of the build API:

```python
load("//defs:eapi.bzl", "eapi_has_feature", "require_eapi")

# Require minimum EAPI
require_eapi(8)

# Check for feature availability
if eapi_has_feature("subslots"):
    # Use subslot-aware dependencies
    pass
```

**Supported EAPI Versions**: 6, 7, 8

### Subslot System

ABI tracking for libraries:

```python
load("//defs:versions.bzl", "subslot_dep", "multi_version_package_with_subslots")

# Subslot-aware dependency
deps = [
    subslot_dep("//packages/linux/dev-libs/openssl", "3", "="),  # Rebuild on ABI change
]

# Define package with subslots
multi_version_package_with_subslots(
    name = "openssl",
    versions = {
        "3.2.0": {"slot": "3/3.2", "status": "stable", ...},
        "3.1.4": {"slot": "3/3.1", "status": "stable", ...},
    },
    default_version = "3.2.0",
)
```

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

### USE_EXPAND Variables

USE_EXPAND variables provide grouped flags for specific subsystems:

```bash
# CPU-specific optimizations
CPU_FLAGS_X86="aes avx avx2 sse4_2"

# Graphics card support
VIDEO_CARDS="intel nvidia"

# Input device support
INPUT_DEVICES="libinput evdev"

# Language/locale support
L10N="en en-US de fr"

# Python target versions
PYTHON_TARGETS="python3_11 python3_12"

# Ruby target versions
RUBY_TARGETS="ruby31 ruby32"
```

**Common USE Flags**:
| Flag | Description |
|------|-------------|
| `wayland` | Wayland display server support |
| `X` | X11 display server support |
| `pulseaudio` | PulseAudio sound server support |
| `systemd` | systemd integration |
| `gtk` | GTK+ GUI toolkit support |
| `qt` | Qt GUI toolkit support |
| `debug` | Build with debugging symbols |
| `doc` | Build documentation |
| `test` | Build and run tests |
| `examples` | Install example code |
| `ssl` | SSL/TLS support |
| `ipv6` | IPv6 networking support |
| `bluetooth` | Bluetooth support |
| `cups` | Printing support via CUPS |
| `zeroconf` | Zeroconf/mDNS support |

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
buckos install ripgrep --use="pcre2"

# This translates to Buck2 config
buck2 build //packages/ripgrep:ripgrep --config //config:use_pcre2=True
```

## Quick Start

```bash
# Install buckos from source
git clone https://github.com/hodgesds/buckos.git && cd buckos
cargo install --path buckos/package

# Initialize and sync
sudo mkdir -p /etc/buckos
buckos --init && buckos sync

# Search and install a package
buckos search ripgrep
buckos install sys-apps/ripgrep --use="pcre2"

# Check what's installed
buckos list
```

## Installation

### Requirements

- Rust 1.70+ (for building from source)
- Buck2 (for package building)
- SQLite 3.x
- Linux kernel 5.x+

### Using the GUI Installer

The recommended way to install Buckos is using the graphical installer:

```bash
# Boot from Buckos installation media
# The installer will start automatically, or run:
buckos-installer
```

The installer guides you through:
1. Hardware detection and driver selection
2. Profile selection (Desktop, Server, Minimal, Handheld)
3. Disk partitioning and encryption
4. Bootloader installation
5. User and network configuration

### Building from Source

```bash
# Clone the repository
git clone https://github.com/hodgesds/buckos.git
cd buckos

# Build all crates
cargo build --release

# Install binaries
cargo install --path buckos/package
cargo install --path buckos/boss
cargo install --path buckos/assist
cargo install --path buckos/tools
```

### Initial Setup

```bash
# Create configuration directory
sudo mkdir -p /etc/buckos

# Copy default configuration
sudo cp -r config/defaults/* /etc/buckos/

# Initialize package database
buckos --init

# Sync repositories
buckos sync

# (Optional) Detect system capabilities and generate optimized config
buckos detect
buckos configure --profile desktop
```

## Usage Examples

### Package Management

```bash
# Sync package repository
buckos sync

# Search for a package
buckos search "web browser"

# Get package information
buckos info www-client/firefox

# Install with specific USE flags
buckos install www-client/firefox --use-flags="wayland,webrtc" --disable-use="dbus"

# Install in pretend mode (dry run)
buckos install -p www-client/firefox

# Update all packages with deep dependency check
buckos update @world -D -N

# Show dependency tree
buckos depgraph www-client/firefox

# Query package dependencies
buckos query deps www-client/firefox
buckos query rdeps dev-libs/openssl

# Find which package owns a file
buckos owner /usr/bin/firefox

# Remove a package and clean unused dependencies
buckos remove www-client/firefox
buckos depclean

# Resume an interrupted operation
buckos resume

# Verify installed packages
buckos verify

# Security audit
buckos audit
```

### USE Flag Management

```bash
# List all available USE flags
buckos useflags list

# Get info about a specific flag
buckos useflags info wayland

# Set global USE flags
buckos useflags set "wayland pulseaudio -systemd"

# Set per-package USE flags
buckos useflags package www-client/firefox "wayland webrtc"

# Show USE_EXPAND configuration
buckos useflags expand

# Validate current configuration
buckos useflags validate
```

### System Configuration

```bash
# Detect system hardware
buckos detect

# Generate optimized configuration
buckos configure --profile desktop

# Show current configuration
buckos config

# Manage package sets
buckos set list
buckos set add mypackages dev-util/ripgrep

# Manage profiles
buckos profile list
buckos profile set default/linux/amd64
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
boss start nginx

# Enable service at boot
boss enable nginx

# Check service status
boss status nginx

# View service logs
boss logs nginx
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
│                    User Interface                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │   buckos     │  │buckos-assist │  │  buckos-boss │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
├─────────────────────────────────────────────────────────┤
│                    Core Libraries                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │buckos-config │  │ buckos-model │  │ buckos-tools │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
├─────────────────────────────────────────────────────────┤
│                    Build System                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │                    Buck2                         │   │
│  └──────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│                  Package Repository                     │
│  ┌──────────────────────────────────────────────────┐   │
│  │              buckos-build (GitHub)               │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Technical Details

### Dependency Resolution

Buckos uses the Varisat SAT solver for dependency resolution:

- Handles complex version constraints
- Resolves USE flag interactions
- Supports slot and subslot dependencies
- ABI tracking via subslots for library packages
- Automatic rebuilds when subslots change
- Detects circular dependencies
- USE-conditional dependency support
- Version conflict handling

### Execution Engine

The parallel execution engine provides high-performance package operations:

- **Multi-threaded downloads**: Concurrent source fetching
- **Parallel builds**: Multiple packages build simultaneously
- **Progress callbacks**: Real-time progress reporting
- **Graceful failure handling**: Proper cleanup on errors

### Transaction Support

Atomic package operations with full rollback capability:

- **Atomic operations**: All-or-nothing package installs
- **Rollback support**: Revert failed operations
- **Operation queuing**: Batch multiple operations
- **Resume capability**: Continue interrupted operations

### Caching

Multiple layers of caching for performance:

- **Download cache**: Compressed source archives
- **Buck2 cache**: Build artifacts and intermediate outputs
- **Package cache**: Built binary packages
- **Binary package support**: Pre-built packages for faster installs

### Repository Management

Flexible multi-repository support:

- **Multiple repositories**: Layer package sources
- **Repository sync**: Various sync methods (git, rsync, http)
- **Metadata loading**: Efficient package metadata caching
- **Priority ordering**: Repository precedence control

### Database

SQLite-based local package database tracking:

- Installed packages and versions
- File ownership records
- Dependencies and reverse dependencies
- USE flags used during build
- Build timestamps and history
- Dependency graph persistence

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

### Completed
- [x] Complete package manager core functionality
- [x] Init system and service manager
- [x] Build definition system (eclasses, EAPI, subslots)
- [x] GUI installer with hardware detection
- [x] Configuration management system

### Near-term
- [ ] Complete system utility tools
- [ ] Documentation and tutorials
- [ ] Initial package repository population
- [ ] System diagnostics assistant

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
