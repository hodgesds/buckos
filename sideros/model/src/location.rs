use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct County {
    id: u32,
    state_id: u32,
    name: String,
    url: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    name: String,
    code: String,
    full_name: String,
    url: Option<Url>,
    counties: Option<Vec<County>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Country {
    name: String,
    code: String,
    full_name: String,
    url: Option<Url>,
    states: Option<Vec<State>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct City {
    id: u32,
    state_id: Option<u32>,
    name: String,
    full_name: String,
    url: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Suburb {
    id: u32,
    city_id: u32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Street {
    id: u32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StreetType {
    Street,
    Avenue,
    Lane,
    Drive,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    city: City,
    suburb: Option<Suburb>,
    street: Street,
    street_type: Option<StreetType>,
    address: String,
    address2: String,
    county: Option<County>,
    state: Option<State>,
    country: Option<Country>,
}
