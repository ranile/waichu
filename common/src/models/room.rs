use crate::Asset;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub uuid: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub icon: Option<Asset>,
}

impl Room {
    pub fn new(name: &str) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            created_at: Utc::now(),
            icon: None,
        }
    }
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
