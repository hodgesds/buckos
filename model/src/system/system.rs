use super::Kernel;
use super::Network;
use super::Processor;
use serde::Deserialize;
use std::time::Duration;
use url::Url;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct System {
    uri: Option<Url>,
    hostname: String,
    networks: Vec<Network>,
    processor: Processor,
    kernel: Kernel,
    uptime: Duration,
    // packages: BTreeMap<String, Package>,
}
