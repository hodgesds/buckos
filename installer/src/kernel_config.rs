//! Hardware-to-Kernel-Config mapping system
//!
//! This module maps detected hardware to Linux kernel configuration options,
//! enabling hardware-optimized kernel builds.

use crate::types::{GpuVendor, HardwareInfo, NetworkInterfaceType, StorageControllerType};

/// Represents a kernel configuration fragment
#[derive(Debug, Clone)]
pub struct KernelConfigFragment {
    #[allow(dead_code)]
    pub name: String,
    pub description: String,
    pub config_options: Vec<ConfigOption>,
}

/// Represents a single kernel config option
#[derive(Debug, Clone)]
pub struct ConfigOption {
    pub key: String,
    pub value: ConfigValue,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Yes,
    Module,
    #[allow(dead_code)]
    No,
    #[allow(dead_code)]
    String(String),
    #[allow(dead_code)]
    Number(i32),
}

impl ConfigValue {
    pub fn to_config_string(&self) -> String {
        match self {
            ConfigValue::Yes => "y".to_string(),
            ConfigValue::Module => "m".to_string(),
            ConfigValue::No => "n".to_string(),
            ConfigValue::String(s) => format!("\"{}\"", s),
            ConfigValue::Number(n) => n.to_string(),
        }
    }
}

/// Generate kernel config fragments based on detected hardware
pub fn generate_hardware_config_fragments(hardware: &HardwareInfo) -> Vec<KernelConfigFragment> {
    let mut fragments = Vec::new();

    // GPU-specific configuration
    for gpu in &hardware.gpus {
        fragments.push(generate_gpu_config(&gpu.vendor));
    }

    // Network interface configuration
    if !hardware.network_interfaces.is_empty() {
        fragments.push(generate_network_config(&hardware.network_interfaces));
    }

    // Storage controller configuration
    fragments.push(generate_storage_config(&hardware.storage_controller));

    // Virtual machine optimizations
    if hardware.is_virtual_machine {
        fragments.push(generate_vm_config());
    }

    // Laptop/portable device optimizations
    if hardware.is_laptop {
        fragments.push(generate_laptop_config());
    }

    // Bluetooth support
    if hardware.has_bluetooth {
        fragments.push(generate_bluetooth_config());
    }

    // Touchscreen support
    if hardware.has_touchscreen {
        fragments.push(generate_touchscreen_config());
    }

    // CPU-specific optimizations
    if !hardware.cpu_vendor.is_empty() {
        fragments.push(generate_cpu_config(&hardware.cpu_vendor, &hardware.cpu_flags));
    }

    fragments
}

/// Generate GPU-specific kernel config
fn generate_gpu_config(vendor: &GpuVendor) -> KernelConfigFragment {
    let (name, description, options) = match vendor {
        GpuVendor::Amd => (
            "amd-gpu",
            "AMD GPU support (AMDGPU driver)",
            vec![
                ConfigOption {
                    key: "CONFIG_DRM_AMDGPU".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("AMD GPU driver".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_DRM_AMDGPU_SI".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Southern Islands support".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_DRM_AMDGPU_CIK".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Sea Islands support".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_DRM_AMD_DC".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Display Core driver".to_string()),
                },
            ],
        ),
        GpuVendor::Nvidia => (
            "nvidia-gpu",
            "NVIDIA GPU support (Nouveau driver)",
            vec![
                ConfigOption {
                    key: "CONFIG_DRM_NOUVEAU".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("Nouveau driver".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_NOUVEAU_LEGACY_CTX_SUPPORT".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Legacy context support".to_string()),
                },
            ],
        ),
        GpuVendor::Intel => (
            "intel-gpu",
            "Intel GPU support (i915 driver)",
            vec![
                ConfigOption {
                    key: "CONFIG_DRM_I915".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("Intel i915 driver".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_DRM_I915_GVT".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Intel GVT-g support".to_string()),
                },
            ],
        ),
        GpuVendor::VirtualBox => (
            "vbox-gpu",
            "VirtualBox graphics support",
            vec![
                ConfigOption {
                    key: "CONFIG_DRM_VBOXVIDEO".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("VirtualBox graphics".to_string()),
                },
            ],
        ),
        GpuVendor::VMware => (
            "vmware-gpu",
            "VMware graphics support",
            vec![
                ConfigOption {
                    key: "CONFIG_DRM_VMWGFX".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("VMware SVGA driver".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_DRM_VMWGFX_FBCON".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Framebuffer console support".to_string()),
                },
            ],
        ),
        GpuVendor::Unknown => (
            "generic-gpu",
            "Generic GPU support",
            vec![
                ConfigOption {
                    key: "CONFIG_DRM_FBDEV_EMULATION".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Framebuffer emulation".to_string()),
                },
            ],
        ),
    };

    KernelConfigFragment {
        name: name.to_string(),
        description: description.to_string(),
        config_options: options,
    }
}

