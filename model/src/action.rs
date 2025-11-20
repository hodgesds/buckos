use serde::{Deserialize, Serialize};
// use std::cell::RefCell;
// use std::rc::{Rc, Weak};
use super::Exception;
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionState {
    Pending,
    Complete,
    Failed,
    Unknown,
}

// TODO: hodgesd - make this a DAG
#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    id: Option<u32>,
    uuid: Option<Uuid>,
    url: Option<Url>,
    message: String,
    state: ActionState,
    exception: Option<Exception>,
}

pub type Actions = Vec<Action>;
