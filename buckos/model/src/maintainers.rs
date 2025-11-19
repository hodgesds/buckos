// use super::Person;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// see:
// https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners

#[derive(Serialize, Deserialize, Debug)]
pub struct Maintainers {
    reviewers: BTreeMap<String, Vec<String>>,
    approvers: BTreeMap<String, Vec<String>>,
}
