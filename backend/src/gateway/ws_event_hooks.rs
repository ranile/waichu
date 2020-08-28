use sqlx::pool::PoolConnection;
use sqlx::PgConnection;
use async_trait::async_trait;
use crate::gateway::ws_send::WsSend;

#[async_trait]
pub trait WsEventHooks: WsSend {
    async fn on_create(&self, conn: &'_ mut PoolConnection<PgConnection>);

    async fn on_update(&self);

    async fn on_delete(&self);
}
