use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sqlx::{FromRow, PgConnection};
use crate::models::{Room, User};
use chrono::{DateTime, Utc};
use crate::gateway::{WsSend, WsEventHooks};
use sqlx::pool::PoolConnection;
use async_trait::async_trait;
use crate::utils::serializer::*;
use warp::reply::Response;
use warp::Reply;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Message {
    pub uuid: Uuid,
    pub content: String,
    #[serde(serialize_with = "serialize_to_uuid")]
    pub author: User,
    #[serde(serialize_with = "serialize_to_uuid")]
    pub room: Room,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(content: &str, author: User, room: Room) -> Self {
        Message {
            uuid: Uuid::new_v4(),
            content: content.to_string(),
            author,
            room,
            created_at: Utc::now(),
        }
    }
}

impl WsSend for Message {}

#[async_trait]
impl WsEventHooks for Message {
    async fn on_create(&self, conn: &'_ mut PoolConnection<PgConnection>) {
        println!("calling message on create");
        let users = self.room.members(conn).await;
        self.send(3, users).await;
    }

    async fn on_update(&self) {
        unimplemented!()
    }

    async fn on_delete(&self) {
        unimplemented!()
    }
}

impl GetUuid for Message {
    fn uuid(&self) -> Uuid { self.uuid }
}

impl Reply for Message {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string_pretty(&self).unwrap().into())
    }
}
