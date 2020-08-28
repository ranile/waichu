use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Room {
    pub uuid: Uuid,
    pub name: String,
    pub owner: Uuid,
    pub created_at: DateTime<Utc>
}
