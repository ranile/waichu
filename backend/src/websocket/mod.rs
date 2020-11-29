mod handler;
mod models;

use crate::utils::with_db;
use crate::websocket::models::WsSession;
use common::websocket::MessagePayload;
use futures::future::BoxFuture;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;
use warp::Filter;

type Heartbeats = Arc<RwLock<HashMap<Uuid, Instant>>>;
type Users = Arc<RwLock<HashMap<Uuid, WsSession>>>;

lazy_static! {
    static ref HEARTBEATS: Heartbeats = Heartbeats::default();
    static ref USERS: Users = Users::default();
}

pub fn route(
    pool: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("api" / "ws")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(with_db(pool))
        .map(|ws: warp::ws::Ws, db: PgPool| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| async {
                // probably a bad idea to ignore this
                // but whatever
                let _ = handler::user_connected(db, socket).await;
            })
        })
}

pub(crate) fn send_message<'a, T>(
    message: Arc<MessagePayload<T>>,
    send_to_predicate: impl Fn(Uuid) -> bool + Send + Sync + 'a,
) -> BoxFuture<'a, ()>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + std::fmt::Debug + 'a,
{
    Box::pin(async move {
        println!("send 1");
        let users = USERS.read().await;

        for (uuid, session) in users.iter() {
            println!("send 2 uuid: {}", uuid);
            if send_to_predicate(*uuid) {
                println!("send 2.5 uuid: {}, message: {:?}", uuid, &*message);
                if session.send(&message).is_err() {
                    // add notification
                };
            } else {
                // add notification
            }
        }
    })
}
