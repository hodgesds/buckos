use super::Group;
use super::User;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum BasePermission {
    Read,
    Write,
    Execute,
    Delete,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupPermissions {
    group: Group,
    permission: BasePermission,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPermissions {
    user: User,
    permission: BasePermission,
}
