//! Sideros System Diagnostic and Troubleshooting Assistant
//!
//! This crate provides tools for collecting system diagnostic information
//! while maintaining user privacy. It can gather hardware and software
//! information and generate reports in various formats.
//!
//! # Features
//!
//! - **Hardware Diagnostics**: CPU, memory, disk, network, and sensor information
//! - **Software Diagnostics**: OS info, kernel version, running processes, environment
//! - **Privacy Controls**: Configurable redaction of sensitive information
//! - **Multiple Output Formats**: JSON, TOML, or human-readable text
//! - **Interactive Mode**: Preview and confirm data before export
//!
//! # Example
//!
//! ```no_run
//! use sideros_assist::{
//!     collectors::SystemDiagnostics,
//!     privacy::PrivacySettings,
//!     report::{DiagnosticReport, OutputFormat},
//! };
//!
//! // Collect diagnostics with default privacy settings
//! let settings = PrivacySettings::default();
//! let diagnostics = SystemDiagnostics::collect(&settings).unwrap();
//!
//! // Create and export report
//! let report = DiagnosticReport::new(diagnostics, settings);
//! let output = report.export(OutputFormat::Text).unwrap();
//! println!("{}", output);
//! ```

pub mod cli;
pub mod collectors;
pub mod error;
pub mod privacy;
pub mod report;

pub use error::{Error, Result};
pub use privacy::PrivacySettings;
pub use report::{DiagnosticReport, OutputFormat};
