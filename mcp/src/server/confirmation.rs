//! Confirmation token system for two-phase operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Confirmation token for two-phase operations
///
/// Used to confirm mutating operations like package installation.
/// Tokens expire after a configurable TTL (default: 5 minutes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationToken {
    /// Unique token identifier
    pub token: String,

    /// The pending operation
    pub operation: PendingOperation,

    /// When the token was created
    pub created_at: DateTime<Utc>,

    /// When the token expires
    pub expires_at: DateTime<Utc>,
}

/// Pending operation waiting for confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PendingOperation {
    /// Package installation
    Install {
        /// Packages to install
        packages: Vec<String>,
    },
}

impl PendingOperation {
    /// Get a human-readable description of the operation
    pub fn description(&self) -> String {
        match self {
            PendingOperation::Install { packages, .. } => {
                format!("Install {} package(s)", packages.len())
            }
        }
    }
}
