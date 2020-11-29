use common::User;
use serde::Serialize;
use sqlx::types::Uuid;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;
use warp::ws::Message;

#[derive(Clone)]
pub struct WsSession {
    pub id: Uuid,
    pub hb: Instant,
    pub pool: PgPool,
    pub tx: Arc<UnboundedSender<Result<Message, warp::Error>>>,
    pub user: Option<Uuid>,
}

impl WsSession {
    pub fn new(pool: PgPool, tx: UnboundedSender<Result<Message, warp::Error>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            hb: Instant::now(),
            pool,
            tx: Arc::new(tx),
            user: None,
        }
    }

    pub fn set_user(&mut self, user: &User) {
        self.user = Some(user.uuid)
    }

    pub fn send<T>(&self, payload: &T) -> anyhow::Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let text = serde_json::to_string(payload)?;
        self.tx.send(Ok(Message::text(text)))?;
        Ok(())
    }
}
