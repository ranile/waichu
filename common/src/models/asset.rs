use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub uuid: Uuid,
    #[allow(clippy::rc_buffer)] // this is an arc so i don't clean the actual bytes a billion times
    #[serde(skip)]
    pub bytes: Arc<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

impl Asset {
    pub fn new(bytes: Arc<Vec<u8>>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            bytes,
            created_at: Utc::now(),
        }
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
