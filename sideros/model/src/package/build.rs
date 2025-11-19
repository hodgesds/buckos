use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub enum BuildApiVersion {
    Zero,
}
