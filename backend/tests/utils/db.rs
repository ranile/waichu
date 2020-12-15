use futures::future::BoxFuture;
use lazy_static::lazy_static;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use tokio::sync::RwLock;

pub async fn setup_test_database() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
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
    let pool = POOL.read().await;
    let pool = pool.as_ref().expect("no db -- shouldn't happen");
    callback(pool.clone()).await;

    let mut tx = pool.begin().await.expect("can't acquire pool");
    sqlx::query("TRUNCATE users, rooms, room_members, messages CASCADE;")
        .execute(&mut tx)
        .await
        .expect("can't delete data");
    tx.commit().await.expect("can't commit delete data")
}
