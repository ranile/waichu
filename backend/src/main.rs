mod auth;
pub mod macros;
mod routes;
mod services;
mod utils;
mod websocket;

use crate::utils::error_reply;
use http_api_problem::HttpApiProblem;
pub use macros::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;
use sqlx::PgPool;
use std::convert::Infallible;
use std::env;
use std::path::PathBuf;
use warp::http::StatusCode;
use warp::path::FullPath;
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
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

async fn setup_database() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    let mut conn = pool.acquire().await?;
    conn.execute(include_str!("../schema.sql")).await?;

    Ok(pool)
}

#[tokio::main]
async fn main() {
    setup_logger().expect("unable to setup logger");

    let pool = setup_database().await.expect("unable to setup database");

    let dist_dir = env::var("DIST_DIR").expect("`DIST_DIR` isn't set");

    let hello = warp::path!("api" / "hello" / String)
        .map(|name| format!("Hello, {}!", name))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Origin",
            "*",
        ));

    let routes = hello
        .or(auth::routes(pool.clone()))
        .or(websocket::route(pool.clone()))
        .or(routes::room::routes(pool.clone()))
        .or(routes::user::routes(pool.clone()))
        .or(routes::message::routes(pool.clone()))
        .or(warp::fs::dir(PathBuf::from(&dist_dir)))
        // .or(warp::get().and(warp::fs::file(PathBuf::from(format!(
        //     "{}/index.html",
        //     dist_dir
        // )))))
        .or(single_page_application(PathBuf::from(&dist_dir)))
        .recover(handler)
        // only if debug
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["GET", "POST", "OPTIONS"])
                .allow_headers(vec!["authorization"]),
        );

    warp::serve(routes).run(([0, 0, 0, 0], 9090)).await;
}

fn single_page_application(
    dist_dir: impl Into<PathBuf>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let dist_dir = dist_dir.into();

    let index_fallback = warp::path::full()
        .and(warp::fs::file(dist_dir.join("index.html")))
        .and_then(|p: FullPath, index| async move {
            if p.as_str().starts_with("/api") {
                Err(warp::reject())
            } else {
                Ok(index)
            }
        });
    warp::fs::dir(dist_dir).or(index_fallback)
}

async fn handler(err: Rejection) -> Result<impl warp::Reply, Infallible> {
    if err.is_not_found() {
        return Ok(error_reply(StatusCode::NOT_FOUND, ""));
    }

    if let Some(e) = err.find::<crate::utils::CustomRejection>() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&e.0),
            e.0.status_or_internal_server_error(),
        )
        .into_response());
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

    Ok(warp::reply::with_status(
        warp::reply::json(&HttpApiProblem::new(&message).set_status(code)),
        code,
    )
    .into_response())
}
