# Buckos Codebase Analysis: USE Flag System Implementation

## Executive Summary

This document provides a comprehensive overview of the Buckos codebase, focusing on the current state of USE flag support and what's needed for full implementation.

**Status**: The project has a solid foundation with significant USE flag infrastructure already in place, but the CLI needs enhancement to fully expose and manage USE flags.

---

## 1. Project Structure Overview

### Main Directory Layout
```
buckos/
├── buckos/                    # Main workspace
│   ├── Cargo.toml            # Rust workspace
│   ├── src/main.rs           # Placeholder
│   ├── model/                # Data models
│   ├── package/              # Package manager (PRIMARY)
│   ├── config/               # Configuration system
│   ├── assist/               # System diagnostics
│   ├── boss/                 # Init system (PID 1)
│   ├── tools/                # System utilities
│   └── web/                  # Documentation site
├── build/defs.bzl            # Buck2 build macros
├── platforms/
├── toolchains/
└── third-party/
```

### Key Crates
1. **buckos-package**: Main package manager CLI (`src/main.rs`)
2. **buckos-config**: Configuration management (`src/use_flags.rs`)
3. **buckos-model**: Core data models
4. **buckos-boss**: Init system
5. **buckos-assist**: System diagnostics
6. **buckos-tools**: System utilities

---

## 2. Current USE Flag System Implementation

### 2.1 Configuration Layer (buckos-config crate)

**File**: `/home/user/buckos/buckos/config/src/use_flags.rs` (495 lines)

#### Already Implemented:
- ✅ **UseConfig**: Complete USE flag configuration system
- ✅ **UseFlag**: Individual flag with enable/disable state
- ✅ **PackageUseEntry**: Per-package USE flag overrides
- ✅ **UseFlagDescription**: Flag metadata (name, description, global flag indicator)
- ✅ **UseExpandVariable**: USE_EXPAND variable definitions

#### Features:
- **Global USE flags**: `make.conf` style (`USE="X wayland systemd"`)
- **Per-package USE flags**: `package.use` style
- **USE_EXPAND variables**: 
  - CPU_FLAGS_X86 (30+ instruction set extensions)
  - VIDEO_CARDS (17 video drivers)
  - INPUT_DEVICES (8 input device types)
  - L10N (language codes)
  - PYTHON_TARGETS (3.10-3.13)
  - RUBY_TARGETS (3.1-3.3)
- **USE flag masking**: Mask and force flags
- **Stable USE**: Separate stable mask/force
- **Flag parsing**: "X", "-gtk", "systemd" syntax support
- **Flag merging**: Configuration merging
- **Common flags**: 18+ well-known global flags predefined

#### Common USE Flags Defined:
- **Display**: X, wayland
- **Init**: systemd, elogind
- **Audio**: pulseaudio, pipewire
- **IPC**: dbus
- **Toolkits**: gtk, qt5, qt6
- **Security**: ssl, gnutls
- **Compression**: zstd, lz4
- **Build**: doc, examples, test, debug

### 2.2 Package Manager Layer (buckos-package crate)

**File**: `/home/user/buckos/buckos/package/src/types.rs` (lines 407-422)

#### Simple Implementation:
```rust
pub struct UseConfig {
    pub global: HashSet<String>,
    pub package: BTreeMap<PackageId, HashSet<String>>,
}
```

#### Features:
- ✅ Global and per-package USE flags
- ✅ Flag retrieval per package
- ❌ No USE_EXPAND support
- ❌ No masking/forcing
- ❌ No flag parsing/validation

### 2.3 CLI Layer (buckos-package/src/main.rs)

**File**: `/home/user/buckos/buckos/package/src/main.rs` (1,395 lines)

#### Current CLI Commands:
- ✅ `install` - Install packages
- ✅ `remove/unmerge` - Remove packages
- ✅ `update` - Update packages (@world compatible)
- ✅ `sync` - Sync repositories
- ✅ `search` - Search packages
- ✅ `info` - Package information (WITH USE flags!)
- ✅ `list` - List installed packages
- ✅ `build` - Build packages
- ✅ `clean` - Cache cleanup
- ✅ `verify` - Verify installed packages
- ✅ `query` - Query package database (files, deps, rdeps)
- ✅ `owner` - Find file owner
- ✅ `depgraph` - Dependency graph
- ✅ `config` - Show configuration
- ✅ `depclean` - Remove unused packages
- ✅ `resume` - Resume interrupted operations
- ✅ `newuse` - Rebuild for USE flag changes
- ✅ `audit` - Security vulnerability check

#### USE Flag Support in CLI:
1. **Install command** (InstallArgs struct):
   - `--use-flags` - Comma-separated flags to enable
   - `--disable-use` - Comma-separated flags to disable
   - These are passed to package manager

2. **Global options**:
   - `--newuse` / `-N` - Rebuild on USE flag changes
   - `--tree` / `-t` - Show dependency tree with USE flags
   - Verbose output shows USE flags

