use serde::Deserialize;

/// CPU frequency boost state
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum FrequencyBoost {
    /// Boost is enabled
    Enabled,
    /// Boost is disabled
    Disabled,
    /// Boost is not supported
    NotSupported,
}

impl Default for FrequencyBoost {
    fn default() -> Self {
        FrequencyBoost::NotSupported
    }
}

impl std::str::FromStr for FrequencyBoost {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "enabled" | "on" | "1" | "yes" | "true" => Ok(FrequencyBoost::Enabled),
            "disabled" | "off" | "0" | "no" | "false" => Ok(FrequencyBoost::Disabled),
            _ => Ok(FrequencyBoost::NotSupported),
        }
    }
}

/// CPU frequency scaling information
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FrequencyScaling {
    /// Current frequency in MHz
    pub current_mhz: Option<u32>,
    /// Minimum frequency in MHz
    pub min_mhz: Option<u32>,
    /// Maximum frequency in MHz
    pub max_mhz: Option<u32>,
    /// Current scaling governor (e.g., "performance", "powersave", "schedutil")
    pub governor: Option<String>,
    /// Available governors
    pub available_governors: Vec<String>,
    /// Scaling driver (e.g., "intel_pstate", "acpi-cpufreq")
    pub driver: Option<String>,
}

impl FrequencyScaling {
    /// Create frequency scaling info from lscpu output string
    pub fn from_lscpu_string(s: &str) -> Self {
        // Parse string like "800.000" or "800-4500"
        let parts: Vec<&str> = s.split('-').collect();
        let current = parts
            .first()
            .and_then(|p| p.trim().parse::<f64>().ok())
            .map(|f| f as u32);

        Self {
            current_mhz: current,
            min_mhz: None,
            max_mhz: None,
            governor: None,
            available_governors: Vec::new(),
            driver: None,
        }
    }

    /// Read frequency scaling info from sysfs
    pub fn from_sysfs(cpu_id: u32) -> Self {
        let base_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq", cpu_id);

        let read_file = |name: &str| -> Option<String> {
            std::fs::read_to_string(format!("{}/{}", base_path, name))
                .ok()
                .map(|s| s.trim().to_string())
        };

        let parse_khz_to_mhz =
            |s: &str| -> Option<u32> { s.parse::<u64>().ok().map(|khz| (khz / 1000) as u32) };

        Self {
            current_mhz: read_file("scaling_cur_freq").and_then(|s| parse_khz_to_mhz(&s)),
            min_mhz: read_file("scaling_min_freq").and_then(|s| parse_khz_to_mhz(&s)),
            max_mhz: read_file("scaling_max_freq").and_then(|s| parse_khz_to_mhz(&s)),
            governor: read_file("scaling_governor"),
            available_governors: read_file("scaling_available_governors")
                .map(|s| s.split_whitespace().map(String::from).collect())
                .unwrap_or_default(),
            driver: read_file("scaling_driver"),
        }
    }
}

/// NUMA node CPU mapping
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NumaNode {
    /// Node ID
    pub id: u8,
    /// CPUs in this node (e.g., "0-7" or "0,2,4,6")
    pub cpus: String,
    /// Memory in bytes (if available)
    pub memory_bytes: Option<u64>,
}

