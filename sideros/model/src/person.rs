// use chrono_tz::Tz;
use super::Address;
use super::User;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub enum PersonStatus {
    Status(String),
    Inactive,
    Busy,
    Banned,
    Deceased,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    status: PersonStatus,
    alias: String,
    first: String,
    middle: Option<String>,
    last: String,
    title: String,
    location: String,
    emails: Vec<String>,
    phone_numbers: Vec<String>,
    websites: Vec<Url>,
    bio: String,
    organization_ids: Vec<u32>,
    // https://crates.io/crates/chrono-tz
    timezone: String,
    address: Option<Address>,
    users: Option<Vec<User>>,
}
