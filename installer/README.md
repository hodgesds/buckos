# buckos-installer

A graphical installer for Buckos - a beginner-friendly system installation tool that guides users through installing Buckos on their system with hardware detection, multiple profiles, and encryption support.

## Overview

buckos-installer provides an intuitive GUI for installing Buckos while maintaining the flexibility for manual installation similar to Gentoo. It's built with egui for a native, cross-platform experience.

## Features

- **Graphical Interface**: Easy-to-use wizard with progress tracking
- **Hardware Detection**: Automatic detection of GPUs, network interfaces, audio devices, and more
- **Hardware-Optimized Kernel**: Dynamic kernel configuration based on detected hardware
- **Kernel Selection**: Choose between LTS, Stable, or Mainline kernel versions
- **Installation Profiles**: Desktop (9 DE choices), Server, Handheld/Gaming, Minimal, Custom
- **Disk Encryption**: LUKS encryption support with multiple encryption schemes
- **Multiple Bootloaders**: GRUB, systemd-boot, rEFInd, Limine, EFISTUB
- **Partition Layouts**: Standard, Btrfs with subvolumes, separate /home, server layout
- **Text Mode**: Command-line instructions for manual installation
- **Dry Run Mode**: Test installation without making changes

## Installation Profiles

### Desktop Profile
Full desktop environment with common applications. Choose from multiple desktop environments:

| Desktop Environment | Description |
|---------------------|-------------|
| GNOME | Modern, user-friendly desktop with GNOME Shell |
| KDE Plasma | Feature-rich desktop with extensive customization |
| Xfce | Lightweight, fast, and traditional desktop |
| MATE | Traditional desktop, continuation of GNOME 2 |
| Cinnamon | Modern desktop with traditional layout |
| LXQt | Lightweight Qt-based desktop environment |
| i3 | Tiling window manager for power users |
| Sway | Wayland compositor compatible with i3 |
| Hyprland | Dynamic tiling Wayland compositor |

### Server Profile
Minimal system with server tools and services. Includes:
- Core system utilities
- SSH server
- Network tools
- System monitoring

### Handheld/Gaming Profile
Optimized for portable gaming devices:
- Steam Deck
- AYA NEO
- GPD Win
- Lenovo Legion Go
- ASUS ROG Ally
- Generic handheld

Includes Steam, Gamescope, and gaming optimizations.

### Minimal Profile
Base system with only essential utilities. Build your system from scratch.

### Custom Profile
Select packages manually after installation.

## Kernel Selection

Choose from three kernel channels:

| Channel | Version | Description |
|---------|---------|-------------|
| LTS (Long-term Support) | 6.6 LTS | Maximum stability, recommended for servers (server-optimized config) |
| Stable | 6.12 | Latest stable kernel, balance of features and stability (default config) |
| Mainline | Latest | Cutting edge features, frequent updates (minimal config) |

Each kernel is automatically optimized based on your detected hardware.

## Bootloader Options

| Bootloader | Boot Mode | Description |
|------------|-----------|-------------|
| GRUB | BIOS/UEFI | Most compatible, many features |
| systemd-boot | UEFI only | Simple, minimal, and fast |
| rEFInd | UEFI only | Graphical boot manager with auto-detection |
| Limine | BIOS/UEFI | Modern bootloader with multiboot support |
| EFISTUB | UEFI only | Boot kernel directly from UEFI (advanced) |

## Disk Configuration

### Partition Layouts

| Layout | Description |
|--------|-------------|
| Standard | Boot/EFI + Swap + Root (recommended) |
| Simple | Single root partition (simplest setup) |
| Separate /home | Standard + separate /home partition |
| Server | Root + /var + /home for servers |
| Btrfs Subvolumes | Btrfs with @, @home, @snapshots subvolumes |
| Custom | Configure partitions manually |

### Encryption Options

