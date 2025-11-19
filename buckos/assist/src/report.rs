//! Report generation and export functionality.

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::collectors::SystemDiagnostics;
use crate::error::{Error, Result};
use crate::privacy::PrivacySettings;

/// Output format for diagnostic reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format.
    Json,
    /// Pretty-printed JSON.
    JsonPretty,
    /// TOML format.
    Toml,
    /// Human-readable text format.
    Text,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "json-pretty" | "jsonpretty" => Ok(OutputFormat::JsonPretty),
            "toml" => Ok(OutputFormat::Toml),
            "text" | "txt" => Ok(OutputFormat::Text),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// A complete diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// Report metadata.
    pub metadata: ReportMetadata,
    /// Privacy settings used for collection.
    pub privacy_settings: PrivacySettings,
    /// Collected system diagnostics.
    pub diagnostics: SystemDiagnostics,
}

/// Metadata about the diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Unique report ID.
    pub id: String,
    /// Report version.
    pub version: String,
    /// Timestamp when the report was generated.
    pub generated_at: String,
    /// Timezone of the system.
    pub timezone: String,
    /// Tool version that generated the report.
    pub tool_version: String,
}

impl DiagnosticReport {
    /// Create a new diagnostic report from collected data.
    pub fn new(diagnostics: SystemDiagnostics, privacy_settings: PrivacySettings) -> Self {
        let now: DateTime<Utc> = Utc::now();
        let local: DateTime<Local> = Local::now();

        Self {
            metadata: ReportMetadata {
                id: Uuid::new_v4().to_string(),
                version: "1.0".to_string(),
                generated_at: now.to_rfc3339(),
                timezone: local.offset().to_string(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            privacy_settings,
            diagnostics,
        }
    }

    /// Export the report to a string in the specified format.
    pub fn export(&self, format: OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Json => {
                serde_json::to_string(self).map_err(|e| Error::SerializationError(e.to_string()))
            }
            OutputFormat::JsonPretty => serde_json::to_string_pretty(self)
                .map_err(|e| Error::SerializationError(e.to_string())),
            OutputFormat::Toml => {
                toml::to_string_pretty(self).map_err(|e| Error::SerializationError(e.to_string()))
            }
            OutputFormat::Text => Ok(self.to_text()),
        }
    }

    /// Export the report to a file.
    pub fn export_to_file(&self, path: &Path, format: OutputFormat) -> Result<()> {
        let content = self.export(format)?;
        std::fs::write(path, content).map_err(|e| Error::ReportWriteError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })
    }

