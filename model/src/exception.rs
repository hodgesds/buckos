use super::Process;
use super::StackFrame;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
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
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Exception {
    id: Option<u32>,
    uuid: Option<Uuid>,
    code: ExceptionCode,
    message: String,
    process: Option<Process>,
    stack_frames: Vec<StackFrame>,
    url: Option<Url>,
}
