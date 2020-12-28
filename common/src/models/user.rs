use crate::Asset;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    #[serde(skip)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub avatar: Option<Asset>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            username,
            password,
            created_at: Utc::now(),
            avatar: None,
        }
    }

    pub fn dummy() -> Self {
        Self {
            uuid: Default::default(),
            username: "username".to_string(),
            password: "".to_string(),
            created_at: Utc::now(),
            avatar: None,
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
