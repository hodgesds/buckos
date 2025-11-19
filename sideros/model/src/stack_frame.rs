use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StackFrame {
    file: Option<String>,
    line: Option<u32>,
    frame: Vec<String>,
    registers: Vec<String>,
}
