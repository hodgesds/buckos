//! Hardware information collectors.

use serde::{Deserialize, Serialize};
use sysinfo::{Components, Disks, Networks, System};

use crate::error::Result;
use crate::privacy::Redactor;

/// Collected hardware information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// CPU information.
    pub cpu: CpuInfo,
    /// Memory information.
    pub memory: MemoryInfo,
    /// Disk information.
    pub disks: Vec<DiskInfo>,
    /// Network interface information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Vec<NetworkInfo>>,
    /// Temperature sensors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensors: Option<Vec<SensorInfo>>,
}

/// CPU information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU brand/model name.
    pub brand: String,
    /// Number of physical cores.
    pub physical_cores: usize,
    /// Number of logical cores (threads).
    pub logical_cores: usize,
    /// CPU architecture.
    pub arch: String,
    /// Current CPU usage percentage per core.
    pub usage_per_core: Vec<f32>,
    /// Global CPU usage percentage.
    pub global_usage: f32,
}

/// Memory information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total RAM in bytes.
    pub total_ram: u64,
    /// Used RAM in bytes.
    pub used_ram: u64,
    /// Available RAM in bytes.
    pub available_ram: u64,
    /// Total swap in bytes.
    pub total_swap: u64,
    /// Used swap in bytes.
    pub used_swap: u64,
}

/// Disk information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Disk name.
    pub name: String,
    /// Mount point.
    pub mount_point: String,
    /// File system type.
    pub file_system: String,
    /// Total space in bytes.
    pub total_space: u64,
    /// Available space in bytes.
    pub available_space: u64,
    /// Whether the disk is removable.
    pub is_removable: bool,
}

/// Network interface information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Interface name.
    pub name: String,
    /// Bytes received.
    pub received: u64,
    /// Bytes transmitted.
    pub transmitted: u64,
    /// Packets received.
    pub packets_received: u64,
    /// Packets transmitted.
    pub packets_transmitted: u64,
    /// MAC address (may be redacted).
    pub mac_address: String,
}

/// Temperature sensor information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorInfo {
    /// Sensor label.
    pub label: String,
    /// Current temperature in Celsius.
    pub temperature: f32,
    /// Maximum temperature in Celsius.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,
    /// Critical temperature in Celsius.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical: Option<f32>,
}

impl HardwareInfo {
    /// Collect hardware information.
    pub fn collect(redactor: &Redactor) -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Collect CPU info
        let cpu = Self::collect_cpu(&sys);

        // Collect memory info
        let memory = Self::collect_memory(&sys);

        // Collect disk info
        let disks = Self::collect_disks(redactor);

        // Collect network info if allowed
        let network = if redactor.should_collect("network") {
            Some(Self::collect_network(redactor))
        } else {
            None
        };

        // Collect sensor info
        let sensors = Self::collect_sensors();

        Ok(Self {
            cpu,
            memory,
            disks,
            network,
            sensors,
        })
    }

    fn collect_cpu(sys: &System) -> CpuInfo {
        let cpus = sys.cpus();
        let brand = cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();
        let usage_per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let global_usage = sys.global_cpu_info().cpu_usage();

        CpuInfo {
            brand,
            physical_cores: sys.physical_core_count().unwrap_or(0),
            logical_cores: cpus.len(),
            arch: std::env::consts::ARCH.to_string(),
            usage_per_core,
            global_usage,
        }
    }

    fn collect_memory(sys: &System) -> MemoryInfo {
        MemoryInfo {
            total_ram: sys.total_memory(),
            used_ram: sys.used_memory(),
            available_ram: sys.available_memory(),
            total_swap: sys.total_swap(),
            used_swap: sys.used_swap(),
        }
    }

    fn collect_disks(redactor: &Redactor) -> Vec<DiskInfo> {
        let disks = Disks::new_with_refreshed_list();
        disks
            .iter()
            .map(|disk| DiskInfo {
                name: redactor.redact(&disk.name().to_string_lossy()),
                mount_point: redactor.redact(&disk.mount_point().to_string_lossy()),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                is_removable: disk.is_removable(),
            })
            .collect()
    }

    fn collect_network(redactor: &Redactor) -> Vec<NetworkInfo> {
        let networks = Networks::new_with_refreshed_list();
        networks
            .iter()
            .map(|(name, data)| NetworkInfo {
                name: name.clone(),
                received: data.total_received(),
                transmitted: data.total_transmitted(),
                packets_received: data.total_packets_received(),
                packets_transmitted: data.total_packets_transmitted(),
                mac_address: redactor.redact(&data.mac_address().to_string()),
            })
            .collect()
    }

    fn collect_sensors() -> Option<Vec<SensorInfo>> {
        let components = Components::new_with_refreshed_list();
        let sensors: Vec<SensorInfo> = components
            .iter()
            .map(|comp| SensorInfo {
                label: comp.label().to_string(),
                temperature: comp.temperature(),
                max: Some(comp.max()),
                critical: comp.critical(),
            })
            .collect();

        if sensors.is_empty() {
            None
        } else {
            Some(sensors)
        }
    }
}

impl MemoryInfo {
    /// Get RAM usage as a percentage.
    pub fn ram_usage_percent(&self) -> f64 {
        if self.total_ram == 0 {
            0.0
        } else {
            (self.used_ram as f64 / self.total_ram as f64) * 100.0
        }
    }

    /// Get swap usage as a percentage.
    pub fn swap_usage_percent(&self) -> f64 {
        if self.total_swap == 0 {
            0.0
        } else {
            (self.used_swap as f64 / self.total_swap as f64) * 100.0
        }
    }
}

/// Format bytes into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
