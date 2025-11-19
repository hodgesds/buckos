# sideros-start

A modern init system (PID 1) and service manager for Sideros, providing systemd-like service supervision.

## Overview

`sideros-start` is the initialization daemon for Sideros, responsible for bootstrapping the system, managing services, and supervising processes. It provides a familiar systemd-like interface while being lightweight and designed specifically for Sideros.

## Features

- **PID 1 Init**: Full system initialization and shutdown management
- **Service Supervision**: Automatic restart with configurable policies
- **Dependency Management**: Service dependency ordering and parallel startup
- **Multiple Service Types**: Simple, Forking, Oneshot, Notify, Idle
- **Signal Handling**: Proper handling of SIGCHLD, SIGTERM, SIGINT
- **Zombie Reaping**: Automatic cleanup of orphaned processes
- **Virtual Filesystem**: Automatic mounting of /proc, /sys, /dev, etc.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sideros-start = { path = "../start" }
```

Or install the binary:

```bash
cargo install --path sideros/start
```

## CLI Usage

### Running as Init (PID 1)

```bash
# Boot the system with start as init
start init
```

### Service Management

```bash
# Start a service
start start nginx

# Stop a service
start stop nginx

# Restart a service
start restart nginx

# Check service status
start status nginx

# Check all services status
start status

# List all services
start list
```

### Creating Services

```bash
# Create a new service
start new nginx "/usr/sbin/nginx -g 'daemon off;'"

# Create with output file
start new myapp "/usr/bin/myapp" -o /var/log/myapp.log
```

### System Control

```bash
# Shutdown the system
start shutdown

# Reboot the system
start shutdown --reboot
```

## Service Configuration

Services are defined in TOML files located in `/etc/sideros/services/`.

### Basic Service Definition

```toml
# /etc/sideros/services/nginx.toml

[service]
name = "nginx"
description = "NGINX HTTP Server"
type = "simple"
exec = "/usr/sbin/nginx -g 'daemon off;'"

[service.restart]
policy = "on-failure"
delay = "5s"
max_attempts = 3

[service.dependencies]
after = ["network.target"]
requires = ["network.target"]
```

### Service Types

#### Simple
The main process is the service. Start is complete when the process starts.

```toml
[service]
type = "simple"
exec = "/usr/bin/myapp"
```

#### Forking
The service forks and the parent exits. The child process is the service.

```toml
[service]
type = "forking"
exec = "/usr/sbin/nginx"
pid_file = "/run/nginx.pid"
```

#### Oneshot
Service runs once and exits. Useful for setup scripts.

```toml
[service]
type = "oneshot"
exec = "/usr/bin/setup-network"
remain_after_exit = true
```

#### Notify
Service sends notification when ready via socket.

```toml
[service]
type = "notify"
exec = "/usr/bin/myapp --notify"
notify_socket = "/run/myapp.notify"
```

#### Idle
Service runs when system is idle (all other services started).

```toml
[service]
type = "idle"
exec = "/usr/bin/update-check"
```

### Restart Policies

| Policy | Description |
|--------|-------------|
| `no` | Never restart automatically |
| `on-success` | Restart only on exit code 0 |
| `on-failure` | Restart on non-zero exit |
| `on-abnormal` | Restart on signal or timeout |
| `always` | Always restart regardless of exit status |

### Service States

| State | Description |
|-------|-------------|
| `inactive` | Service is not running |
| `starting` | Service is starting up |
| `active` | Service is running |
| `stopping` | Service is shutting down |
| `failed` | Service has failed |
| `restarting` | Service is restarting |

### Complete Service Example

```toml
# /etc/sideros/services/postgresql.toml

[service]
name = "postgresql"
description = "PostgreSQL Database Server"
type = "notify"
exec = "/usr/bin/postgres -D /var/lib/postgresql/data"
user = "postgres"
group = "postgres"
working_directory = "/var/lib/postgresql"

[service.environment]
PGDATA = "/var/lib/postgresql/data"
PGPORT = "5432"

[service.restart]
policy = "on-failure"
delay = "10s"
max_attempts = 5

[service.timeout]
start = "90s"
stop = "30s"

[service.dependencies]
after = ["network.target", "local-fs.target"]
requires = ["network.target"]
wants = ["syslog.target"]

[service.resource_limits]
memory_max = "4G"
cpu_quota = "80%"
tasks_max = 512
```

## Library Usage

### Basic Service Management

```rust
use sideros_start::manager::ServiceManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ServiceManager::new("/etc/sideros/services")?;

    // Start a service
    manager.start("nginx").await?;

    // Check status
    let status = manager.status("nginx").await?;
    println!("nginx is {:?}", status.state);

    // Stop a service
    manager.stop("nginx").await?;

    Ok(())
}
```

### Running as Init

```rust
use sideros_start::init::Init;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This should only be run as PID 1
    let init = Init::new()?;

    // Boot the system
    init.boot()?;

    // Main loop - handles signals and manages services
    init.run()?;

    Ok(())
}
```

### Custom Service Types

```rust
use sideros_start::service::{Service, ServiceConfig, ServiceType};

let config = ServiceConfig {
    name: "myapp".into(),
    description: "My Application".into(),
    service_type: ServiceType::Simple,
    exec: "/usr/bin/myapp".into(),
    user: Some("appuser".into()),
    group: Some("appgroup".into()),
    ..Default::default()
};

