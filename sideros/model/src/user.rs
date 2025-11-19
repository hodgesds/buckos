use super::Group;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: u32,
    name: String,
    groups: Vec<Group>,
}