/// Generate network interface configuration
fn generate_network_config(interfaces: &[crate::types::NetworkInterfaceInfo]) -> KernelConfigFragment {
    let mut options = vec![
        ConfigOption {
            key: "CONFIG_NETDEVICES".to_string(),
            value: ConfigValue::Yes,
            comment: Some("Network device support".to_string()),
        },
    ];

    // Check for WiFi interfaces
    let has_wifi = interfaces.iter().any(|i| i.interface_type == NetworkInterfaceType::Wifi);
    if has_wifi {
        options.extend(vec![
            ConfigOption {
                key: "CONFIG_WLAN".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Wireless LAN".to_string()),
            },
            ConfigOption {
                key: "CONFIG_CFG80211".to_string(),
                value: ConfigValue::Module,
                comment: Some("cfg80211 wireless configuration".to_string()),
            },
            ConfigOption {
                key: "CONFIG_MAC80211".to_string(),
                value: ConfigValue::Module,
                comment: Some("Generic IEEE 802.11".to_string()),
            },
        ]);
    }

    // Ethernet is almost always present
    options.push(ConfigOption {
        key: "CONFIG_ETHERNET".to_string(),
        value: ConfigValue::Yes,
        comment: Some("Ethernet driver support".to_string()),
    });

    KernelConfigFragment {
        name: "network".to_string(),
        description: "Network interface support".to_string(),
        config_options: options,
    }
}

