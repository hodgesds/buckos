# Buckos Codebase Analysis

## Executive Summary

Buckos is a comprehensive Linux distribution framework built on Buck2 for deterministic, hermetic builds. It implements a Portage-compatible package manager with advanced features including USE flags, slots/subslots, EAPI support, and a SAT-based dependency resolver.

**Current Status**: The project has mature package management, a comprehensive build definition system, hardware-aware installer, and framework components for init system and system utilities.

---

## 1. Project Structure Overview

### Main Directory Layout
```
buckos/
├── buckos/                    # Main Rust workspace
│   ├── Cargo.toml            # Workspace configuration
│   ├── model/                # Core data models (21 modules)
│   ├── package/              # Package manager CLI (PRIMARY)
│   ├── config/               # Configuration system (20 modules)
│   ├── installer/            # GUI installer with hardware detection
│   ├── assist/               # System diagnostics
│   ├── boss/                 # Init system (PID 1)
│   ├── tools/                # System utilities
│   └── web/                  # Documentation site
├── defs/                      # Build definition system
│   ├── eapi.bzl              # EAPI version support (6, 7, 8)
│   ├── eclasses.bzl          # Eclass implementations
│   ├── licenses.bzl          # License definitions
│   ├── versions.bzl          # Subslot/ABI tracking
│   ├── use_flags.bzl         # USE flag definitions
│   ├── package_defs.bzl      # Core package macro
│   ├── package_sets.bzl      # @world, @system sets
│   ├── tooling.bzl           # Tool integration
│   ├── registry.bzl          # Package catalog
│   ├── maintainers.bzl       # Maintainer info
│   └── package_customize.bzl # Per-package customizations
├── build/defs.bzl            # Buck2 build macros
├── platforms/                # Platform configurations
├── toolchains/               # Toolchain definitions
└── third-party/              # External dependencies
```

### Key Crates
1. **buckos-package**: Main package manager CLI (`src/main.rs`)
2. **buckos-config**: Configuration management (`src/use_flags.rs`)
3. **buckos-model**: Core data models
4. **buckos-boss**: Init system
5. **buckos-assist**: System diagnostics
6. **buckos-tools**: System utilities

---

## 2. Build Definition System

### 2.1 Overview

The `defs/` directory contains Starlark build definitions (4,855 total lines) that implement a Portage-compatible package specification system.

### 2.2 EAPI Support (defs/eapi.bzl - 539 lines)

Implements EAPI versions 6, 7, and 8 with version-specific features:

```starlark
# EAPI 8 features
- BDEPEND for build-time dependencies
- Enhanced USE defaults
- Improved fetch restrictions
- SRC_URI arrows for renaming
```

### 2.3 Eclasses (defs/eclasses.bzl - 480 lines)

Provides 11+ eclasses for common build patterns:

| Eclass | Purpose |
|--------|---------|
| `cmake` | CMake build system |
| `meson` | Meson build system |
| `autotools` | GNU Autotools |
| `python-r1` | Python packages |
| `cargo` | Rust/Cargo projects |
| `go-module` | Go modules |
| `kernel-build` | Linux kernel |
| `llvm` | LLVM toolchain |
| `xdg` | XDG specifications |
| `systemd` | Systemd units |
| `git-r3` | Git repositories |

### 2.4 License System (defs/licenses.bzl - 702 lines)

Defines 60+ licenses with metadata:

```starlark
licenses = {
    "GPL-2": {
        "name": "GNU General Public License v2",
        "url": "https://www.gnu.org/licenses/old-licenses/gpl-2.0.html",
        "libre": True,
        "copyleft": True,
    },
    # ... 60+ more licenses
}

# License groups
license_groups = {
    "FREE": [...],
    "EULA": [...],
    "OSI-APPROVED": [...],
}
```

### 2.5 Version/Subslot System (defs/versions.bzl - 626 lines)

Implements slot and subslot support for ABI compatibility tracking:

```starlark
# Slot format: SLOT/SUBSLOT
# Example: "0/1.2" where 0 is slot, 1.2 is ABI version

def slot_operator_deps(dep, slot_op):
    """
    Slot operators:
    - :=  Rebuild when subslot changes
    - :*  Accept any slot
    - :0  Specific slot only
    """
```

### 2.6 USE Flags (defs/use_flags.bzl - 436 lines)

Defines global USE flags and expansion variables:

