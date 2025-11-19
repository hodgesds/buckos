//! System information collectors for diagnostics.

pub mod hardware;
pub mod software;

pub use hardware::HardwareInfo;
pub use software::SoftwareInfo;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::privacy::{PrivacySettings, Redactor};

/// Complete system diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDiagnostics {
    /// Hardware information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware: Option<HardwareInfo>,
    /// Software information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<SoftwareInfo>,
}

impl SystemDiagnostics {
    /// Collect system diagnostics based on privacy settings.
    pub fn collect(settings: &PrivacySettings) -> Result<Self> {
        let redactor = Redactor::new(settings.clone());

        let hardware = if redactor.should_collect("hardware") {
            Some(HardwareInfo::collect(&redactor)?)
        } else {
            None
        };

        let software = if redactor.should_collect("software") {
            Some(SoftwareInfo::collect(&redactor)?)
        } else {
            None
        };

        Ok(Self { hardware, software })
    }
}