| Type | Description |
|------|-------------|
| No Encryption | No disk encryption (fastest) |
| Encrypt Root Only | Encrypt root partition with LUKS |
| Full Disk Encryption | Encrypt all partitions except boot (most secure) |
| Encrypt /home Only | Only encrypt the home partition |

## Hardware Detection

The installer automatically detects your hardware and optimizes the system accordingly:

### Automatic Package Selection
- **GPUs**: NVIDIA, AMD, Intel drivers
- **Network**: WiFi, Ethernet, Bluetooth
- **Audio**: PipeWire, PulseAudio, or ALSA
- **Storage**: NVMe tools
- **Power Management**: Laptop power optimization (TLP, thermald)
- **Virtual Machines**: VirtualBox/VMware/QEMU guest tools
- **CPU Microcode**: Intel/AMD microcode updates
- **Touchscreen**: Input drivers for touch devices
- **Firmware**: linux-firmware for WiFi, Bluetooth, and GPU drivers

### Dynamic Kernel Configuration

The installer generates hardware-specific kernel configuration fragments based on detected hardware:

- **GPU Drivers**: Enables AMDGPU, Nouveau, i915, or VM graphics drivers
- **Network**: Enables WiFi (cfg80211, mac80211) and Ethernet support
- **Storage**: Optimizes for NVMe, AHCI, VirtIO, USB, or RAID controllers
- **Virtual Machines**: Enables hypervisor guest support and paravirtualization
- **Laptop**: Adds ACPI battery, CPU frequency scaling, suspend/hibernate support
- **Bluetooth**: Enables Bluetooth subsystem and HID protocols
- **Touchscreen**: Enables touch input support
- **CPU**: Enables vendor-specific optimizations (Intel P-State, AMD P-State, AES-NI, AVX2)

The kernel config fragment is saved to `hardware-kernel.config` in the buckos-build directory and can be merged with the base kernel config during builds.

## Building

```bash
cd installer
cargo build --release
```

The binary will be available at `target/release/buckos-installer`.

## Usage

### Graphical Mode (Default)

```bash
sudo buckos-installer
```

### Text Mode

For manual installation instructions:

```bash
sudo buckos-installer --text-mode
```

### Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--text-mode` | Run in text-only mode (no GUI) | false |
| `--target <PATH>` | Target root directory for installation | /mnt/buckos |
| `--buckos-build-path <PATH>` | Path to buckos-build repository | Auto-detected |
| `--skip-checks` | Skip system requirements check | false |
| `--debug` | Enable debug logging | false |
| `--dry-run` | Perform a dry run without making changes | false |

### Buckos-Build Repository Detection

The installer needs the buckos-build repository for package definitions. It will automatically search for it in these locations (in order):

1. `/var/db/repos/buckos-build` - Standard Gentoo-style repository location
2. `/usr/share/buckos-build` - System-wide read-only location (typical for live USB)
3. `/opt/buckos-build` - Alternative system location
4. `~/buckos-build` - User home directory
5. `./buckos-build` - Current directory (for development)

If buckos-build is not found in any of these locations, or you want to use a specific version, you can specify the path manually:

```bash
sudo buckos-installer --buckos-build-path /path/to/buckos-build
```

**For Live USB systems**: The buckos-build repository should be pre-installed at `/usr/share/buckos-build` for optimal performance.

### Examples

```bash
# Run installer with custom target directory
sudo buckos-installer --target /mnt/myroot

# Specify custom buckos-build repository location
sudo buckos-installer --buckos-build-path /path/to/buckos-build

# Dry run to preview installation
sudo buckos-installer --dry-run

# Debug mode with custom target and build repo
sudo buckos-installer --debug --target /mnt/buckos --buckos-build-path /opt/buckos-build

# Skip system checks (advanced users)
sudo buckos-installer --skip-checks

# For development with local buckos-build
sudo buckos-installer --buckos-build-path ./buckos-build
```

## Installation Wizard Steps

