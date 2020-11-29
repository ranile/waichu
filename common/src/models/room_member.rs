use crate::models::{Room, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomMember {
    pub user: User,
    pub room: Room,
    pub has_elevated_permissions: bool,
    pub joined_at: DateTime<Utc>,
}
