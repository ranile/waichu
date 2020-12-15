pub mod auth;
pub mod macros;
pub mod routes;
pub mod services;
pub mod utils;
pub mod websocket;

pub use macros::*;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

pub fn setup_logger() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

pub async fn setup_database() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
