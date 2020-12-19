pub mod auth;
#[macro_use]
pub mod macros;
pub mod routes;
pub mod services;
pub mod utils;
pub mod websocket;

pub use macros::*;

use crate::utils::{error_reply, json_with_status, CustomRejection};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use warp::http::StatusCode;
use warp::{Filter, Rejection};

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

pub fn api(pool: PgPool) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    let prefix = warp::path!("api" / ..);

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let auth = auth::routes(pool.clone());
    let websocket = websocket::route(pool.clone());
    let room = routes::room::routes(pool.clone());
    let user = routes::user::routes(pool.clone());
    let message = routes::message::routes(pool);

    let api = balanced_or_tree!(hello, auth, websocket, room, user, message).recover(handler);
    prefix.and(api)
}

async fn handler(err: Rejection) -> Result<impl warp::Reply, Rejection> {
    if err.is_not_found() {
        return Ok(error_reply(StatusCode::NOT_FOUND, ""));
    }

    if let Some(e) = err.find::<CustomRejection>() {
        return Ok(json_with_status(
            e.0.status_or_internal_server_error(),
            &e.0,
        ));
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
