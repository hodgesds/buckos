// use chrono::TimeZone;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Organization {
    url: Option<Url>,
    name: String,
    primary_timezone: DateTime<Utc>,
}
