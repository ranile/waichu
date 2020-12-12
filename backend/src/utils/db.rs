use crate::bail_if_err;
use futures::future::BoxFuture;
use sqlx::{PgPool, Postgres};
use warp::{Rejection, Reply};

pub type Transaction<'c> = sqlx::Transaction<'c, Postgres>;

fn transaction<F, R, E>(pool: PgPool, callback: F) -> BoxFuture<'static, Result<R, E>>
where
    for<'c> F: FnOnce(&'c mut Transaction) -> BoxFuture<'c, Result<R, E>> + 'static + Send + Sync,
    R: Send,
    E: From<sqlx::Error> + Send,
{
    Box::pin(async move {
        let mut transaction = pool.begin().await?;
        let ret = callback(&mut transaction).await;

        match ret {
            Ok(ret) => {
                transaction.commit().await?;

                Ok(ret)
            }
            Err(err) => {
                transaction.rollback().await?;

                Err(err)
            }
        }
    })
}

pub async fn with_transaction<F, R>(pool: PgPool, callback: F) -> Result<impl Reply, Rejection>
where
    for<'c> F:
        FnOnce(&'c mut Transaction) -> BoxFuture<'c, anyhow::Result<R>> + 'static + Send + Sync,
    R: Reply,
{
    let ret: anyhow::Result<R> =
        transaction(pool, |db| Box::pin(async move { Ok(callback(db).await?) })).await;

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
