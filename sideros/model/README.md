# sideros-model

Core data models and types for the Sideros operating system.

## Overview

`sideros-model` provides the foundational data structures and types used throughout the Sideros ecosystem. This crate serves as the shared vocabulary for all other Sideros crates, ensuring consistent data representation across the system.

## Features

- **Comprehensive Type System**: Rich data models for packages, users, organizations, and more
- **Serialization Support**: Full Serde integration for JSON, TOML, and other formats
- **Time Zone Aware**: Chrono-based datetime handling with timezone support
- **CLI Integration**: Clap derive macros for command-line argument parsing
- **UUID Support**: Unique identifiers for entities

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sideros-model = { path = "../model" }
```

## Modules

### Core Entity Models

#### `user`
User account management types.

```rust
use sideros_model::user::User;

let user = User {
    id: Uuid::new_v4(),
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
    // ...
};
```

#### `person`
Person/individual definitions separate from user accounts.

```rust
use sideros_model::person::Person;

let person = Person {
    name: "Alice Smith".to_string(),
    email: Some("alice@example.com".to_string()),
    // ...
};
```

#### `group`
Group and team definitions for access control.

```rust
use sideros_model::group::Group;

let group = Group {
    name: "developers".to_string(),
    members: vec![user_id1, user_id2],
    // ...
};
```

#### `organization`
Organization structures for multi-tenant systems.

```rust
use sideros_model::organization::Organization;

let org = Organization {
    name: "Acme Corp".to_string(),
    // ...
};
```

### Package Management Models

#### `package`
Package definitions and metadata.

```rust
use sideros_model::package::Package;

let pkg = Package {
    name: "firefox".to_string(),
    category: "www-client".to_string(),
    version: "120.0".to_string(),
    // ...
};
```

Sub-modules:
- `package::build` - Build configuration and instructions
- `package::package` - Core package structures

#### `maintainers`
Package maintainer information.

```rust
use sideros_model::maintainers::Maintainer;

let maintainer = Maintainer {
    name: "John Doe".to_string(),
    email: "john@gentoo.org".to_string(),
    // ...
};
```

### Application Models

#### `application`
Application definitions and metadata.

```rust
use sideros_model::application::Application;

let app = Application {
    name: "MyApp".to_string(),
    version: "1.0.0".to_string(),
    // ...
};
```

### Access Control Models

#### `permission`
Permission and access control types.

```rust
use sideros_model::permission::Permission;

let perm = Permission {
    resource: "file:/etc/passwd".to_string(),
    action: "read".to_string(),
    // ...
};
```

#### `license`
License information for packages and software.

```rust
use sideros_model::license::License;

let license = License {
    name: "MIT".to_string(),
    // ...
};
```

### System Models

#### `system`
System-level types and configurations.

```rust
use sideros_model::system::SystemInfo;

let info = SystemInfo {
    hostname: "server01".to_string(),
    // ...
};
```

#### `profile`
User profile configurations.

```rust
use sideros_model::profile::Profile;

let profile = Profile {
    user_id: user_id,
    preferences: HashMap::new(),
    // ...
};
```

### Tracking and Debugging

#### `bug`
Bug tracking and issue types.

```rust
use sideros_model::bug::Bug;

let bug = Bug {
    id: 12345,
    title: "Application crashes on startup".to_string(),
    // ...
};
```

#### `exception`
Exception and error handling types.

```rust
use sideros_model::exception::Exception;

let exception = Exception {
    message: "Null pointer exception".to_string(),
    stack_trace: vec![],
    // ...
};
```

#### `stack_frame`
Stack trace information for debugging.

```rust
use sideros_model::stack_frame::StackFrame;

let frame = StackFrame {
    function: "main".to_string(),
    file: "src/main.rs".to_string(),
    line: 42,
    // ...
};
```

### Utility Models

#### `action`
Action types for operations and events.

```rust
use sideros_model::action::Action;

let action = Action {
    name: "install".to_string(),
    // ...
};
```

#### `location`
Location and geographic types.

```rust
use sideros_model::location::Location;

let location = Location {
    name: "San Francisco".to_string(),
    // ...
};
```

#### `mapping`
Data mapping utilities for transformations.

#### `news`
News and announcement types.

```rust
use sideros_model::news::NewsItem;

let news = NewsItem {
    title: "New Release".to_string(),
    content: "Version 2.0 is now available!".to_string(),
    // ...
};
```

#### `subscription`
Subscription and notification types.

```rust
use sideros_model::subscription::Subscription;

let sub = Subscription {
    user_id: user_id,
    topic: "security-updates".to_string(),
    // ...
};
```

#### `todo`
Todo and task tracking types.

```rust
use sideros_model::todo::Todo;

let todo = Todo {
    title: "Review PR #123".to_string(),
    completed: false,
    // ...
};
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `chrono` | 0.4.24 | Date and time handling |
| `chrono-tz` | 0.8.2 | Timezone support |
| `clap` | 4.2.7 | CLI argument parsing |
| `serde` | 1.0 | Serialization/deserialization |
| `url` | 2.3.1 | URL parsing and validation |
| `uuid` | 1.3.2 | UUID generation |

## Database Design

This crate follows database normalization principles up to Sixth Normal Form (6NF):

- **Fifth Normal Form (5NF)**: All functional dependencies resolved
- **Sixth Normal Form (6NF)**: Irreducible form with at most one non-key column per row

This design eliminates update anomalies and the null problem, providing a solid foundation for data integrity.

## Usage in Other Crates

`sideros-model` is a foundational crate with no internal dependencies, making it safe to use as a base dependency:

```rust
// In sideros-package
use sideros_model::package::Package;

// In sideros-config
use sideros_model::user::User;

// In sideros-start
use sideros_model::system::SystemInfo;
```

## Examples

### Creating a Package Definition

```rust
use sideros_model::package::{Package, PackageBuild};
use chrono::Utc;

let package = Package {
    name: "nginx".to_string(),
    category: "www-servers".to_string(),
    version: "1.24.0".to_string(),
    description: "High-performance HTTP server".to_string(),
    homepage: "https://nginx.org".to_string(),
    license: "BSD-2".to_string(),
    keywords: vec!["amd64".to_string(), "~arm64".to_string()],
    created_at: Utc::now(),
    // ...
};
```

### Working with Users and Groups

```rust
use sideros_model::user::User;
use sideros_model::group::Group;
use uuid::Uuid;

// Create a user
let user = User {
    id: Uuid::new_v4(),
    username: "webmaster".to_string(),
    // ...
};

// Create a group with the user
let group = Group {
    name: "web-admins".to_string(),
    members: vec![user.id],
    // ...
};
```

### Serialization

```rust
use sideros_model::package::Package;
use serde_json;

let package = Package { /* ... */ };

// Serialize to JSON
let json = serde_json::to_string(&package)?;

// Deserialize from JSON
let package: Package = serde_json::from_str(&json)?;
```

## Testing

Run tests for this crate:

```bash
cargo test -p sideros-model
```

## Contributing

When adding new models:

1. Define the struct with appropriate derives (`Serialize`, `Deserialize`, `Clone`, etc.)
2. Add the module to `lib.rs`
3. Document all public fields and methods
4. Add unit tests for serialization round-trips
5. Consider database normalization principles

## License

This crate is part of the Sideros project and is licensed under the same terms.
