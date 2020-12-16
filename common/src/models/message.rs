use crate::{Room, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub uuid: Uuid,
    pub author: User,
    pub room: Room,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(author: User, room: Room, content: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            author,
            content,
            room,
            created_at: Utc::now(),
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
