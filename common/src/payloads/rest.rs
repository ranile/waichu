use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct JwtToken {
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateRoom {
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateMessagePayload {
    pub content: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JoinMembers {
    pub member: Uuid,
    pub with_elevated_permissions: bool,
}
