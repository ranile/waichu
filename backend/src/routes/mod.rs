use warp::Filter;
use sqlx::PgPool;
use serde::de::DeserializeOwned;

pub mod rooms;
pub mod users;
pub mod auth;
pub mod messages;

pub fn with_db(db: PgPool) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}


#[derive(Debug)]
pub struct Unauthorized;

impl warp::reject::Reject for Unauthorized {}

#[derive(Debug)]
pub struct Forbidden;

impl warp::reject::Reject for Forbidden {}

#[derive(Debug)]
pub struct InternalServerError(String);

impl warp::reject::Reject for InternalServerError {}

#[derive(Debug)]
pub struct BadRequest {
    msg: String
}

impl warp::reject::Reject for BadRequest {}

pub fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
