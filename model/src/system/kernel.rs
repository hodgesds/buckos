use super::License;
use super::Maintainers;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct KernelModule {
    name: String,
    intree: bool,
    retpoline: bool,
    filename: String,
    version_magic: String,
    license: License,
    description: String,
    url: Option<Url>,
    assignee: Option<Maintainers>,
    firmware: Vec<String>,
    alias: Vec<String>,
    depends: Vec<String>,
    // params: map<>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Kernel {
    name: String,
    release: String,
    version: String,
    node_name: String,
    machine: String,
    processor: String,
    platform: String,
    hardware_platform: String,
    operating_system: String,
    // todos: Vec<Rc<TODO>>,
    // bugs: Vec<Bug>,
    modules: Vec<KernelModule>,
}
