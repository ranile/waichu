use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use crate::models::User;

#[derive(Deserialize, Clone, Debug)]
pub struct Message {
    pub uuid: Uuid,
    pub content: String,
    pub author: User,
    pub room: Uuid,
    pub created_at: DateTime<Utc>,
}
