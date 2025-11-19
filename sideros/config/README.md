# sideros-config

System configuration management for Sideros, inspired by Gentoo's Portage configuration system.

## Overview

`sideros-config` provides comprehensive system configuration management following the familiar patterns of Gentoo's `/etc/portage` directory structure. It handles package atoms, USE flags, keywords, licenses, masks, and all other aspects of system configuration.

## Features

- **Portage-Compatible**: Familiar configuration structure for Gentoo users
- **Package Atoms**: Full support for dependency atoms (`>=category/package-1.0:slot::repo`)
- **USE Flag System**: Fine-grained feature control for packages
- **Keyword Management**: Architecture and stability keyword handling
- **License Acceptance**: Granular license approval system
- **Package Masking**: Flexible package masking and unmasking
- **Repository Configuration**: Multi-repository support
- **Profile System**: Hierarchical system profiles
- **Package Sets**: Organized package groups (@world, @system, custom)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sideros-config = { path = "../config" }
```

## Configuration Structure

Sideros uses a configuration structure mirroring Gentoo's `/etc/portage`:

```
/etc/sideros/
├── make.conf                    # Global build settings
├── repos.conf/                  # Repository configuration
│   └── gentoo.conf
├── package.use/                 # Per-package USE flags
│   ├── system
│   └── desktop
├── package.accept_keywords/     # Keyword acceptance
│   └── testing
├── package.license/             # License acceptance
│   └── proprietary
├── package.mask/                # Package masks
│   └── unstable
├── package.unmask/              # Package unmasks
│   └── needed
├── package.env/                 # Per-package environment
│   └── compiler-flags
├── env/                         # Environment definitions
│   └── no-lto.conf
├── sets/                        # Custom package sets
│   ├── @development
│   └── @multimedia
├── profile/                     # Profile overrides
│   └── use.mask
└── world                        # User-selected packages
```

## Modules

### Core Configuration

#### `portage`
Main configuration container that loads and manages the entire system configuration.

```rust
use sideros_config::portage::PortageConfig;

let config = PortageConfig::load("/etc/sideros")?;

// Access global settings
let cflags = config.make_conf.cflags();

// Check USE flags for a package
let use_flags = config.use_flags.for_package("www-client/firefox");
```

#### `make_conf`
Global build settings (CFLAGS, USE, FEATURES, etc.).

```rust
use sideros_config::make_conf::MakeConf;

let conf = MakeConf::load("/etc/sideros/make.conf")?;

// Get compiler flags
let cflags = conf.cflags.as_deref().unwrap_or("-O2 -pipe");
let cxxflags = conf.cxxflags.as_deref().unwrap_or("${CFLAGS}");

// Get USE flags
let use_flags = &conf.use_flags;

// Get features
let features = &conf.features;
```

**Supported Variables:**
- `CFLAGS`, `CXXFLAGS`, `LDFLAGS` - Compiler/linker flags
- `USE` - Global USE flags
- `MAKEOPTS` - Make parallelization
- `FEATURES` - Portage features
- `ACCEPT_KEYWORDS` - Global keyword acceptance
- `ACCEPT_LICENSE` - Global license acceptance
- `GENTOO_MIRRORS` - Download mirrors
- And many more...

### Package Atoms

#### `atom`
Package atom parsing and matching (Gentoo-style dependency specifications).

```rust
use sideros_config::atom::Atom;

// Parse various atom formats
let atom = Atom::parse(">=www-client/firefox-120.0")?;
let atom = Atom::parse("sys-libs/glibc:2.2")?;
let atom = Atom::parse("dev-lang/python:3.11::gentoo")?;

// Check atom properties
assert_eq!(atom.category(), "www-client");
assert_eq!(atom.package(), "firefox");
assert_eq!(atom.version(), Some("120.0"));
assert_eq!(atom.operator(), Some(Operator::GreaterOrEqual));
```

**Supported Atom Formats:**
- `category/package` - Simple package reference
- `>=category/package-1.0` - Version with operator
- `category/package:slot` - Slot specification
- `category/package::repository` - Repository specification
- `=category/package-1.0*` - Glob matching
- `!!category/package` - Hard blocker

**Operators:**
- `=` - Exact version
- `>=` - Greater or equal
- `<=` - Less or equal
- `>` - Greater than
- `<` - Less than
- `~` - Revision match
- `=*` - Glob match

### USE Flags

#### `use_flags`
USE flag management for feature control.

```rust
use sideros_config::use_flags::UseFlags;