**Global USE Flags (18+)**:
- Display: `X`, `wayland`
- Init: `systemd`, `elogind`
- Audio: `pulseaudio`, `pipewire`
- Toolkits: `gtk`, `qt5`, `qt6`
- Security: `ssl`, `gnutls`
- Build: `doc`, `examples`, `test`, `debug`

**USE_EXPAND Variables**:
- `CPU_FLAGS_X86`: 30+ instruction set extensions
- `VIDEO_CARDS`: 17 video drivers
- `INPUT_DEVICES`: 8 input device types
- `L10N`: Language codes
- `PYTHON_TARGETS`: 3.10-3.13
- `RUBY_TARGETS`: 3.1-3.3

### 2.7 Package Definitions (defs/package_defs.bzl - 468 lines)

Core `buckos_package()` macro:

```starlark
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
    subslot = None,
    keywords = None,
    eapi = "8",
    eclass = None,
    **kwargs
)
```

### 2.8 Package Sets (defs/package_sets.bzl - 378 lines)

Implements Portage-compatible package sets:

- `@world` - User-selected packages
- `@system` - Base system packages
- `@selected` - Explicitly installed
- Custom sets with pattern matching

---

## 3. Package Manager (buckos-package)

### 3.1 CLI Commands

**File**: `buckos/package/src/main.rs` (1,395+ lines)

| Command | Description | Status |
|---------|-------------|--------|
| `install` | Install packages | ✅ |
| `remove`/`unmerge` | Remove packages | ✅ |
| `update` | Update packages (@world) | ✅ |
| `sync` | Sync repositories | ✅ |
| `search` | Search packages | ✅ |
| `info` | Package information | ✅ |
| `list` | List installed packages | ✅ |
| `build` | Build packages | ✅ |
| `clean` | Cache cleanup | ✅ |
| `verify` | Verify installed packages | ✅ |
| `query` | Query database | ✅ |
| `owner` | Find file owner | ✅ |
| `depgraph` | Dependency graph | ✅ |
| `config` | Show configuration | ✅ |
| `depclean` | Remove unused packages | ✅ |
| `resume` | Resume operations | ✅ |
| `newuse` | Rebuild for USE changes | ✅ |
| `audit` | Security vulnerability check | ✅ |

### 3.2 Submodules

```
buckos/package/src/
├── main.rs           # CLI entry point
├── lib.rs            # Package manager library
├── types.rs          # Data types
├── config.rs         # Configuration
├── buck/             # Buck2 integration
├── cache/            # Artifact caching
├── catalog/          # Package catalog
├── db/               # SQLite database
├── executor/         # Parallel execution
├── repository/       # Repository management
├── resolver/         # SAT-based dependency resolution
├── transaction/      # Atomic operations
└── validation/       # Data validation
```

### 3.3 USE Flag Support

**Install command options**:
```bash
buckos install <pkg> --use-flags=X,wayland --disable-use=gtk
```

**Global options**:
- `--newuse` / `-N` - Rebuild on USE flag changes
- `--tree` / `-t` - Show dependency tree with USE flags

### 3.4 Dependency Resolution

Uses Varisat SAT solver for:
- Conflict resolution
- USE-conditional dependencies
- Slot/subslot constraints
- Circular dependency detection

---

## 4. Configuration System (buckos-config)

### 4.1 Overview

**Directory**: `buckos/config/` (20 modules, 10,000+ lines)

Portage-compatible configuration with full feature support.

### 4.2 USE Flag Configuration

**File**: `buckos/config/src/use_flags.rs` (495 lines)

```rust
pub struct UseConfig {
    pub global: HashSet<String>,
    pub package: Vec<PackageUseEntry>,
    pub expand: IndexMap<String, HashSet<String>>,
    pub mask: HashSet<String>,
    pub force: HashSet<String>,
    pub stable_mask: HashSet<String>,
    pub stable_force: HashSet<String>,
}
```

**Features**:
- ✅ Global USE flags (`make.conf` style)
- ✅ Per-package USE flags (`package.use` style)
- ✅ USE_EXPAND variables
- ✅ USE flag masking and forcing
- ✅ Stable USE mask/force
- ✅ Flag parsing (`X`, `-gtk`, `systemd`)
- ✅ Configuration merging

### 4.3 Package Atoms

