use super::Process;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    url: Option<Url>,
    processes: Option<Vec<Process>>,
}