let service = Service::new(config)?;
service.start().await?;
```

### Process Supervision

```rust
use sideros_start::process::ProcessSupervisor;

let supervisor = ProcessSupervisor::new();

// Spawn and supervise a process
let handle = supervisor.spawn("/usr/bin/myapp", &args)?;

// Check if running
if supervisor.is_running(&handle) {
    println!("Process is running");
}

// Terminate
supervisor.terminate(&handle).await?;
```

## Modules

### `init`
Main init system coordinator for PID 1 operations.

```rust
use sideros_start::init::Init;

let init = Init::new()?;
init.mount_virtual_filesystems()?;
init.set_hostname("sideros")?;
init.boot()?;
```

**Responsibilities:**
- Mount virtual filesystems (/proc, /sys, /dev, /run)
- Set hostname
- Initialize random seed
- Start early services
- Manage system state
- Handle shutdown/reboot

### `manager`
Service manager for tracking and controlling services.

```rust
use sideros_start::manager::ServiceManager;

let manager = ServiceManager::new("/etc/sideros/services")?;

// Load all services
manager.load_services()?;

// Get service list
let services = manager.list_services();

// Start all services respecting dependencies
manager.start_all().await?;
```

### `service`
Service types and state management.

```rust
use sideros_start::service::{Service, ServiceState};

// Query state
let state = service.state();
match state {
    ServiceState::Active => println!("Running"),
    ServiceState::Failed => println!("Failed"),
    _ => println!("Other state"),
}
```

### `process`
Process supervision and lifecycle management.

```rust
use sideros_start::process::{Process, RestartPolicy};

let mut process = Process::new(config)?;
process.set_restart_policy(RestartPolicy::OnFailure);
process.start().await?;
```

### `error`
Error types for the init system.

```rust
use sideros_start::error::StartError;

match result {
    Err(StartError::ServiceNotFound(name)) => {
        eprintln!("Service {} not found", name);
    }
    Err(StartError::AlreadyRunning(name)) => {
        eprintln!("Service {} is already running", name);
    }
    _ => {}
}
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.0 | Async runtime |
| `nix` | Latest | Unix system calls |
| `libc` | 0.2 | C library bindings |
| `serde` | 1.0 | Configuration parsing |
| `toml` | Latest | TOML parsing |
| `tracing` | 0.1 | Logging |
| `clap` | 4.0 | CLI parsing |
| `uuid` | Latest | Service identification |
| `chrono` | Latest | Time handling |

## Signal Handling

`sideros-start` handles the following signals:

| Signal | Action |
|--------|--------|
| `SIGCHLD` | Reap zombie processes |
| `SIGTERM` | Graceful shutdown |
| `SIGINT` | Graceful shutdown |
| `SIGHUP` | Reload configuration |
| `SIGUSR1` | Log status |

## Comparison with systemd

| Feature | systemd | sideros-start |
|---------|---------|---------------|
| Language | C | Rust |
| Service Files | INI format | TOML |
| Socket Activation | Yes | Planned |
| cgroups | Yes | Planned |
| Journal | Yes | Standard logs |
| Timers | Yes | Planned |
| Network | networkd | External |

## Boot Process

1. **Mount Virtual Filesystems**
   - `/proc` - Process information
   - `/sys` - Kernel/device information
   - `/dev` - Device nodes
   - `/run` - Runtime data

2. **Initialize System**
   - Set hostname
   - Initialize random seed
   - Set up logging

3. **Start Early Services**
   - udev (device management)
   - syslog

4. **Start Services**
   - Load service definitions
   - Build dependency graph
   - Start services in order

5. **Main Loop**
   - Monitor services
   - Handle signals
   - Reap zombies
   - Restart failed services

## Testing

```bash
# Run all tests
cargo test -p sideros-start

# Run specific test
cargo test -p sideros-start service

# Run with logging
RUST_LOG=debug cargo test -p sideros-start

# Test as non-PID 1 (limited functionality)
cargo run -p sideros-start -- status
```

## Logging

`sideros-start` uses the `tracing` crate for logging:

```bash
# Set log level via environment
RUST_LOG=info start init

# Debug level for troubleshooting
RUST_LOG=debug start init

# Specific module logging
RUST_LOG=sideros_start::service=debug start init
```

## Configuration

### Global Configuration

```toml
# /etc/sideros/start.toml

[init]
# Hostname
hostname = "sideros"

# Default service directory
services_dir = "/etc/sideros/services"

# Runtime directory
runtime_dir = "/run/sideros"

[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log output
output = "journal"  # or "file:/var/log/start.log"

[defaults]
# Default restart delay
restart_delay = "5s"

# Default stop timeout
stop_timeout = "30s"

# Default start timeout
start_timeout = "60s"
```

## Security Considerations

- Services run with dropped privileges when user/group specified
- Resource limits can be set per service
- Process isolation via namespaces (planned)
- Secure defaults for service execution

## Contributing

When contributing to the init system:

1. Be extremely careful with PID 1 code - bugs can be unrecoverable
2. Test thoroughly in VMs
3. Follow Rust safety best practices
4. Document signal handling behavior
5. Consider edge cases in service lifecycle

## License

This crate is part of the Sideros project and is licensed under the same terms.

## See Also

- [systemd Documentation](https://systemd.io/)
- [Init Systems Comparison](https://wiki.gentoo.org/wiki/Comparison_of_init_systems)
- [Linux Signals](https://man7.org/linux/man-pages/man7/signal.7.html)
