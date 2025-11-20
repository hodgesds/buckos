# Buckos Package Manager - Feature TODO

This document tracks features required to make Buckos work similar to Gentoo's Portage package manager. Features are organized by priority and can be worked on by multiple agents in parallel.

## Implementation Status Summary

**Core Package Management:** 5/6 high priority complete, 5/5 medium priority complete
- Repository sync, USE flags, package sets, slots all implemented
- Masking and keywords system implemented (package.mask/unmask, ACCEPT_KEYWORDS, license-based masking)
- Missing: Ebuild-like build scripts

**Dependency Resolution:** 5/5 high priority complete, 2/2 medium priority complete
- SAT solver, virtual packages, blockers, circular deps, backtracking, autounmask all implemented

**Build System:** 2/3 high priority complete, 0/2 medium priority complete
- Sandbox and distfile management implemented
- Missing: Parallel building with load average, FEATURES flags, cross-compilation

**Database & Querying:** 4/4 complete
- SQLite VDB, file collision detection, file ownership, reverse dependencies

**Security:** 2/2 high priority complete, 0/1 medium priority complete
- GLSA support implemented
- Package signing implemented (GPG key management, Manifest signing, repository signing)
- Missing: Hardened support

**User Interface:** 4/4 complete
- Emerge output formatting, pretend mode, interactive mode all implemented

**CLI Commands:** ~26 Portage-compatible commands implemented (including revdep-rebuild and package signing)

---

## Core Package Management

### High Priority

- [ ] **Ebuild-like Build Scripts** - Implement a build script format similar to ebuilds for defining package builds
  - Support for phases: src_unpack, src_prepare, src_configure, src_compile, src_install
  - Variable inheritance and expansion
  - Eclasses for shared build logic
  - Location: `buckos/package/src/ebuild/`

- [x] **Repository Sync (rsync/git)** - Full repository synchronization
  - Support for rsync, git, HTTP, and local protocols
  - Incremental updates via git pull
  - Webrsync mode support
  - Location: `buckos/package/src/repository/mod.rs`

- [x] **USE Flag System** - Complete USE flag implementation
  - Global USE flags in make.conf
  - Package-specific USE flags in package.use
  - USE flag dependencies (REQUIRED_USE)
  - USE_EXPAND variables (CPU_FLAGS, VIDEO_CARDS, PYTHON_TARGETS, etc.)
  - CLI: `buckos useflags` with list/info/set/get/package/expand/validate subcommands
  - Location: `buckos/package/src/main.rs`, `buckos/config/src/`

- [x] **World and System Sets** - Package set management
  - @world - user-selected packages (get_world_set)
  - @system - base system packages (get_system_set)
  - @selected - combined world + system (get_selected_set)
  - Custom package sets via `buckos set` command
  - Location: `buckos/package/src/lib.rs:407-449`

- [x] **Slot Support** - Package slotting for multiple versions
  - SLOT and SUBSLOT support in database schema
  - Slot dependencies (dev-lang/python:3.11) in PackageSpec
  - Slot-aware dependency resolution
  - Location: `buckos/package/src/types.rs`, `buckos/package/src/db/mod.rs`

### Medium Priority

- [ ] **Profile System** - System profiles for defaults
  - Cascading profiles
  - Profile-specific USE flags
  - Package masking via profiles
  - Architecture-specific profiles
  - Location: `buckos/package/src/profile/`

- [x] **Masking and Keywords** - Package availability control
  - package.mask / package.unmask
  - ACCEPT_KEYWORDS (stable/testing)
  - ~arch vs arch keywords
  - License-based masking
  - Autounmask suggestions
  - License groups (@FREE, @OSI-APPROVED, @COPYLEFT, etc.)
  - Location: `buckos/package/src/mask/`

- [x] **Preserved Libraries** - Handle shared library transitions
  - Track libraries in use by other packages
  - Preserve old libraries during upgrades
  - Rebuild dependents when safe
  - Location: `buckos/package/src/preserved_libs/`

- [x] **Configuration Protection** - Protect user config files
  - CONFIG_PROTECT and CONFIG_PROTECT_MASK
  - etc-update / dispatch-conf functionality
  - ._cfg0000_ file management
  - Location: `buckos/package/src/config_protect/`

- [x] **Binary Package Support** - Pre-built binary packages
  - PKGDIR for binary package storage
  - binpkg-multi-instance support
  - Binary package signing
  - --getbinpkg and --usepkg flags
  - Location: `buckos/package/src/binary/`

### Lower Priority

- [ ] **Overlay Support** - Additional package repositories
  - Local overlays
  - layman/eselect-repository equivalent
  - Repository priorities
  - Location: `buckos/package/src/overlay/`

