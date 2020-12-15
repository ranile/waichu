use futures::executor::block_on;
use lazy_static::lazy_static;
use sqlx::{PgPool, Connection};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::sync::RwLock;
use backend::utils::Transaction;
use futures::future::BoxFuture;

pub async fn setup_test_database() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&env::var("TEST_DATABASE_URL")?)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}


lazy_static! {
    static ref POOL: RwLock<Option<PgPool>> = RwLock::new(None);
}

pub async fn db<F>(callback: F)
    where
            F: FnOnce(PgPool) -> BoxFuture<'static, ()> + 'static + Send + Sync,
{
    if POOL.read().await.is_none() {
        let mut pool = POOL.write().await;
        *pool = Some(setup_test_database().await.unwrap());
    }
    let pool = POOL.read().await.as_ref().expect("no db -- shouldn't happen").clone();
    callback(pool).await;
    // TODO reset_database;

    /*let mut transaction = POOL.read().await.as_ref().expect("no db -- shouldn't happen").begin().await.unwrap();

    callback(&mut transaction).await;
    if let Err(_) = transaction.rollback().await {
    };*/
}