let use_flags = UseFlags::load("/etc/sideros/package.use")?;

// Get USE flags for a package
let flags = use_flags.for_atom("www-client/firefox");
// Returns: ["X", "wayland", "-pulseaudio", "dbus"]

// Check if a flag is enabled
let has_wayland = flags.contains("wayland");
```

**USE Flag Syntax:**
```
# /etc/sideros/package.use/desktop
www-client/firefox X wayland -pulseaudio dbus
>=media-video/mpv-0.35 vulkan vaapi
sys-libs/glibc -multilib
```

### Keywords

#### `keywords`
Keyword acceptance for architecture and stability.

```rust
use sideros_config::keywords::Keywords;

let keywords = Keywords::load("/etc/sideros/package.accept_keywords")?;

// Check accepted keywords for a package
let accepted = keywords.for_atom("sys-apps/systemd");
// Returns: ["~amd64"]
```

**Keyword Syntax:**
```
# /etc/sideros/package.accept_keywords/testing
=sys-apps/systemd-254 ~amd64
dev-lang/rust ~amd64
app-misc/neofetch **
```

**Keyword Types:**
- `amd64` - Stable on amd64
- `~amd64` - Testing on amd64
- `-amd64` - Not supported on amd64
- `**` - Accept any keyword
- `~*` - Accept any testing keyword

### Licenses

#### `license`
License acceptance management.

```rust
use sideros_config::license::License;

let licenses = License::load("/etc/sideros/package.license")?;

