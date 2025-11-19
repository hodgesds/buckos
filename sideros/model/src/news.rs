use super::Maintainers;
use super::Organization;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub enum NewsAuthor {
    User,
    Maintainers,
    Person,
    Organization,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct News {
    auth: NewsAuthor,
    url: Option<Url>,
    title: String,
    header: String,
    body: String,
    tags: Vec<String>,
    published: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrganizationNews {
    org: Organization,
    news: Vec<News>,
}

// impl OrganizationNews {
//     pub fn last_published(&self) -> Option<DateTime<Utc>> {
//         let mut s: Vec<String> = Vec::new();
//         let mut last_published = Dat
//         for m in self.news.values().cloned() {
//         }
//     }
//     pub fn last_updated(&self) -> DateTime<Utc> {
//         let mut s: Vec<String> = Vec::new();
//     }
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct MaintainersNews {
    maintainers: Maintainers,
    news: Vec<News>,
}
