use super::Person;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TODO {
    text: String,
    assignee: Person,
    repo: String,
    filename: String,
    line: u32,
}
