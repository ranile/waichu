use crate::models::{Room, User};
use serde::Serialize;

#[derive(Serialize)]
pub struct InitialPayload {
    pub rooms: Vec<Room>,
    pub me: User
}