/// Generate storage controller configuration
fn generate_storage_config(controller: &StorageControllerType) -> KernelConfigFragment {
    let (name, description, options) = match controller {
        StorageControllerType::Nvme => (
            "nvme-storage",
            "NVMe storage support",
            vec![
                ConfigOption {
                    key: "CONFIG_BLK_DEV_NVME".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("NVMe block device".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_NVME_MULTIPATH".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("NVMe multipath support".to_string()),
                },
            ],
        ),
        StorageControllerType::Ahci => (
            "ahci-storage",
            "AHCI/SATA storage support",
            vec![
                ConfigOption {
                    key: "CONFIG_SATA_AHCI".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("AHCI SATA support".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_ATA".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("ATA/ATAPI/MFM/RLL support".to_string()),
                },
            ],
        ),
        StorageControllerType::Virtio => (
            "virtio-storage",
            "VirtIO storage support",
            vec![
                ConfigOption {
                    key: "CONFIG_VIRTIO_BLK".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("VirtIO block driver".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_SCSI_VIRTIO".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("VirtIO SCSI driver".to_string()),
                },
            ],
        ),
        StorageControllerType::Usb => (
            "usb-storage",
            "USB storage support",
            vec![
                ConfigOption {
                    key: "CONFIG_USB_STORAGE".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("USB mass storage".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_USB_UAS".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("USB attached SCSI".to_string()),
                },
            ],
        ),
        StorageControllerType::Raid => (
            "raid-storage",
            "RAID storage support",
            vec![
                ConfigOption {
                    key: "CONFIG_MD".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("Multiple device support".to_string()),
                },
                ConfigOption {
                    key: "CONFIG_BLK_DEV_MD".to_string(),
                    value: ConfigValue::Module,
                    comment: Some("RAID support".to_string()),
                },
            ],
        ),
        StorageControllerType::Unknown => (
            "generic-storage",
            "Generic storage support",
            vec![
                ConfigOption {
                    key: "CONFIG_SCSI".to_string(),
                    value: ConfigValue::Yes,
                    comment: Some("SCSI device support".to_string()),
                },
            ],
        ),
    };

    KernelConfigFragment {
        name: name.to_string(),
        description: description.to_string(),
        config_options: options,
    }
}

/// Generate VM-specific optimizations
fn generate_vm_config() -> KernelConfigFragment {
    KernelConfigFragment {
        name: "vm-optimizations".to_string(),
        description: "Virtual machine optimizations".to_string(),
        config_options: vec![
            ConfigOption {
                key: "CONFIG_HYPERVISOR_GUEST".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Hypervisor guest support".to_string()),
            },
            ConfigOption {
                key: "CONFIG_PARAVIRT".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Enable paravirtualization".to_string()),
            },
            ConfigOption {
                key: "CONFIG_VIRTIO".to_string(),
                value: ConfigValue::Yes,
                comment: Some("VirtIO drivers".to_string()),
            },
            ConfigOption {
                key: "CONFIG_VIRTIO_PCI".to_string(),
                value: ConfigValue::Yes,
                comment: Some("VirtIO PCI".to_string()),
            },
            ConfigOption {
                key: "CONFIG_VIRTIO_NET".to_string(),
                value: ConfigValue::Yes,
                comment: Some("VirtIO network".to_string()),
            },
            ConfigOption {
                key: "CONFIG_VIRTIO_BALLOON".to_string(),
                value: ConfigValue::Module,
                comment: Some("VirtIO balloon driver".to_string()),
            },
        ],
    }
}

/// Generate laptop-specific optimizations
fn generate_laptop_config() -> KernelConfigFragment {
    KernelConfigFragment {
        name: "laptop-optimizations".to_string(),
        description: "Laptop power management and features".to_string(),
        config_options: vec![
            ConfigOption {
                key: "CONFIG_ACPI_BATTERY".to_string(),
                value: ConfigValue::Module,
                comment: Some("ACPI battery support".to_string()),
            },
            ConfigOption {
                key: "CONFIG_ACPI_AC".to_string(),
                value: ConfigValue::Module,
                comment: Some("ACPI AC adapter".to_string()),
            },
            ConfigOption {
                key: "CONFIG_CPU_FREQ".to_string(),
                value: ConfigValue::Yes,
                comment: Some("CPU frequency scaling".to_string()),
            },
            ConfigOption {
                key: "CONFIG_CPU_FREQ_GOV_POWERSAVE".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Powersave governor".to_string()),
            },
            ConfigOption {
                key: "CONFIG_CPU_FREQ_GOV_ONDEMAND".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Ondemand governor".to_string()),
            },
            ConfigOption {
                key: "CONFIG_SUSPEND".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Suspend to RAM".to_string()),
            },
            ConfigOption {
                key: "CONFIG_HIBERNATION".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Hibernation support".to_string()),
            },
        ],
    }
}

/// Generate Bluetooth configuration
fn generate_bluetooth_config() -> KernelConfigFragment {
    KernelConfigFragment {
        name: "bluetooth".to_string(),
        description: "Bluetooth support".to_string(),
        config_options: vec![
            ConfigOption {
                key: "CONFIG_BT".to_string(),
                value: ConfigValue::Module,
                comment: Some("Bluetooth subsystem".to_string()),
            },
            ConfigOption {
                key: "CONFIG_BT_RFCOMM".to_string(),
                value: ConfigValue::Module,
                comment: Some("RFCOMM protocol".to_string()),
            },
            ConfigOption {
                key: "CONFIG_BT_HIDP".to_string(),
                value: ConfigValue::Module,
                comment: Some("HID protocol".to_string()),
            },
            ConfigOption {
                key: "CONFIG_BT_HCIBTUSB".to_string(),
                value: ConfigValue::Module,
                comment: Some("USB Bluetooth driver".to_string()),
            },
        ],
    }
}

/// Generate touchscreen configuration
fn generate_touchscreen_config() -> KernelConfigFragment {
    KernelConfigFragment {
        name: "touchscreen".to_string(),
        description: "Touchscreen input support".to_string(),
        config_options: vec![
            ConfigOption {
                key: "CONFIG_INPUT_TOUCHSCREEN".to_string(),
                value: ConfigValue::Yes,
                comment: Some("Touchscreen support".to_string()),
            },
            ConfigOption {
                key: "CONFIG_TOUCHSCREEN_USB_COMPOSITE".to_string(),
                value: ConfigValue::Module,
                comment: Some("USB touchscreen".to_string()),
            },
        ],
    }
}

/// Generate CPU-specific optimizations
fn generate_cpu_config(vendor: &str, flags: &[String]) -> KernelConfigFragment {
    let mut options = Vec::new();

    // CPU vendor optimizations
    if vendor.contains("Intel") {
        options.push(ConfigOption {
            key: "CONFIG_X86_INTEL_PSTATE".to_string(),
            value: ConfigValue::Yes,
            comment: Some("Intel P-State driver".to_string()),
        });
    } else if vendor.contains("AMD") {
        options.push(ConfigOption {
            key: "CONFIG_X86_AMD_PSTATE".to_string(),
            value: ConfigValue::Yes,
            comment: Some("AMD P-State driver".to_string()),
        });
    }

    // Check for specific CPU features
    if flags.contains(&"aes".to_string()) {
        options.push(ConfigOption {
            key: "CONFIG_CRYPTO_AES_NI_INTEL".to_string(),
            value: ConfigValue::Module,
            comment: Some("AES-NI acceleration".to_string()),
        });
    }

    if flags.contains(&"avx2".to_string()) || flags.contains(&"avx".to_string()) {
        options.push(ConfigOption {
            key: "CONFIG_CRYPTO_AVX2".to_string(),
            value: ConfigValue::Yes,
            comment: Some("AVX2 optimizations".to_string()),
        });
    }

    KernelConfigFragment {
        name: "cpu-optimizations".to_string(),
        description: format!("CPU-specific optimizations for {}", vendor),
        config_options: options,
    }
}

/// Convert config fragments to kernel .config format
pub fn fragments_to_config_file(fragments: &[KernelConfigFragment]) -> String {
    let mut config = String::new();

    config.push_str("#\n");
    config.push_str("# Hardware-specific kernel configuration\n");
    config.push_str("# Auto-generated by BuckOS installer\n");
    config.push_str("#\n\n");

    for fragment in fragments {
        config.push_str(&format!("#\n# {}\n#\n", fragment.description));

        for option in &fragment.config_options {
            if let Some(comment) = &option.comment {
                config.push_str(&format!("# {}\n", comment));
            }

            match &option.value {
                ConfigValue::No => {
                    config.push_str(&format!("# {} is not set\n", option.key));
                }
                _ => {
                    config.push_str(&format!("{}={}\n", option.key, option.value.to_config_string()));
                }
            }
        }
        config.push('\n');
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_formatting() {
        assert_eq!(ConfigValue::Yes.to_config_string(), "y");
        assert_eq!(ConfigValue::Module.to_config_string(), "m");
        assert_eq!(ConfigValue::No.to_config_string(), "n");
        assert_eq!(ConfigValue::String("test".to_string()).to_config_string(), "\"test\"");
        assert_eq!(ConfigValue::Number(42).to_config_string(), "42");
    }
}