    /// Convert the report to human-readable text format.
    fn to_text(&self) -> String {
        use crate::collectors::hardware::format_bytes;
        use crate::collectors::software::format_uptime;

        let mut output = String::new();

        // Header
        output.push_str("=== Buckos System Diagnostic Report ===\n\n");
        output.push_str(&format!("Report ID: {}\n", self.metadata.id));
        output.push_str(&format!("Generated: {}\n", self.metadata.generated_at));
        output.push_str(&format!("Tool Version: {}\n", self.metadata.tool_version));
        output.push('\n');

        // Hardware info
        if let Some(hw) = &self.diagnostics.hardware {
            output.push_str("--- Hardware Information ---\n\n");

            // CPU
            output.push_str("CPU:\n");
            output.push_str(&format!("  Brand: {}\n", hw.cpu.brand));
            output.push_str(&format!("  Architecture: {}\n", hw.cpu.arch));
            output.push_str(&format!("  Physical Cores: {}\n", hw.cpu.physical_cores));
            output.push_str(&format!("  Logical Cores: {}\n", hw.cpu.logical_cores));
            output.push_str(&format!("  Global Usage: {:.1}%\n", hw.cpu.global_usage));
            output.push('\n');

            // Memory
            output.push_str("Memory:\n");
            output.push_str(&format!(
                "  Total RAM: {}\n",
                format_bytes(hw.memory.total_ram)
            ));
            output.push_str(&format!(
                "  Used RAM: {} ({:.1}%)\n",
                format_bytes(hw.memory.used_ram),
                hw.memory.ram_usage_percent()
            ));
            output.push_str(&format!(
                "  Available RAM: {}\n",
                format_bytes(hw.memory.available_ram)
            ));
            if hw.memory.total_swap > 0 {
                output.push_str(&format!(
                    "  Total Swap: {}\n",
                    format_bytes(hw.memory.total_swap)
                ));
                output.push_str(&format!(
                    "  Used Swap: {} ({:.1}%)\n",
                    format_bytes(hw.memory.used_swap),
                    hw.memory.swap_usage_percent()
                ));
            }
            output.push('\n');

            // Disks
            output.push_str("Disks:\n");
            for disk in &hw.disks {
                output.push_str(&format!("  {}:\n", disk.name));
                output.push_str(&format!("    Mount: {}\n", disk.mount_point));
                output.push_str(&format!("    Filesystem: {}\n", disk.file_system));
                output.push_str(&format!(
                    "    Total: {}\n",
                    format_bytes(disk.total_space)
                ));
                output.push_str(&format!(
                    "    Available: {}\n",
                    format_bytes(disk.available_space)
                ));
            }
            output.push('\n');

            // Network
            if let Some(networks) = &hw.network {
                output.push_str("Network Interfaces:\n");
                for net in networks {
                    output.push_str(&format!("  {}:\n", net.name));
                    output.push_str(&format!("    MAC: {}\n", net.mac_address));
                    output.push_str(&format!(
                        "    Received: {}\n",
                        format_bytes(net.received)
                    ));
                    output.push_str(&format!(
                        "    Transmitted: {}\n",
                        format_bytes(net.transmitted)
                    ));
                }
                output.push('\n');
            }

            // Sensors
            if let Some(sensors) = &hw.sensors {
                output.push_str("Temperature Sensors:\n");
                for sensor in sensors {
                    output.push_str(&format!(
                        "  {}: {:.1}°C",
                        sensor.label, sensor.temperature
                    ));
                    if let Some(max) = sensor.max {
                        output.push_str(&format!(" (max: {:.1}°C)", max));
                    }
                    output.push('\n');
                }
                output.push('\n');
            }
        }

        // Software info
        if let Some(sw) = &self.diagnostics.software {
            output.push_str("--- Software Information ---\n\n");

            // OS
            output.push_str("Operating System:\n");
            output.push_str(&format!("  Name: {}\n", sw.os.name));
            output.push_str(&format!("  Version: {}\n", sw.os.version));
            output.push_str(&format!("  Kernel: {}\n", sw.os.kernel_version));
            output.push_str(&format!("  Architecture: {}\n", sw.os.arch));
            output.push_str(&format!("  Hostname: {}\n", sw.os.hostname));
            output.push_str(&format!("  Uptime: {}\n", format_uptime(sw.os.uptime)));
            output.push('\n');

            // Processes
            if let Some(procs) = &sw.processes {
                output.push_str("Processes:\n");
                output.push_str(&format!("  Total: {}\n", procs.total_count));
                output.push_str(&format!("  Running: {}\n", procs.running_count));
                output.push_str(&format!("  Sleeping: {}\n", procs.sleeping_count));
                output.push('\n');

                output.push_str("  Top by CPU:\n");
                for (i, proc) in procs.top_by_cpu.iter().take(5).enumerate() {
                    output.push_str(&format!(
                        "    {}. {} (PID {}): {:.1}%\n",
                        i + 1,
                        proc.name,
                        proc.pid,
                        proc.cpu_usage
                    ));
                }
                output.push('\n');

                output.push_str("  Top by Memory:\n");
                for (i, proc) in procs.top_by_memory.iter().take(5).enumerate() {
                    output.push_str(&format!(
                        "    {}. {} (PID {}): {}\n",
                        i + 1,
                        proc.name,
                        proc.pid,
                        format_bytes(proc.memory)
                    ));
                }
                output.push('\n');
            }

            // Environment
            if let Some(env) = &sw.environment {
                output.push_str("Environment:\n");
                for var in env {
                    output.push_str(&format!("  {}={}\n", var.name, var.value));
                }
                output.push('\n');
            }
        }

        output.push_str("=== End of Report ===\n");
        output
    }
}
