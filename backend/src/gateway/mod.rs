mod payloads;
mod ws_send;
mod ws_event_hooks;

pub use ws_send::WsSend;
pub use ws_event_hooks::WsEventHooks;

use std::collections::HashMap;
use std::sync::{Arc};

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use warp::ws::{Message, WebSocket};
use warp::Filter;
use uuid::Uuid;
use sqlx::PgPool;
use std::time::{Instant, Duration};
use serde_json::Value;
use crate::auth;
use crate::services::room_service;
use crate::routes::with_db;
use once_cell::sync::Lazy;
use crate::models::User;
use crate::gateway::payloads::InitialPayload;
use serde::Serialize;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct WsSession {
    pub id: Uuid,
    pub hb: Instant,
    pub pool: PgPool,
    pub tx: mpsc::UnboundedSender<Result<Message, warp::Error>>,
    pub user: Option<User>,
}

#[derive(Serialize)]
struct OutgoingMessage<T: Serialize> {
    op: u32,
    data: T,
}

impl<T: Serialize> OutgoingMessage<T> {
    pub fn to_message(&self) -> Message {
        Message::text(serde_json::to_string(self).unwrap())
    }
}

async fn heartbeat(session_id: Uuid, tx: mpsc::UnboundedSender<Result<Message, warp::Error>>) {
    loop {
        let hb = HEARTBEATS.read().await;
        let hb = hb.get(&session_id);
        if let Some(hb) = hb {

            if Instant::now().duration_since(*hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                user_disconnected(&session_id);

                // don't try to send a ping
                return;
            }

            println!("Sending heartbeat ping");
            tx.send(Ok(Message::ping("".as_bytes()))).unwrap_or_else(|_err| {
                user_disconnected(&session_id);
            });

            tokio::time::delay_for(HEARTBEAT_INTERVAL).await;
        }
    }
}

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a user session of `WsSession`
type Users = Arc<RwLock<HashMap<Uuid, WsSession>>>;

static USERS: Lazy<Users> = Lazy::new(|| Users::default());

type Heartbeats = Arc<RwLock<HashMap<Uuid, Instant>>>;

static HEARTBEATS: Lazy<Heartbeats> = Lazy::new(|| Heartbeats::default());


pub fn gateway_route(pool: PgPool) -> impl Filter<Extract=(impl warp::Reply, ), Error=warp::Rejection> + Clone {
    warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        // .and(users)
        .and(with_db(pool))
        .map(|ws: warp::ws::Ws, db: PgPool| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, db))
        })
}

async fn user_connected(ws: WebSocket, pool: PgPool) {

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    let mut ws_session = WsSession {
        id: Uuid::new_v4(),
        hb: Instant::now(),
        pool,
        tx,
        user: None,
    };


    // Every time the user sends a message, broadcast it to
    // all other users...
    ws_session.tx.send(Ok(Message::ping("".as_bytes())));
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", ws_session.id, e);
                break;
            }
        };
        println!("Websocket message received {:?}", msg);
        user_message(&mut ws_session, msg).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(&ws_session.id).await;
}

async fn user_message(session: &mut WsSession, msg: Message) {
    if msg.is_text() {
        let msg = msg.to_str().unwrap();

        let json = serde_json::from_str::<HashMap<String, Value>>(msg);
        println!("json, {:?}", json);

        let json = match json {
            Ok(json) => json,
            Err(e) => {
                close(session, &format!("Bad payload: {}", e.to_string())).await;
                return;
            }
        };

        println!("got till op check");
        match json.get("op") {
            Some(serde_json::Value::Number(num)) => {
                match num.as_u64() {
                    Some(1) => {
                        handle_authenticate_payload(session, &json).await;
                    }
                    _ => close(session, "Unknown OP code").await
                }
            }
            _ => close(session, "Bad payload: invalid OP code").await
        }
    } else if msg.is_ping() {
        //...
    } else if msg.is_pong() {
        session.hb = Instant::now();
        HEARTBEATS.write().await.insert(session.id, Instant::now());
    }
}

async fn user_disconnected(my_id: &Uuid) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    USERS.write().await.remove(my_id);
    HEARTBEATS.write().await.remove(my_id);
}

async fn handle_authenticate_payload(session: &mut WsSession, json: &HashMap<String, Value>) {
    println!("auth handler");
    match json.get("token") {
        Some(Value::String(token)) => {
            let user = auth::parse_token(&session.pool, &token).await;
            println!("got user");
            match user {
                Ok(user) => {
                    session.user = Some(user.clone());
                    USERS.write().await.insert(session.id, session.clone());


                    println!("before rooms fetch");
                    let rooms = room_service::get_with_user(&session.pool, &user.uuid).await;

                    println!("after rooms fetch, got rooms");
                    if let Ok(rooms) = rooms {
                        let msg = OutgoingMessage {
                            op: 2,
                            data: InitialPayload {
                                rooms,
                                me: user,
                            },
                        };
                        session.tx.send(Ok(msg.to_message())).unwrap();
                    }
                    println!("Sending ping 11");
                    tokio::task::spawn(heartbeat(session.id, session.tx.clone()));
                }
                Err(e) => {
                    close(session, &format!("Invalid token provided: {}", e.to_string())).await;
                    return;
                }
            }
        }
        _ => {
            println!("bad token close");
            close(session, "Invalid token provided").await;
        }
    }
}

async fn close(session: &WsSession, _msg: &str) {
    session.tx.send(Ok(Message::close()));
    user_disconnected(&session.id).await
}

