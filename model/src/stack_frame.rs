use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StackFrame {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub frame: Vec<String>,
    pub registers: Vec<String>,
}
