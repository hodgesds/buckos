# buckos-assist

Help and assistance utilities for Buckos users and developers.

## Overview

`buckos-assist` provides interactive help, troubleshooting guides, and assistance features for the Buckos operating system. It serves as the primary interface for users seeking help with system configuration, package management, and common issues.

## Features

- **Interactive Help System**: Context-aware help and guidance
- **Troubleshooting Guides**: Step-by-step problem resolution
- **Documentation Browser**: Built-in documentation viewer
- **System Diagnostics**: Automated system health checks
- **Configuration Wizards**: Guided configuration for common tasks

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
buckos-assist = { path = "../assist" }
```

Or install the binary:

```bash
cargo install --path buckos/assist
```

## Planned CLI Usage

### Getting Help

```bash
# Show general help
assist help

# Get help on specific topic
assist help package-management

# Search help topics
assist search "USE flags"

# Interactive help browser
assist browse
```

### Troubleshooting

```bash
# Run system diagnostics
assist diagnose

# Troubleshoot specific issue
assist troubleshoot network

# Check for common problems
assist check

# Generate system report
assist report
```

### Configuration Wizards

```bash
# Run initial system setup
assist setup

# Configure package manager
assist configure package

# Set up network
assist configure network

# Configure services
assist configure services
```

### Documentation

```bash
# Open documentation browser
assist docs

# View specific documentation
assist docs package-management

# Show man page
assist man buckos

# List available documentation
assist docs --list
```

## Planned Features

### Interactive Help

```rust
use buckos_assist::help::HelpSystem;

let help = HelpSystem::new()?;

// Get help for topic
let content = help.get_topic("package-management")?;

// Search help
let results = help.search("how to install packages")?;

// Get context-aware suggestions
let suggestions = help.suggest_for_error("dependency conflict")?;
```

### System Diagnostics

```rust
use buckos_assist::diagnostics::Diagnostics;

let diag = Diagnostics::new()?;

// Run all diagnostics
let report = diag.run_all()?;

// Run specific checks
let disk_check = diag.check_disk_space()?;
let network_check = diag.check_network()?;
let service_check = diag.check_services()?;

// Get recommendations
let recommendations = report.recommendations();
```

### Configuration Wizards

```rust
use buckos_assist::wizard::ConfigWizard;

let wizard = ConfigWizard::new()?;

// Run package configuration wizard
wizard.configure_packages().await?;

// Run network wizard
wizard.configure_network().await?;
```

## Help Topics

### Package Management
- Installing packages
- Removing packages
- Updating system
- Managing USE flags
- Resolving conflicts

### Configuration
- System configuration
- make.conf settings
- Repository setup
- Profile selection

### Services
- Service management
- Creating services
- Troubleshooting services
- Boot configuration

### Network
- Network configuration
- Firewall setup
- DNS configuration
- VPN setup

### Development
- Building packages
- Creating ebuilds
- Contributing to Buckos
- API documentation

## Status

This crate is currently in early development. The basic structure is in place, but most features are still being implemented.

### Roadmap

1. **Phase 1**: Basic help system with static documentation
2. **Phase 2**: Interactive troubleshooting guides
3. **Phase 3**: System diagnostics and health checks
4. **Phase 4**: Configuration wizards
5. **Phase 5**: AI-assisted help (optional)

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.0 | CLI parsing (planned) |
| `dialoguer` | Latest | Interactive prompts (planned) |
| `syntect` | Latest | Syntax highlighting (planned) |
| `pulldown-cmark` | Latest | Markdown rendering (planned) |
| `serde` | 1.0 | Configuration (planned) |

## Contributing

As this crate is in early development, contributions are especially welcome:

1. Help write documentation content
2. Implement troubleshooting guides
3. Add diagnostic checks
4. Create configuration wizards
5. Improve the help search

### Documentation Format

Help topics should be written in Markdown:

```markdown
# Topic Title

Brief description of the topic.

## Overview

Detailed explanation...

## Examples

```bash
# Example command
buckos install www-client/firefox
```

## Troubleshooting

Common issues and solutions...

## See Also

- Related topic 1
- Related topic 2
```

## Testing

```bash
# Run tests
cargo test -p buckos-assist

# Run with test documentation
ASSIST_DOCS=/path/to/test/docs cargo test -p buckos-assist
```

## License

This crate is part of the Buckos project and is licensed under the same terms.

## See Also

- [Buckos Documentation](https://buckos.org/docs)
- [Gentoo Wiki](https://wiki.gentoo.org/) - Many concepts are similar