1. **Welcome** - System information and overview
2. **Hardware Detection** - Detect hardware and suggest drivers/packages
3. **Profile Selection** - Choose installation profile and desktop environment
4. **Disk Setup** - Select disk, partition layout, and encryption
5. **Bootloader** - Choose bootloader type
6. **User Setup** - Configure root password and create user accounts
7. **Network Setup** - Set hostname and network configuration
8. **Timezone & Locale** - Configure timezone, locale, and keyboard layout
9. **Summary** - Review settings with confirmation checkboxes
10. **Installing** - Installation progress
11. **Complete** - Post-installation instructions

## Manual Installation

If you prefer to install manually (similar to Gentoo), use text mode to get the commands:

```bash
buckos-installer --text-mode
```

This displays the manual installation steps:

1. **Partition your disk**:
   ```bash
   fdisk /dev/sdX  # or parted /dev/sdX
   ```

2. **Create filesystems**:
   ```bash
   mkfs.ext4 /dev/sdX1
   mkswap /dev/sdX2
   ```

3. **Set up encryption (optional)**:
   ```bash
   cryptsetup luksFormat /dev/sdX1
   cryptsetup open /dev/sdX1 cryptroot
   mkfs.ext4 /dev/mapper/cryptroot
   ```

4. **Mount the target**:
   ```bash
   mount /dev/sdX1 /mnt/buckos
   ```

5. **Install the base system**:
   ```bash
   buckos --root /mnt/buckos install @system
   ```

6. **Configure the bootloader**:
   ```bash
   chroot /mnt/buckos grub-install /dev/sdX
   chroot /mnt/buckos grub-mkconfig -o /boot/grub/grub.cfg
   ```

7. **Set up users and finalize**:
   ```bash
   chroot /mnt/buckos passwd root
   chroot /mnt/buckos useradd -m -G wheel username
   ```

## System Requirements

### Required Tools

The installer requires these tools to be available:

| Tool | Package |
|------|---------|
| fdisk | util-linux |
| mkfs.ext4 | e2fsprogs |
| mount | util-linux |
| umount | util-linux |
| chroot | coreutils |

### Recommended Tools

| Tool | Package |
|------|---------|
| parted | parted |
| mkfs.btrfs | btrfs-progs |
| mkfs.xfs | xfsprogs |
| grub-install | grub |
| blkid | util-linux |
| lsblk | util-linux |
| cryptsetup | cryptsetup |
| lspci | pciutils |

### Minimum Hardware

- Root privileges required
- UEFI or BIOS boot support
- Sufficient disk space for chosen profile

## Automatic Partitioning

When using automatic partitioning, the installer creates partitions based on the selected layout:

### Standard Layout (UEFI)
- EFI System Partition (512 MB, FAT32)
- Swap Partition (based on RAM size, max 8 GB)
- Root Partition (remaining space, ext4)

### Standard Layout (BIOS)
- BIOS Boot Partition (1 MB)
- Swap Partition (based on RAM size, max 8 GB)
- Root Partition (remaining space, ext4)

### Btrfs Subvolumes
- EFI/BIOS Boot Partition
- Swap Partition
- Btrfs Partition with subvolumes: @, @home, @snapshots

## Post-Installation

After installation, follow these steps:

1. Remove the installation media
2. Reboot your computer
3. Log in with your user account
4. Update package information:
   ```bash
   buckos sync
   ```
5. Update all packages:
   ```bash
   buckos update @world
   ```

## Useful Commands

```bash
buckos search <package>   # Search for packages
buckos install <package>  # Install a package
buckos info <package>     # Show package info
buckos --help             # Show all commands
```

## Dependencies

- eframe / egui - GUI framework
- sysinfo - System information gathering
- nix - POSIX system calls
- clap - Command-line argument parsing
- serde / toml - Configuration serialization
- tokio - Async runtime
- tracing - Logging

## License

Apache-2.0
