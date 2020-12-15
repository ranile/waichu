use backend::setup_rejection;
use backend::utils::{error_reply, json_with_status, single_page_application, CustomRejection};
use backend::{auth, routes, setup_database, setup_logger, websocket};
use std::convert::Infallible;
use std::env;
use std::path::PathBuf;
use warp::http::StatusCode;
use warp::{Filter, Rejection};

#[tokio::main]
async fn main() {
    setup_logger().expect("unable to setup logger");

    let pool = setup_database().await.expect("unable to setup database");

    let dist_dir = env::var("DIST_DIR").expect("`DIST_DIR` isn't set");

    let hello = warp::path!("api" / "hello" / String).map(|name| format!("Hello, {}!", name));

    let routes = hello
        .or(auth::routes(pool.clone()))
        .or(websocket::route(pool.clone()))
        .or(routes::room::routes(pool.clone()))
        .or(routes::user::routes(pool.clone()))
        .or(routes::message::routes(pool.clone()))
        .or(single_page_application(PathBuf::from(&dist_dir)))
        .recover(handler)
        .with(warp::compression::gzip())
        // only if debug
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["GET", "POST", "OPTIONS"])
                .allow_headers(vec!["authorization"]),
        );

    let port = env::var("PORT")
        .map(|it| it.parse().expect("invalid port"))
        .unwrap_or(9090);

    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        });

    tokio::task::spawn(async move {
        log::info!("running server on http://{}/", addr);
        server.await;
    })
    .await
    .expect("failed to start server");
}

async fn handler(err: Rejection) -> Result<impl warp::Reply, Infallible> {
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
