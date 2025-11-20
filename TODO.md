# Buckos Package Manager - Feature TODO

This document tracks features required to make Buckos work similar to Gentoo's Portage package manager. Features are organized by priority and can be worked on by multiple agents in parallel.

## Core Package Management

### High Priority

- [ ] **Ebuild-like Build Scripts** - Implement a build script format similar to ebuilds for defining package builds
  - Support for phases: src_unpack, src_prepare, src_configure, src_compile, src_install
  - Variable inheritance and expansion
  - Eclasses for shared build logic
  - Location: `buckos/package/src/ebuild/`

- [ ] **Repository Sync (rsync/git)** - Full repository synchronization
  - Support for rsync, git, and webrsync protocols
  - Incremental updates
  - Repository signature verification
  - Location: `buckos/package/src/repository/`

- [ ] **USE Flag System** - Complete USE flag implementation
  - Global USE flags in make.conf
  - Package-specific USE flags in package.use
  - USE flag dependencies (REQUIRED_USE)
  - USE_EXPAND variables (PYTHON_TARGETS, etc.)
  - Location: `buckos/package/src/use_flags/`

- [ ] **World and System Sets** - Package set management
  - @world - user-selected packages
  - @system - base system packages
  - @selected - manually selected packages
  - Custom package sets
  - Location: `buckos/package/src/sets/`

- [ ] **Slot Support** - Package slotting for multiple versions
  - SLOT and SUBSLOT support
  - Slot dependencies (dev-lang/python:3.11)
  - Slot rebuilds on subslot changes
  - Location: `buckos/package/src/slot/`

### Medium Priority

- [ ] **Profile System** - System profiles for defaults
  - Cascading profiles
  - Profile-specific USE flags
  - Package masking via profiles
  - Architecture-specific profiles
  - Location: `buckos/package/src/profile/`

- [ ] **Masking and Keywords** - Package availability control
  - package.mask / package.unmask
  - ACCEPT_KEYWORDS (stable/testing)
  - ~arch vs arch keywords
  - License-based masking
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

- [ ] **Binary Package Support** - Pre-built binary packages
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

- [ ] **Complete Dependency Types** - All Portage dependency types
  - DEPEND (build dependencies)
  - RDEPEND (runtime dependencies)
  - BDEPEND (build host dependencies)
  - PDEPEND (post dependencies)
  - IDEPEND (install dependencies)
  - Location: `buckos/package/src/resolver/`

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

- [ ] **VDB (Var Database)** - Package installation database
  - /var/db/pkg equivalent
  - CONTENTS file tracking
  - Package metadata storage
  - Atomic updates
  - Location: `buckos/package/src/db/vdb.rs`

- [x] **File Collision Detection** - Prevent overwrites
  - COLLISION_IGNORE
  - Detect and warn on conflicts
  - Handle via blockers
  - Location: `buckos/package/src/db/collision.rs`

### Medium Priority

- [ ] **qfile/equery belongs** - Find package owning file
  - Fast file-to-package lookup
  - Regex support
  - Location: `buckos/package/src/query/owner.rs`

- [ ] **Reverse Dependency Tracking** - Find dependents
  - equery depends equivalent
  - Cached reverse dep graph
  - Location: `buckos/package/src/query/rdeps.rs`

---

## Security

### High Priority

- [x] **GLSA Support** - Security advisories
  - glsa-check equivalent
  - CVE tracking
  - Affected package detection
  - Location: `buckos/package/src/security/glsa.rs`

- [ ] **Package Signing** - Verify package authenticity
  - Manifest signing (GPGKEY)
  - Repository signing
  - gemato support
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

- [ ] **Emerge Output Formatting** - Familiar output
  - Color-coded package status (N/U/R/D)
  - Progress indicators
  - Size estimates
  - Location: `buckos/package/src/ui/`

- [ ] **Pretend Mode** - Show what would happen
  - --pretend (-p) flag
  - Detailed dependency tree
  - Download size estimation
  - Location: Already partially implemented in `main.rs`

### Medium Priority

- [ ] **Interactive Mode** - User prompts
  - --ask (-a) flag
  - Selective package merging
  - Configuration file handling
  - Location: Already partially implemented in `main.rs`

- [ ] **Dispatch-conf** - Configuration file management
  - Three-way merge
  - Interactive diff review
  - Auto-merge trivial changes
  - Location: `buckos/package/src/config/dispatch.rs`

---

## Portage Compatibility Commands

### To Implement

- [ ] `emerge` - Main package management command (current: `buckos install/remove/update`)
- [ ] `equery` - Package querying tool (current: `buckos query/info`)
- [ ] `eclean` - Clean distfiles/packages (current: `buckos clean`)
- [ ] `eselect` - Configuration management tool
- [ ] `etc-update` - Configuration file updates
- [ ] `revdep-rebuild` - Rebuild packages with broken deps
- [ ] `emaint` - Repository maintenance
- [ ] `egencache` - Generate metadata cache

---

## Configuration Files

### To Support

- [ ] `/etc/portage/make.conf` - Main configuration
  - CFLAGS, CXXFLAGS, LDFLAGS
  - USE flags
  - MAKEOPTS
  - FEATURES
  - ACCEPT_KEYWORDS
  - ACCEPT_LICENSE
  - Location: `buckos/package/src/config/`

- [ ] `/etc/portage/package.use` - Per-package USE flags
- [ ] `/etc/portage/package.mask` - Package masking
- [ ] `/etc/portage/package.unmask` - Package unmasking
- [ ] `/etc/portage/package.accept_keywords` - Per-package keywords
- [ ] `/etc/portage/package.license` - License acceptance
- [ ] `/etc/portage/repos.conf` - Repository configuration

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