3. **Display**:
   - Shows USE flags in package info
   - Shows USE flags in installation list (in verbose mode)
   - Shows USE flag changes in newuse output

#### Current Limitations:
- ❌ No command to set/manage global USE flags
- ❌ No command to modify package-specific USE flags
- ❌ No command to view/list available USE flags
- ❌ No command to show/edit USE_EXPAND variables
- ❌ No command to check USE flag descriptions
- ❌ No interactive USE flag configuration
- ❌ No config file editor/modifier
- ❌ Global flags are only read from config, never modified by CLI

---

## 3. Buck2 Integration

### 3.1 Build Macros (build/defs.bzl)

**File**: `/home/user/buckos/build/defs.bzl` (204 lines)

#### Package Definition:
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
    use_flags = None,        # <-- Already supported!
    slot = "0",
    keywords = None,
    **kwargs
)
```

**Status**:
- ✅ USE flags parameter exists
- ✅ Metadata generation includes USE flags
- ❌ No mechanism to pass runtime USE flags to build
- ❌ No USE flag conditional compilation
- ❌ No USE flag expansion handling

### 3.2 Buck Integration Module

**File**: `/home/user/buckos/buckos/package/src/buck/mod.rs` (120+ lines)

**Current capabilities**:
- ✅ Basic `buck build` execution
- ✅ Job count configuration
- ✅ Release mode support
- ✅ Custom build arguments
- ❌ No USE flag passing to Buck
- ❌ No environment variable setup for USE flags

---

## 4. Configuration Management

### 4.1 Config File Structure

**File**: `/home/user/buckos/buckos/package/src/config.rs` (200 lines)

```rust
pub struct Config {
    pub root: PathBuf,
    pub db_path: PathBuf,
    pub cache_dir: PathBuf,
    pub buck_repo: PathBuf,
    pub buck_path: PathBuf,
    pub parallelism: usize,
    pub repositories: Vec<RepositoryConfig>,
    pub use_flags: UseConfig,           // <-- Integrated!
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

**Features**:
- ✅ TOML-based configuration
- ✅ Default configuration
- ✅ Load from file
- ✅ Save to file
- ✅ System paths management
- ✅ Cache directory management

**Limitations**:
- ❌ No USE flags loading from `/etc/buckos/make.conf`
- ❌ No package.use file loading
- ❌ No configuration validation
- ❌ No configuration merging from multiple sources
- ❌ No environment variable override support

---

## 5. Data Model Layer

### 5.1 Package Types

**File**: `/home/user/buckos/buckos/package/src/types.rs`

#### Key Types:
1. **UseFlag** (lines 154-160):
   - name: String
   - description: String
   - default: bool

2. **UseFlagStatus** (lines 200-204):
   - name: String
   - enabled: bool

3. **UseFlagChange** (lines 233-237):
   - flag: String
   - added: bool

4. **NewusePackage** (lines 240-246):
   - Tracks USE flag changes for rebuilds

5. **ResolvedPackage** (lines 208-221):
   - Includes use_flags vector for resolution display

6. **Dependency** (lines 82-105):
   - use_flags: UseCondition (Always, IfEnabled, IfDisabled, And, Or)
   - Supports conditional dependencies

#### Support for:
- ✅ USE-conditional dependencies
- ✅ USE flag tracking in packages
- ✅ USE change detection
- ✅ USE flag status in resolution

---

## 6. Missing Components

### 6.1 Critical Missing Features

1. **Config File Modifications**
   - No `buckos config` command to modify settings
   - No ability to set global USE flags from CLI
   - No ability to set per-package USE flags from CLI
   - No config file templating or generation

2. **USE Flag Management Commands**
   - No `buckos useflags list` - List all available flags
   - No `buckos useflags info <flag>` - Show flag description
   - No `buckos useflags set <flag>` - Enable/disable globally
   - No `buckos useflags package <pkg> <flags>` - Set per-package flags
   - No `buckos useflags expand` - Show USE_EXPAND definitions

3. **USE_EXPAND Integration**
   - Config system has full USE_EXPAND support
   - CLI has no way to view/set USE_EXPAND variables
   - Buck integration doesn't use them

4. **Build Integration**
   - Buck targets receive USE flag metadata
   - No mechanism to convert CLI flags to build configuration
   - No environment variables set for USE flags during build
   - No USE flag validation against available flags

5. **Configuration Persistence**
   - No make.conf equivalent loading/saving
   - No package.use loading from files
   - No profile system integration
   - No cascading configuration

6. **Validation & Error Handling**
   - No validation module for USE flags
   - No checking against available flags
   - No conflict detection
   - No dependency validation

### 6.2 Enhancement Opportunities

1. **Better Package Info Display**
   - Show which USE flags are currently set
   - Show which USE flags are available
   - Show descriptions of flags
   - Show how flags affect dependencies

2. **Interactive Mode**
   - Interactive USE flag selection during install
   - USE flag editor UI
   - Dependency impact preview

3. **Advanced Features**
   - USE flag profiles (recommended sets)
   - Auto-detection of best USE flags
   - USE flag change history
   - Rollback to previous configurations

---

## 7. File Inventory

### Core Files:

| File | Lines | Purpose | Status |
|------|-------|---------|--------|
| `/home/user/buckos/buckos/package/src/main.rs` | 1,395 | CLI entry point | ✅ Complete, needs enhancements |
| `/home/user/buckos/buckos/package/src/lib.rs` | 1,000+ | Package manager lib | ✅ Core complete |
| `/home/user/buckos/buckos/package/src/types.rs` | 600+ | Data types | ✅ Core types exist |
| `/home/user/buckos/buckos/package/src/config.rs` | 200 | Config management | ✅ Basic complete |
| `/home/user/buckos/buckos/config/src/use_flags.rs` | 495 | USE flag system | ✅ Feature-rich |
| `/home/user/buckos/buckos/config/src/lib.rs` | 132 | Config lib | ✅ Well-documented |
| `/home/user/buckos/build/defs.bzl` | 204 | Buck macros | ✅ Partial support |
| `/home/user/buckos/buckos/package/src/buck/mod.rs` | 120+ | Buck integration | ⚠️ Basic only |

### Submodules in Package/src:
- `buck/` - Buck2 integration
- `cache/` - Artifact caching
- `catalog/` - Package catalog management
- `db/` - SQLite database
- `executor/` - Parallel execution
- `repository/` - Repository management
- `resolver/` - Dependency resolution (SAT solver)
- `transaction/` - Atomic operations
- `validation/` - Data validation

---

## 8. Recommendations for USE Flag System Implementation

### Phase 1: CLI Commands (High Priority)
1. Add `buckos useflags` command group:
   - `list [--package=<pkg>]` - List USE flags
   - `info <flag>` - Show flag details
   - `set <flag>` - Set global flag
   - `unset <flag>` - Unset global flag
   - `package <pkg> <flags>` - Set per-package flags

2. Enhance `buckos config`:
   - Show current USE flags
   - Show configuration file paths
   - Option to reset to defaults

3. Add validation:
   - Validate USE flags against available flags
   - Check for conflicts
   - Warn about unknown flags

### Phase 2: Configuration Integration
1. Load/save from Gentoo-compatible files:
   - `/etc/buckos/make.conf` or TOML equivalent
   - `/etc/buckos/package.use/`

2. Add profile system:
   - Load system profiles
   - Cascade configuration
   - Profile-specific USE flags

3. Implement validation module

### Phase 3: Build System Integration
1. Pass USE flags to Buck builds:
   - Convert to environment variables
   - Pass as build configuration
   - Support USE-conditional rules

2. Implement USE flag constraints:
   - REQUIRED_USE validation
   - Flag conflict detection

### Phase 4: Advanced Features
1. Interactive USE flag selection
2. USE flag profiles
3. Historical tracking
4. Auto-optimization

---

## 9. Current Strengths

1. ✅ **Solid Foundation**: USE flag types and structures already defined
2. ✅ **Rich Config System**: buckos-config has comprehensive USE flag support
3. ✅ **CLI Framework**: Clap-based CLI is well-structured
4. ✅ **Buck Integration**: Metadata system ready
5. ✅ **Emerge Compatibility**: Many Portage features already implemented
6. ✅ **Type Safety**: Strong typing prevents errors
7. ✅ **Comprehensive Docs**: Good inline documentation

---

## 10. Current Gaps

1. ❌ **No USE flag management CLI**
2. ❌ **No config file loading** (USE flags specifically)
3. ❌ **No USE flag validation**
4. ❌ **No build system integration** (USE flags to Buck)
5. ❌ **No interactive configuration**
6. ❌ **Limited use_flags.rs integration** in package manager

---

## 11. Code Examples

### Current USE Flag Structure in Config:
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

pub fn effective_flags(&self, category: &str, name: &str) -> HashSet<String>
```

### Current CLI USE Flag Support:
```rust
#[derive(Args)]
struct InstallArgs {
    #[arg(long, value_delimiter = ',')]
    use_flags: Vec<String>,

    #[arg(long = "disable-use", value_delimiter = ',')]
    disable_use_flags: Vec<String>,
}
```

### Buck Package Definition:
```starlark
def buckos_package(
    name,
    category,
    version,
    use_flags = None,  # Available, but not used at runtime
    ...
)
```

---

## Summary

The Buckos project has a **strong foundation for USE flag support** with comprehensive configuration infrastructure in the `buckos-config` crate. However, the **CLI needs significant enhancement** to expose and manage these features effectively. The work involves:

1. Creating new CLI commands for USE flag management
2. Integrating the rich `buckos-config` system with the package manager
3. Implementing configuration file loading/saving
4. Adding validation and conflict detection
5. Passing USE flags to the Buck2 build system

The infrastructure is solid; it's primarily a matter of CLI and integration work.