// Check if a license is accepted for a package
let accepted = licenses.accepts_for_atom("app-misc/proprietary-app", "EULA");
```

**License Syntax:**
```
# /etc/sideros/package.license/proprietary
app-misc/google-chrome google-chrome
media-video/nvidia-drivers NVIDIA-r2
*/* @FREE
```

**License Groups:**
- `@FREE` - All OSI-approved licenses
- `@GPL-COMPATIBLE` - GPL-compatible licenses
- `@BINARY-REDISTRIBUTABLE` - Binary redistribution allowed
- `@EULA` - End user license agreements

### Masking

#### `mask`
Package masking and unmasking.

```rust
use sideros_config::mask::{Mask, Unmask};

// Load masks
let masks = Mask::load("/etc/sideros/package.mask")?;
let unmasks = Unmask::load("/etc/sideros/package.unmask")?;

// Check if a package is masked
let is_masked = masks.is_masked("=sys-apps/systemd-999");

// Check if unmasked
let is_unmasked = unmasks.is_unmasked("=sys-apps/systemd-254");
```

**Mask Syntax:**
```
# /etc/sideros/package.mask/unstable
# Masked due to known bugs
>=sys-apps/systemd-255

# Development versions
=dev-lang/rust-9999
```

### Environment

#### `env`
Per-package environment variable configuration.

```rust
use sideros_config::env::Env;

let env = Env::load("/etc/sideros/package.env", "/etc/sideros/env")?;

// Get environment for a package
let package_env = env.for_atom("sys-devel/gcc");
// Returns environment variables defined in the associated env file
```

**Environment Configuration:**
```
# /etc/sideros/package.env/compiler-flags
sys-devel/gcc no-lto.conf
media-libs/mesa debug.conf

# /etc/sideros/env/no-lto.conf
CFLAGS="${CFLAGS} -fno-lto"
CXXFLAGS="${CXXFLAGS} -fno-lto"
```

### Repositories

#### `repos`
Repository configuration.

```rust
use sideros_config::repos::Repos;

let repos = Repos::load("/etc/sideros/repos.conf")?;

// Get repository information
let gentoo = repos.get("gentoo")?;
println!("Location: {}", gentoo.location);
println!("Sync URI: {}", gentoo.sync_uri);
```

**Repository Configuration:**
```toml
# /etc/sideros/repos.conf/gentoo.conf
[gentoo]
location = /var/db/repos/gentoo
sync-type = git
sync-uri = https://github.com/gentoo-mirror/gentoo.git
auto-sync = yes
priority = -1000

[custom-overlay]
location = /var/db/repos/custom
priority = 50
```

### Profiles

#### `profile`
System profile configuration.

```rust
use sideros_config::profile::Profile;

let profile = Profile::load("/etc/sideros/profile")?;

// Get profile USE masks
let use_mask = profile.use_mask();

// Get profile USE forces
let use_force = profile.use_force();
```

### Package Sets

#### `sets`
Package set management.

```rust
use sideros_config::sets::Sets;

let sets = Sets::load("/etc/sideros/sets")?;

// Get packages in a set
let dev_packages = sets.get("@development")?;
// Returns: ["dev-util/git", "dev-lang/rust", "sys-devel/gcc", ...]

// Built-in sets
let world = sets.get("@world")?;
let system = sets.get("@system")?;
```

**Set Definition:**
```
# /etc/sideros/sets/@development
dev-util/git
dev-lang/rust
dev-lang/python
sys-devel/gcc
sys-devel/make
app-editors/vim
```

### Features

#### `features`
FEATURES configuration for Portage behavior.

```rust
use sideros_config::features::Features;

let features = Features::parse("parallel-fetch ccache -sandbox")?;

// Check if a feature is enabled
let has_ccache = features.is_enabled("ccache");
let has_sandbox = features.is_enabled("sandbox"); // false (disabled)
```

**Common Features:**
- `parallel-fetch` - Download packages in parallel
- `ccache` - Use ccache for compilation
- `distcc` - Distributed compilation
- `sandbox` - Build isolation
- `usersandbox` - User-level sandboxing
- `buildpkg` - Build binary packages
- `getbinpkg` - Use binary packages

### Mirrors

#### `mirrors`
Mirror configuration for package downloads.

```rust
use sideros_config::mirrors::Mirrors;

let mirrors = Mirrors::load("/etc/sideros/mirrors")?;

// Get mirror list
let mirror_urls = mirrors.urls();
```

### Loading

#### `loader`
Configuration loading utilities.

```rust
use sideros_config::loader;

// Load a single configuration file
let content = loader::load_file("/etc/sideros/make.conf")?;

// Load a configuration directory
let entries = loader::load_dir("/etc/sideros/package.use")?;
```

### Errors

#### `error`
Error types for configuration operations.

```rust
use sideros_config::error::ConfigError;

match result {
    Err(ConfigError::ParseError(msg)) => println!("Parse error: {}", msg),
    Err(ConfigError::IoError(e)) => println!("IO error: {}", e),
    Err(ConfigError::InvalidAtom(atom)) => println!("Invalid atom: {}", atom),
    Ok(_) => println!("Success"),
}
```

## Complete Example

```rust
use sideros_config::portage::PortageConfig;
use sideros_config::atom::Atom;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load entire configuration
    let config = PortageConfig::load("/etc/sideros")?;

    // Parse a package atom
    let atom = Atom::parse(">=www-client/firefox-120.0")?;

    // Get USE flags for the package
    let use_flags = config.use_flags.for_atom(&atom);
    println!("USE flags: {:?}", use_flags);

    // Check if package is masked
    if config.masks.is_masked(&atom) {
        println!("Package is masked");

        // Check if unmasked by user
        if config.unmasks.is_unmasked(&atom) {
            println!("But unmasked by user configuration");
        }
    }

    // Get accepted keywords
    let keywords = config.keywords.for_atom(&atom);
    println!("Accepted keywords: {:?}", keywords);

    // Check license acceptance
    if config.licenses.accepts_for_atom(&atom, "MPL-2.0") {
        println!("License accepted");
    }

    Ok(())
}
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization |
| `toml` | 0.8 | TOML parsing |
| `thiserror` | 1.0 | Error handling |
| `glob` | 0.3 | Pattern matching |
| `regex` | 1.10 | Regular expressions |
| `tracing` | 0.1 | Logging |
| `indexmap` | 2.0 | Ordered maps |
| `chrono` | 0.4 | Date/time |

## Migration from Gentoo

If you're migrating from Gentoo, your existing `/etc/portage` configuration should work with minimal changes:

1. Copy your configuration to `/etc/sideros`:
   ```bash
   cp -r /etc/portage /etc/sideros
   ```

2. Update paths in `repos.conf` if needed

3. Review any Gentoo-specific features that may not be supported

## Testing

Run tests for this crate:

```bash
cargo test -p sideros-config
```

## Contributing

When adding new configuration modules:

1. Follow the existing pattern for loaders
2. Support both file and directory configurations
3. Handle graceful fallbacks for missing files
4. Add comprehensive documentation
5. Include unit tests for parsing edge cases

## License

This crate is part of the Sideros project and is licensed under the same terms.

## See Also

- [Gentoo Portage Documentation](https://wiki.gentoo.org/wiki/Portage)
- [make.conf Manual](https://wiki.gentoo.org/wiki//etc/portage/make.conf)
- [Package Atoms](https://wiki.gentoo.org/wiki/Atom)
