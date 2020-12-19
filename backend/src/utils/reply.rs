use common::errors::ApiError;
use serde::Serialize;
use warp::http::StatusCode;
use warp::reply::Response;
use warp::Reply;

pub fn error_reply(status: StatusCode, message: &str) -> Response {
    ApiError::new_with_message_and_status(message, status).into_response()
}

pub fn json_with_status<T>(status: StatusCode, json: &T) -> Response
where
    T: Serialize,
{
    warp::reply::with_status(warp::reply::json(&json), status).into_response()
}
