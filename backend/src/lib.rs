extern crate chrono;
extern crate bcrypt;

pub mod routes;
pub mod models;
pub mod auth;
pub mod services;
pub mod utils;
pub mod gateway;

use sqlx::postgres::PgPool;
use sqlx::{Pool, PgConnection, Connection, Executor};

use warp::Filter;

pub static DATABASE_URL: &'static str = "postgres://postgres:password@127.0.0.1:5432/chatr";

pub type DbPool = Pool<PgConnection>;

pub fn setup_logger() -> Result<(), fern::InitError> {
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

pub async fn init_db() -> sqlx::Result<DbPool> {
    let pool = PgPool::new(DATABASE_URL).await?;


    let mut conn = pool.acquire().await?;
    conn.execute(include_str!("../schema.sql"));
    conn.close();

    Ok(pool)
}

pub async fn start() {
    let pool = init_db().await.unwrap();

    setup_logger().unwrap();

    let api = routes::users::users(pool.clone())
        .or(routes::rooms::routes(pool.clone()))
        .or(routes::auth::auth(pool.clone()))
        .or(routes::messages::messages(pool.clone()))
        .or(gateway::gateway_route(pool.clone()))
        .or(warp::fs::dir("dist"))
        .or(warp::fs::file("dist/index.html"));

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}