- [x] **News System** - Important notifications
  - GLEP 42 news items
  - Read/unread tracking
  - eselect news equivalent
  - Location: `buckos/package/src/news/`

---

## Dependency Resolution

### High Priority

- [x] **Complete Dependency Types** - All Portage dependency types
  - DEPEND (build dependencies) - `build_dependencies`
  - RDEPEND (runtime dependencies) - `runtime_dependencies`
  - BDEPEND (build host dependencies) - supported via build_time flag
  - Dependencies with slot, USE conditions, version specs
  - SAT solver (varisat) for complex constraint resolution
  - Location: `buckos/package/src/resolver/mod.rs`, `buckos/package/src/types.rs`

- [x] **Virtual Packages** - Provider abstraction
  - virtual/* category support
  - Provider selection
  - Default provider configuration
  - Location: `buckos/package/src/virtual/`

- [x] **Blockers** - Package conflicts
  - Hard blockers (!!category/package)
  - Soft blockers (!category/package)
  - Automatic blocker resolution
  - Location: `buckos/package/src/resolver/blocker.rs`

- [x] **Circular Dependency Handling** - Break dependency cycles
  - Detection of circular deps
  - Bootstrap package support
  - USE-conditional dep breaking
  - Location: `buckos/package/src/resolver/circular.rs`

### Medium Priority

- [x] **Backtracking** - Better dependency solving
  - Backtrack on conflicts
  - --backtrack option
  - Intelligent retry strategies
  - Location: `buckos/package/src/resolver/backtrack.rs`

- [x] **Autounmask** - Automatic keyword/USE adjustments
  - --autounmask
  - --autounmask-write
  - User confirmation for changes
  - Location: `buckos/package/src/resolver/autounmask.rs`

---

## Build System

### High Priority

- [x] **Sandbox Support** - Isolated builds
  - FEATURES="sandbox"
  - Network isolation
  - Filesystem access control
  - Location: `buckos/package/src/sandbox/`

- [ ] **Parallel Building** - Efficient builds
  - --jobs support
  - MAKEOPTS propagation
  - Load average limiting
  - Location: `buckos/package/src/build/parallel.rs`

- [x] **Distfile Management** - Source downloads
  - SRC_URI handling with mirrors
  - RESTRICT="fetch" support
  - Checksum verification (BLAKE2B, SHA512)
  - Mirror selection and fallback
  - Location: `buckos/package/src/distfile/`

### Medium Priority

- [ ] **FEATURES Flags** - Build behavior control
  - test, doc, ccache, distcc
  - split-log, parallel-fetch
  - binpkg-logs, unmerge-orphans
  - Location: `buckos/package/src/features/`

- [ ] **Cross-Compilation** - Build for other architectures
  - CBUILD, CHOST, CTARGET
  - Sysroot support
  - Cross-toolchain management
  - Location: `buckos/package/src/cross/`

---

## Database & Querying

### High Priority

- [x] **VDB (Var Database)** - Package installation database
  - SQLite-based database (packages.db)
  - CONTENTS file tracking with blake3 hashes
  - Package metadata storage (version, slot, USE flags, size)
  - Atomic updates with transaction support (BEGIN/COMMIT/ROLLBACK)
  - Location: `buckos/package/src/db/mod.rs`

- [x] **File Collision Detection** - Prevent overwrites
  - COLLISION_IGNORE
  - Detect and warn on conflicts
  - Handle via blockers
  - Location: `buckos/package/src/db/collision.rs`

### Medium Priority

- [x] **qfile/equery belongs** - Find package owning file
  - Fast file-to-package lookup via SQLite index
  - Pattern matching support (find_file_owners_by_pattern)
  - CLI: `buckos owner <path>`
  - Location: `buckos/package/src/lib.rs:853-896`, `buckos/package/src/db/mod.rs:377-388`

- [x] **Reverse Dependency Tracking** - Find dependents
  - equery depends equivalent
  - Database-backed reverse dependency queries
  - CLI: `buckos rdeps <package>` and `buckos query rdeps <package>`
  - Location: `buckos/package/src/db/mod.rs:342-356`, `buckos/package/src/lib.rs:847-850`

---

## Security

### High Priority

- [x] **GLSA Support** - Security advisories
  - glsa-check equivalent
  - CVE tracking
  - Affected package detection
  - Location: `buckos/package/src/security/glsa.rs`

- [x] **Package Signing** - Verify package authenticity
  - Manifest signing (GPGKEY)
  - Repository signing
  - gemato support
  - GPG key management (import/export/trust)
  - CLI: `buckos sign` with list-keys/import-key/sign-manifest/verify-manifest/sign-repo/verify-repo subcommands
  - Location: `buckos/package/src/security/signing.rs`

### Medium Priority

- [ ] **Hardened Support** - Security-focused builds
  - PIE, SSP, RELRO
  - PaX flags (if applicable)
  - Security-focused CFLAGS
  - Location: `buckos/package/src/security/hardened.rs`

---

## User Interface

### High Priority

- [x] **Emerge Output Formatting** - Familiar output
  - Color-coded package status (N/U/R/D) using console crate
  - Progress indicators for downloads and builds
  - Size estimates (download and install)
  - USE flag display with +enabled/-disabled
  - Location: `buckos/package/src/main.rs`

- [x] **Pretend Mode** - Show what would happen
  - --pretend (-p) global flag
  - Detailed dependency tree (--tree flag)
  - Download and install size estimation
  - Shows resolved packages without executing
  - Location: `buckos/package/src/main.rs:42-43`

### Medium Priority

- [x] **Interactive Mode** - User prompts
  - --ask (-a) global flag
  - Confirmation prompts using dialoguer crate
  - Configuration file handling
  - Location: `buckos/package/src/main.rs:46-47`

- [ ] **Dispatch-conf** - Configuration file management
  - Three-way merge
  - Interactive diff review
  - Auto-merge trivial changes
  - Location: `buckos/package/src/config/dispatch.rs`

---

## Portage Compatibility Commands

### Implemented

- [x] `emerge` - Main package management command
  - `buckos install` - install packages with --pretend, --ask, --fetchonly, --oneshot, --deep, --newuse
  - `buckos remove` (alias: unmerge) - remove packages
  - `buckos update` - update @world with sync
- [x] `equery` - Package querying tool
  - `buckos query files/deps/rdeps` - query subcommands
  - `buckos info` - package information
  - `buckos owner` - find file owner (equery belongs)
  - `buckos depgraph` - show dependency tree
- [x] `eclean` - Clean distfiles/packages
  - `buckos clean --all/--downloads/--builds`
- [x] `glsa-check` - Security advisory checks
  - `buckos audit` - check for vulnerabilities
- [x] Package signing and verification
  - `buckos sign` - manage keys, sign/verify manifests and repositories
- [x] `emerge --depclean` - Remove unused packages
  - `buckos depclean`
- [x] `emerge --resume` - Resume interrupted operations
  - `buckos resume`
- [x] `emerge --newuse` - Rebuild for USE flag changes
  - `buckos newuse`

### To Implement

- [ ] `eselect` - Configuration management tool
- [ ] `etc-update` / `dispatch-conf` - Configuration file updates
- [x] `revdep-rebuild` - Rebuild packages with broken library dependencies
  - `buckos revdep` - scan for broken deps and rebuild
  - Supports --pretend, --library, --ignore options
  - Uses readelf to detect missing shared libraries
- [ ] `emaint` - Repository maintenance
- [ ] `egencache` - Generate metadata cache

---

## Configuration Files

### Implemented

- [x] `/etc/portage/make.conf` - Main configuration
  - CFLAGS, CXXFLAGS, LDFLAGS
  - USE flags (global)
  - MAKEOPTS
  - FEATURES
  - ACCEPT_KEYWORDS
  - ACCEPT_LICENSE
  - Buck2 integration settings
  - Location: `buckos/config/src/`

- [x] `/etc/portage/package.use` - Per-package USE flags
- [x] `/etc/portage/package.mask` - Package masking
- [x] `/etc/portage/package.unmask` - Package unmasking
- [x] `/etc/portage/package.accept_keywords` - Per-package keywords
- [x] `/etc/portage/package.license` - License acceptance
- [x] `/etc/portage/repos.conf` - Repository configuration with multi-repo support

---

## Testing & Quality

### To Implement

- [ ] **Test Suite** - Comprehensive testing
  - Unit tests for all modules
  - Integration tests for workflows
  - Dependency resolution test cases
  - Location: `buckos/package/tests/`

- [ ] **Repoman Equivalent** - Package QA checking
  - Ebuild syntax checking
  - Dependency completeness
  - Metadata validation
  - Location: `buckos/package/src/qa/`

---

## Documentation

### To Create

- [ ] Man pages for all commands
- [ ] Configuration file documentation
- [ ] Ebuild writing guide
- [ ] Migration guide from Portage
- [ ] API documentation for library users

---

## Notes for Contributors

1. Each feature should be developed in its own module
2. Follow existing code patterns in `buckos/package/src/`
3. Add tests for all new functionality
4. Update relevant README files when adding features
5. Use the SAT solver (`varisat`) for dependency resolution

## Agent Assignment

Agents can claim features by adding their identifier next to the checkbox:
- `- [ ] @agent-name Feature description`

When a feature is complete:
- `- [x] @agent-name Feature description`
