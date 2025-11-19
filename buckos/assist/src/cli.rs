//! CLI command definitions and argument parsing.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Buckos system diagnostic and troubleshooting assistant.
///
/// Collect system information for troubleshooting while maintaining privacy control.
#[derive(Parser, Debug)]
#[command(name = "buckos-assist")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Verbosity level (can be repeated for more verbosity)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Collect system diagnostic information
    Collect(CollectArgs),

    /// Display a quick system summary
    Summary(SummaryArgs),

    /// Configure privacy settings
    Privacy(PrivacyArgs),
}

/// Arguments for the collect command.
#[derive(Parser, Debug)]
pub struct CollectArgs {
    /// Output format
    #[arg(short, long, default_value = "text")]
    pub format: OutputFormatArg,

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Privacy preset to use
    #[arg(short, long, default_value = "default")]
    pub privacy: PrivacyPreset,

    /// Include hardware information
    #[arg(long, default_value = "true")]
    pub hardware: bool,

    /// Include software information
    #[arg(long, default_value = "true")]
    pub software: bool,

    /// Include network information
    #[arg(long, default_value = "true")]
    pub network: bool,

    /// Include process information
    #[arg(long, default_value = "true")]
    pub processes: bool,

    /// Skip redacting usernames
    #[arg(long)]
    pub no_redact_usernames: bool,

    /// Skip redacting IP addresses
    #[arg(long)]
    pub no_redact_ips: bool,

    /// Skip redacting MAC addresses
    #[arg(long)]
    pub no_redact_macs: bool,

    /// Skip redacting home directory paths
    #[arg(long)]
    pub no_redact_home: bool,

    /// Redact hostnames
    #[arg(long)]
    pub redact_hostnames: bool,

    /// Interactive mode - preview and confirm before saving
    #[arg(short, long)]
    pub interactive: bool,
}

/// Arguments for the summary command.
#[derive(Parser, Debug)]
pub struct SummaryArgs {
    /// Include process summary
    #[arg(long)]
    pub processes: bool,
}

/// Arguments for the privacy command.
#[derive(Parser, Debug)]
pub struct PrivacyArgs {
    /// Privacy subcommand
    #[command(subcommand)]
    pub command: PrivacyCommands,
}

/// Privacy-related subcommands.
#[derive(Subcommand, Debug)]
pub enum PrivacyCommands {
    /// Show current privacy settings
    Show,

    /// List available privacy presets
    Presets,

    /// Interactively configure privacy settings
    Configure,
}

/// Output format argument.
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormatArg {
    /// Compact JSON
    Json,
    /// Pretty-printed JSON
    JsonPretty,
    /// TOML format
    Toml,
    /// Human-readable text
    Text,
}

impl From<OutputFormatArg> for crate::report::OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Json => crate::report::OutputFormat::Json,
            OutputFormatArg::JsonPretty => crate::report::OutputFormat::JsonPretty,
            OutputFormatArg::Toml => crate::report::OutputFormat::Toml,
            OutputFormatArg::Text => crate::report::OutputFormat::Text,
        }
    }
}

/// Privacy preset options.
#[derive(Debug, Clone, ValueEnum)]
pub enum PrivacyPreset {
    /// Default settings - balanced privacy with useful diagnostics
    Default,
    /// Minimal collection - only essential hardware info
    Minimal,
    /// Full collection - everything, no redaction (local use only)
    Full,
}
