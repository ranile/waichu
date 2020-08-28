use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sqlx::{FromRow, PgConnection};
use crate::models::User;
use chrono::{DateTime, Utc};
use sqlx::pool::PoolConnection;
use crate::services::room_service;
use crate::utils::serializer::*;
use warp::Reply;
use warp::reply::Response;
use crate::gateway::{WsEventHooks, WsSend};
use async_trait::async_trait;

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct Room {
    pub uuid: Uuid,
    pub name: String,
    #[serde(serialize_with = "serialize_to_uuid")]
    pub owner: User,
    pub created_at: DateTime<Utc>,
    pub member_count: i64,
}

impl Room {
    pub fn new(name: &str, owner: User) -> Room {
        Room { uuid: Uuid::new_v4(), name: name.to_string(), created_at: Utc::now(), owner, member_count: 0 }
    }

    pub async fn members(&self, conn: &mut PoolConnection<PgConnection>) -> sqlx::Result<Vec<RoomMember>> {
        room_service::get_room_members(conn, &self.uuid).await
    }
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct RoomMember {
    pub username: String,
    pub uuid: Uuid,
    pub created_at: DateTime<Utc>,
    pub joined_at: DateTime<Utc>,
}

impl GetUuid for Room {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

impl Reply for Room {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string_pretty(&self).unwrap().into())
    }
}

impl WsSend for Room {}

#[async_trait]
impl WsEventHooks for Room {
    async fn on_create(&self, conn: &mut PoolConnection<PgConnection>) {
        let users = self.members(conn).await;
        self.send(4, users).await;
    }

    async fn on_update(&self) {
        unimplemented!()
    }

    async fn on_delete(&self) {
        unimplemented!()
    }
}
