pub mod auth;
#[macro_use]
pub mod macros;
pub mod routes;
pub mod services;
pub mod utils;
pub mod websocket;

pub use macros::*;

use crate::utils::{error_reply, ASSETS_PATH};
use anyhow::Context;
use common::errors::ApiError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io::ErrorKind;
use std::path::Path;
use std::str::FromStr;
use std::{env, io};
use tokio::fs;
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

pub fn setup_logger(file_writer: NonBlocking) -> anyhow::Result<()> {
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "debug,hyper=info,sqlx::query=warn".to_owned());
    let filter = EnvFilter::from_str(&filter)?;
    //
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::Subscriber::new()
                .with_target(true)
                .with_level(true)
                .with_ansi(true)
                .with_timer(fmt::time::ChronoLocal::with_format(
                    "%I:%M:%S %p".to_string(),
                ))
                .with_writer(io::stdout),
        )
        .with(
            fmt::Subscriber::new()
                .with_target(true)
                .with_level(true)
                .with_ansi(false)
                .with_writer(file_writer),
        )
        .init();
    Ok(())
}

pub async fn setup_database() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .connect(
            &env::var("DATABASE_URL").context("environment variable `DATABASE_URL` not defined")?,
        )
        .await
        .context("failed to connect to database")?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    Ok(pool)
}

pub async fn setup_assets_directory() -> anyhow::Result<String> {
    let assets_path =
        env::var("ASSETS_PATH").context("environment variable `ASSETS_PATH` not defined")?;
    anyhow::ensure!(
        exists(&assets_path).await?,
        "assets directory doesn't exists"
    );

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

pub async fn exists(path: impl AsRef<Path>) -> anyhow::Result<bool> {
    Ok(fs::metadata(path.as_ref())
        .await
        .map(|_| true)
        .or_else(|error| {
            if error.kind() == ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(error)
            }
        })?)
}
