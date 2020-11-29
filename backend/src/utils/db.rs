use crate::bail_if_err;
use futures::future::BoxFuture;
use sqlx::{Connection, PgPool, Postgres};
use warp::{Rejection, Reply};

pub type Transaction<'c> = sqlx::Transaction<'c, Postgres>;

pub async fn with_transaction<F, R>(pool: PgPool, callback: F) -> Result<impl Reply, Rejection>
where
    for<'c> F:
        FnOnce(&'c mut Transaction) -> BoxFuture<'c, anyhow::Result<R>> + 'static + Send + Sync,
    R: Reply,
{
    let mut conn = bail_if_err!(pool.acquire().await.map_err(anyhow::Error::from));

    let ret: anyhow::Result<R> = conn
        .transaction(|db| Box::pin(async move { Ok(callback(db).await?) }))
        .await;

    let ret = bail_if_err!(ret);
    Ok(ret.into_response())
}

/*
macro_rules! with_transaction {
    ($pool:ident, $expr:expr) => {{
        let block = || async {
            let mut db = $pool.acquire().await?;
            let mut db = db.begin().await?;

            let res = ($expr)(&mut db).await;

            db.commit().await?;
            Ok(res)
        };
        let resp = bail_if_err!(block().await);
        Ok(bail_if_err!(resp))
    }};
}
*/
