use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    #[serde(skip)]
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            username,
            password,
            created_at: Utc::now(),
        }
    }

    pub fn dummy() -> Self {
        Self {
            uuid: Default::default(),
            username: "username".to_string(),
            password: "".to_string(),
            created_at: Utc::now(),
        }
    }
}
