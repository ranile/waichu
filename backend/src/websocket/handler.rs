use super::{HEARTBEATS, USERS};
use crate::websocket::models::WsSession;
use crate::{auth, services};
use anyhow::{anyhow, Context};
use common::websocket::{AuthenticatePayload, AuthenticatedPayload, MessagePayload, OpCode};
use futures::{FutureExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use sqlx::types::Uuid;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::WebSocket;
use warp::ws::Message;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize)]
struct ResponseMessage {}

async fn heartbeat(session_id: Uuid, tx: Arc<mpsc::UnboundedSender<Result<Message, warp::Error>>>) {
    if tx.send(Ok(Message::ping(""))).is_err() {
        return user_disconnected(session_id).await;
    };
    loop {
        let hb = HEARTBEATS.read().await;
        if let Some(hb) = hb.get(&session_id) {
            if Instant::now().duration_since(*hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // disconnect user
                let _ = tx.send(Ok(Message::close()));
                user_disconnected(session_id).await;

                // don't try to send a ping
                return;
            }

            println!("Sending heartbeat ping");
            if tx.send(Ok(Message::ping(""))).is_err() {
                user_disconnected(session_id).await;
            };

            tokio::time::sleep(HEARTBEAT_INTERVAL).await;
        }
    }
}

pub async fn user_connected(pool: PgPool, ws: WebSocket) -> anyhow::Result<()> {
    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    // Create our own websocket session
    let mut session = WsSession::new(pool, tx);

    // listen to messages
    while let Some(result) = user_ws_rx.next().await {
        let msg = result?;
        println!("Websocket message received {:?}", msg);

        if let Err(e) = user_message(&mut session, msg).await {
            // TODO codes
            session
                .tx
                .send(Ok(Message::close_with(0_u16, e.to_string())))?;
            break;
        };
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(session.id).await;
    Ok(())
}

async fn user_message(session: &mut WsSession, message: Message) -> anyhow::Result<()> {
    if message.is_text() {
        let message = message
            .to_str()
            .map_err(|_| anyhow!("failed to parse message as string"))?;

        let json =
            serde_json::from_str::<MessagePayload<Value>>(message).context("invalid json found")?;

        match json.op {
            OpCode::Authenticate => {
                let token = serde_json::from_value::<AuthenticatePayload>(json.data)?.token;

                let mut db = session.pool.begin().await?;

                let user = auth::parse_token(&mut db, &token).await?;
                let user = match user {
                    Some(user) => user,
                    None => {
                        return Err(anyhow!("no user found"));
                    }
                };
                session.set_user(&user);
                // maybe arc this clone?
                USERS.write().await.insert(user.uuid, session.clone());

                tokio::task::spawn(heartbeat(session.id, session.tx.clone()));

                let rooms = services::room::get_with_user(&mut db, &user).await?;
                let payload = MessagePayload {
                    op: OpCode::Authenticated,
                    data: AuthenticatedPayload { me: user, rooms },
                };

                session.send(&payload)?;
            }
            _ => return Err(anyhow!("invalid OP code")),
        }
    } else if message.is_pong() {
        let hb = Instant::now();
        session.hb = hb;
        HEARTBEATS.write().await.insert(session.id, hb);
        println!("~~meat~~ beaten at {:?}", hb);
    };

    Ok(())
}

async fn user_disconnected(uuid: Uuid) {
    eprintln!("good bye user: {}", uuid);

    // Stream closed up, so remove from the user list and heartbeats
    USERS.write().await.remove(&uuid);
    HEARTBEATS.write().await.remove(&uuid);
}