impl NumaNode {
    /// Parse CPU list into individual CPU IDs
    pub fn cpu_list(&self) -> Vec<u32> {
        let mut cpus = Vec::new();

        for part in self.cpus.split(',') {
            let part = part.trim();
            if let Some((start, end)) = part.split_once('-') {
                if let (Ok(s), Ok(e)) = (start.parse::<u32>(), end.parse::<u32>()) {
                    cpus.extend(s..=e);
                }
            } else if let Ok(cpu) = part.parse::<u32>() {
                cpus.push(cpu);
            }
        }

        cpus
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Processor {
    /// CPU architecture (e.g., "x86_64", "aarch64")
    architecture: String,
    /// Total number of CPUs
    cpus: u16,
    /// Online CPUs string (e.g., "0-15")
    online_cpus: String,
    /// CPU vendor (e.g., "GenuineIntel", "AuthenticAMD")
    vendor_id: String,
    /// Model name (e.g., "AMD Ryzen 9 5900X 12-Core Processor")
    model_name: String,
    /// CPU family number
    cpu_family: String,
    /// Model number
    model: String,
    /// Threads per core
    threads_per_core: u8,
    /// Cores per socket
    cores_per_socket: u16,
    /// Number of sockets
    sockets: u8,
    /// CPU stepping
    stepping: String,
    /// Frequency boost state
    #[serde(default)]
    freq_boost: FrequencyBoost,
    /// Frequency scaling information
    #[serde(default)]
    freq_scaling: FrequencyScaling,
    /// Maximum frequency in MHz
    cpu_max_mhz: u32,
    /// Minimum frequency in MHz
    cpu_min_mhz: u32,
    /// BogoMIPS value
    bogo_mips: f64,
    /// CPU flags (e.g., "sse4_2 avx2 aes")
    flags: String,
    /// Virtualization support (e.g., "AMD-V", "VT-x")
    virtualization: String,
    /// L1 data cache size
    l1d_cache: String,
    /// L1 instruction cache size
    l1i_cache: String,
    /// L2 cache size
    l2_cache: String,
    /// L3 cache size
    l3_cache: String,
    /// Number of NUMA nodes
    numa_nodes: u8,
    /// NUMA node to CPU mapping
    #[serde(default)]
    numa_node_cpus: Vec<NumaNode>,
}

impl Processor {
    /// Get the frequency boost state
    pub fn freq_boost(&self) -> &FrequencyBoost {
        &self.freq_boost
    }

    /// Get frequency scaling information
    pub fn freq_scaling(&self) -> &FrequencyScaling {
        &self.freq_scaling
    }

    /// Get NUMA node CPU mappings
    pub fn numa_node_cpus(&self) -> &[NumaNode] {
        &self.numa_node_cpus
    }

    /// Check if CPU supports a specific flag
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags
            .split_whitespace()
            .any(|f| f.eq_ignore_ascii_case(flag))
    }

    /// Get all CPU flags as a vector
    pub fn flag_list(&self) -> Vec<&str> {
        self.flags.split_whitespace().collect()
    }

    /// Get total physical cores (sockets * cores_per_socket)
    pub fn physical_cores(&self) -> u32 {
        (self.sockets as u32) * (self.cores_per_socket as u32)
    }

    /// Get total logical CPUs (physical_cores * threads_per_core)
    pub fn logical_cpus(&self) -> u32 {
        self.physical_cores() * (self.threads_per_core as u32)
    }

    /// Parse online CPUs string into list
    pub fn online_cpu_list(&self) -> Vec<u32> {
        let mut cpus = Vec::new();

        for part in self.online_cpus.split(',') {
            let part = part.trim();
            if let Some((start, end)) = part.split_once('-') {
                if let (Ok(s), Ok(e)) = (start.parse::<u32>(), end.parse::<u32>()) {
                    cpus.extend(s..=e);
                }
            } else if let Ok(cpu) = part.parse::<u32>() {
                cpus.push(cpu);
            }
        }

        cpus
    }

    /// Check if virtualization is supported
    pub fn has_virtualization(&self) -> bool {
        !self.virtualization.is_empty() && self.virtualization.to_lowercase() != "none"
    }
}

/*

[[lscpu]]
field = "Vulnerability Itlb multihit:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability L1tf:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Mds:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Meltdown:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Mmio stale data:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Retbleed:"
data = "Mitigation; untrained return thunk; SMT enabled with STIBP protection"

[[lscpu]]
field = "Vulnerability Spec store bypass:"
data = "Mitigation; Speculative Store Bypass disabled via prctl"

[[lscpu]]
field = "Vulnerability Spectre v1:"
data = "Mitigation; usercopy/swapgs barriers and __user pointer sanitization"

[[lscpu]]
field = "Vulnerability Spectre v2:"
data = "Mitigation; Retpolines, IBPB conditional, STIBP always-on, RSB filling, PBRSB-eIBRS Not affected"

[[lscpu]]
field = "Vulnerability Srbds:"
data = "Not affected"

[[lscpu]]
field = "Vulnerability Tsx async abort:"
data = "Not affected"

*/
