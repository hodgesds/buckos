use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapping {
    id: u64,
    uuid: Uuid,
}