Full support for Portage package atom syntax:
- `category/package`
- `>=category/package-1.0`
- `=category/package-1.0*`
- `category/package:slot`
- `category/package[use_flag]`

### 4.4 Configuration Files

| File | Purpose |
|------|---------|
| `make.conf` | Global settings (CFLAGS, USE, etc.) |
| `package.use/*` | Per-package USE flags |
| `package.mask` | Masked packages |
| `package.unmask` | Unmasked packages |
| `package.accept_keywords` | Keyword overrides |
| `repos.conf` | Repository configuration |

### 4.5 Main Config Structure

```rust
pub struct Config {
    pub root: PathBuf,
    pub db_path: PathBuf,
    pub cache_dir: PathBuf,
    pub buck_repo: PathBuf,
    pub buck_path: PathBuf,
    pub parallelism: usize,
    pub repositories: Vec<RepositoryConfig>,
    pub use_flags: UseConfig,
    pub world: WorldSet,
    pub arch: String,
    pub chost: String,
    pub cflags: String,
    pub cxxflags: String,
    pub ldflags: String,
    pub makeopts: String,
    pub features: HashSet<String>,
    pub accept_keywords: HashSet<String>,
    pub accept_license: String,
}
```

---

## 5. Installer (buckos-installer)

### 5.1 Overview

**Directory**: `buckos/installer/` (5 files, ~125 KB)

GUI installation wizard built with egui/eframe.

### 5.2 Hardware Detection

Automatic detection of:
- **GPU**: Vendor, model, driver recommendations
- **Network**: Interfaces, capabilities
- **Audio**: Sound cards, codecs
- **Storage**: Disks, partitions, SMART data

### 5.3 Installation Profiles

| Profile | Description |
|---------|-------------|
| **Desktop** | Full desktop environment |
| **Server** | Headless server |
| **Handheld/Gaming** | Gaming-optimized |
| **Minimal** | Base system only |

### 5.4 Desktop Environments

9 supported desktop environments:
- GNOME
- KDE Plasma
- XFCE
- Cinnamon
- MATE
- LXQt
- Budgie
- Sway (Wayland compositor)
- i3 (tiling WM)

### 5.5 Bootloaders

5 supported bootloaders:
- GRUB
- systemd-boot
- rEFInd
- Limine
- EFISTUB

### 5.6 Disk Configuration

**Partition Layouts**:
- Basic (single root)
- Standard (separate /home)
- Advanced (LVM)
- Btrfs subvolumes
- ZFS datasets
- Manual partitioning

**Encryption**:
- LUKS full-disk encryption
- Encrypted /home only
- Custom encryption schemes

---

## 6. Init System (buckos-start)

### 6.1 Overview

**Directory**: `buckos/start/` (8 files, ~110 KB)

PID 1 init system with service supervision.

### 6.2 Service Types

| Type | Description |
|------|-------------|
| `simple` | Main process stays in foreground |
| `forking` | Traditional daemon fork |
| `oneshot` | Run once at startup |
| `notify` | sd_notify() integration |
| `idle` | Run when system is idle |

### 6.3 Features

- Dependency-based service ordering
- Socket activation
- Watchdog support
- Cgroup integration
- Resource limits
- Service restart policies

### 6.4 Service Definition

```toml
[service]
name = "example"
type = "simple"
exec_start = "/usr/bin/example"
dependencies = ["network.target"]

[service.restart]
policy = "on-failure"
delay = "5s"
```

---

## 7. System Utilities

### 7.1 Assist (buckos-assist)

**Directory**: `buckos/assist/` (6 files, ~43 KB)

System diagnostics and help system:
- Hardware information gathering
- System health checks
- Privacy-controlled reporting
- Troubleshooting guides

### 7.2 Tools (buckos-tools)

**Directory**: `buckos/tools/` (1 file, ~22 KB)

System utility framework for common operations.

---

## 8. Buck2 Integration

### 8.1 Build Macros

**File**: `build/defs.bzl` (204 lines)

Core Buck2 macros for package builds:

```starlark
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
)
```

### 8.2 Buck Integration Module

**File**: `buckos/package/src/buck/mod.rs`

- Basic `buck build` execution
- Job count configuration
- Release mode support
- Custom build arguments

---

## 9. Data Models (buckos-model)

### 9.1 Overview

**Directory**: `buckos/model/` (21 modules)

Foundational data types used across all crates.

