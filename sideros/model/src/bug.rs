use super::Maintainers;
use super::Person;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub enum BugStatus {
    Unconfirmed,
    Confirmed,
    InProgress,
    Fixed,
    Invalid,
    Duplicate,
    WontFix,
    InfoRequired,
    Upstream,
    Unsupported,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BugPriority {
    Low,
    High,
    Critical,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bug {
    // https://bugs.gentoo.org/page.cgi?id=fields.html#alias
    id: String,
    alias: String,
    url: Option<Url>,
    status: BugStatus,
    assignee: Maintainers,
    subscribed: Vec<Person>,
    tags: Vec<String>,
    priority: BugPriority,
}
