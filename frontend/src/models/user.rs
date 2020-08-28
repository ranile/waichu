use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>
}
