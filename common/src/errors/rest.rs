use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    #[serde(skip)]
    pub status: StatusCode,
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub message: String,
}

impl ApiError {
    /// Creates a new `ApiError` with provided message and
    /// 500 [Internal Server Error][StatusCode::INTERNAL_SERVER_ERROR] error code
    pub fn new_with_message(message: &str) -> Self {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        Self {
            status,
            message: message.to_string(),
            title: title(status),
        }
    }

    /// Creates a new `ApiError` with provided message and status
    pub fn new_with_message_and_status(message: &str, status: StatusCode) -> Self {
        Self {
            status,
            message: message.to_string(),
            title: title(status),
        }
    }
}

fn title(status: StatusCode) -> String {
    status
        .canonical_reason()
        .unwrap_or("<unknown status code>")
        .to_string()
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string_pretty(&self)
            .expect("failed to convert to json -- should never happen");
        write!(f, "{}", json)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl warp::Reply for ApiError {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::with_status(warp::reply::json(&self), self.status).into_response()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl warp::reject::Reject for ApiError {}

#[cfg(not(target_arch = "wasm32"))]
impl ApiError {
    pub fn into_rejection(self) -> warp::Rejection {
        warp::reject::custom(self)
    }
}

impl std::error::Error for ApiError {
    fn description(&self) -> &str {
        &self.message
    }
}