### 9.2 Key Types

```rust
// Package identification
pub struct PackageId {
    pub category: String,
    pub name: String,
    pub version: Version,
    pub slot: Slot,
}

// USE flag types
pub struct UseFlag {
    pub name: String,
    pub description: String,
    pub default: bool,
}

pub enum UseCondition {
    Always,
    IfEnabled(String),
    IfDisabled(String),
    And(Vec<UseCondition>),
    Or(Vec<UseCondition>),
}

// Dependency specification
pub struct Dependency {
    pub atom: PackageAtom,
    pub use_flags: UseCondition,
    pub slot_op: Option<SlotOperator>,
}
```

---

## 10. Technology Stack

### Core Technologies

| Component | Technology |
|-----------|------------|
| Language | Rust 2021 Edition |
| Build System | Buck2 |
| Database | SQLite |
| SAT Solver | Varisat |
| Async Runtime | Tokio |
| CLI Parser | Clap |
| GUI Framework | egui/eframe |
| Web Framework | Axum |
| Serialization | Serde (TOML, JSON) |

### External Dependencies

40+ external crates including:
- `tokio` - Async runtime
- `clap` - CLI parsing
- `serde` - Serialization
- `rusqlite` - SQLite bindings
- `varisat` - SAT solver
- `egui`/`eframe` - GUI
- `axum` - Web server
- `reqwest` - HTTP client

---

## 11. Current State Summary

### Fully Implemented

- ✅ Package management (install, remove, update, search)
- ✅ USE flag system (global, per-package, USE_EXPAND)
- ✅ Dependency resolution (SAT solver)
- ✅ Slot/subslot support with ABI tracking
- ✅ EAPI 6, 7, 8 support
- ✅ Eclass system (11+ eclasses)
- ✅ License management (60+ licenses)
- ✅ Package verification and auditing
- ✅ Configuration management (Portage-compatible)
- ✅ Parallel operations
- ✅ Transaction support with rollback
- ✅ Security audit with CVE checking
- ✅ GUI installer with hardware detection
- ✅ Multiple desktop environments
- ✅ Disk encryption support

### Framework Components

- ⚠️ Init system (structure complete, needs testing)
- ⚠️ System diagnostics (basic implementation)
- ⚠️ System utilities (minimal implementation)
- ⚠️ Documentation website (minimal)

### Future Enhancements

1. **Build Integration**
   - Pass USE flags to Buck builds as environment variables
   - USE flag conditional compilation
   - REQUIRED_USE validation

2. **Interactive Features**
   - Interactive USE flag selection
   - Configuration wizard
   - TUI interface

3. **Advanced Features**
   - USE flag profiles
   - Auto-optimization
   - Historical tracking
   - Distributed builds

---

## 12. File Statistics

| Component | Files | Lines |
|-----------|-------|-------|
| Build definitions (defs/) | 11 | 4,855 |
| Package manager | 15+ | 3,000+ |
| Configuration | 20 | 10,000+ |
| Installer | 5 | ~3,000 |
| Init system | 8 | ~3,000 |
| Model | 21 | ~2,000 |
| Total Rust | 70+ | 20,000+ |

---

## 13. Quick Reference

### Common Commands

```bash
# Package management
buckos install app-misc/hello
buckos remove app-misc/hello
buckos update @world
buckos search hello

# With USE flags
buckos install media-video/mpv --use-flags=X,wayland
buckos newuse  # Rebuild for USE changes

# System maintenance
buckos depclean
buckos verify
buckos audit
```

### Configuration Locations

```
/etc/buckos/
├── make.conf           # Global settings
├── package.use/        # Per-package USE flags
├── package.mask        # Masked packages
├── package.unmask      # Unmasked packages
├── package.accept_keywords
└── repos.conf          # Repository configuration
```

---

## Summary

Buckos is a well-architected Linux distribution framework with:

1. **Mature Package Management**: Full Portage compatibility with modern Rust implementation
2. **Comprehensive Build System**: Buck2 integration with rich Starlark definitions
3. **Advanced Configuration**: Complete USE flag, licensing, and EAPI support
4. **Modern Installer**: Hardware-aware GUI with multiple profiles
5. **Framework Components**: Init system and utilities in development

The codebase demonstrates strong type safety, good documentation, and clean separation of concerns. Primary development focus should be on completing the init system and improving build integration for USE flags.
