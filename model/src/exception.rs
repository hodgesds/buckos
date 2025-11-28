use super::Process;
use super::StackFrame;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ExceptionCode {
    ApplicationError,
    ArithmeticError,
    AssertionError,
    ConfigurationError,
    EofError,
    InputError,
    KeyboardInterrupt,
    NotImplementedError,
    SyntaxError,
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Exception {
    pub id: Option<u32>,
    pub uuid: Option<Uuid>,
    pub code: ExceptionCode,
    pub message: String,
    pub process: Option<Process>,
    pub stack_frames: Vec<StackFrame>,
    pub url: Option<Url>,
}

impl Exception {
    /// Create a new exception with just a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Default::default()
        }
    }

    /// Create an exception with a specific code and message
    pub fn with_code(code: ExceptionCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            ..Default::default()
        }
    }
}
