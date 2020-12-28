pub mod auth;
#[macro_use]
pub mod macros;
pub mod routes;
pub mod services;
pub mod utils;
pub mod websocket;

pub use macros::*;

use crate::utils::{error_reply, ASSETS_PATH};
use common::errors::ApiError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs::OpenOptions;
use std::{env, io};
use tokio::fs;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

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
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .append(false)
                .open("backend.log")?,
        )
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

pub async fn setup_assets_directory() -> anyhow::Result<String> {
    let assets_path = env::var("ASSETS_PATH")?;
    println!("{}", assets_path);
    if let Err(err) = fs::read_dir(&assets_path).await {
        if let io::ErrorKind::NotFound = err.kind() {
            anyhow::bail!("assets directory doesn't exists");
        }
    };
    ASSETS_PATH.lock().await.replace(assets_path.clone());

    Ok(assets_path)
}

pub fn api(pool: PgPool) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    let prefix = warp::path!("api" / ..);

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let auth = auth::routes(pool.clone());
    let websocket = websocket::route(pool.clone());
    let room = routes::room::routes(pool.clone());
    let user = routes::user::routes(pool.clone());
    let message = routes::message::routes(pool.clone());
    let asset = routes::assets::routes(pool);

    let api =
        balanced_or_tree!(hello, auth, websocket, room, user, message, asset).recover(handler);
    prefix.and(api)
}

async fn handler(err: Rejection) -> Result<impl warp::Reply, Rejection> {
    if err.is_not_found() {
        return Ok(error_reply(StatusCode::NOT_FOUND, ""));
    }

    if let Some(e) = err.find::<ApiError>() {
        return Ok(e.clone().into_response());
    }

    let mut code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut message = "Internal server error".to_string();

    setup_rejection!(err code message
        warp::reject::MissingHeader, StatusCode::BAD_REQUEST;
        warp::reject::MethodNotAllowed, StatusCode::METHOD_NOT_ALLOWED;
        warp::reject::InvalidHeader, StatusCode::BAD_REQUEST;
        warp::reject::MissingCookie, StatusCode::BAD_REQUEST;
        warp::reject::InvalidQuery, StatusCode::BAD_REQUEST;
        warp::reject::LengthRequired, StatusCode::BAD_REQUEST;
        warp::reject::PayloadTooLarge, StatusCode::BAD_REQUEST;
        warp::reject::UnsupportedMediaType, StatusCode::BAD_REQUEST;
        warp::body::BodyDeserializeError, StatusCode::BAD_REQUEST;
        warp::ws::MissingConnectionUpgrade, StatusCode::BAD_REQUEST;
        warp::ext::MissingExtension, StatusCode::INTERNAL_SERVER_ERROR
    );

    Ok(error_reply(code, &message))
}
