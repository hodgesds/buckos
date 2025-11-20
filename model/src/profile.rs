use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThirdPartyMirror {
    name: String,
    uris: Vec<Url>,
    last_checked: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlagMeta {
    description: String,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Flags {
    flags: BTreeMap<String, FlagMeta>,
}

impl Flags {
    pub fn get_tags(&self) -> Vec<String> {
        let mut s: Vec<String> = Vec::new();
        for m in self.flags.values().cloned() {
            s.extend(m.tags);
        }
        s
    }
}
