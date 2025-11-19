use super::Maintainers;
use super::Person;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionMeta {
    name: String,
    features: Option<Vec<String>>,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    url: Option<Url>,
    active: bool,
    meta: SubscriptionMeta,
    expiration: Option<DateTime<Utc>>,
}

impl Subscription {
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        self.expiration
            .map_or(false, |exp| bool::from((now - exp).is_zero()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MaintainersSubscription {
    maintainers: Maintainers,
    subscription: Subscription,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonSubscription {
    person: Person,
    subscription: Subscription,
}
