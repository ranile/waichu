use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sqlx::{FromRow, Type};

use chrono::{DateTime, Utc};
use bcrypt::{DEFAULT_COST, hash, BcryptResult};
use crate::utils::serializer::GetUuid;

use warp::Reply;
use warp::reply::Response;

#[derive(Serialize, Deserialize, FromRow, Debug, Type, Clone)]
pub struct User {
    pub username: String,
    pub uuid: Uuid,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: &str, password: &str) -> BcryptResult<User> {
        let hash = hash(password, DEFAULT_COST)?;
        Ok(User { username: username.to_string(), uuid: Uuid::new_v4(), password: hash, created_at: Utc::now() })
    }

    pub fn dummy() -> Self {
        User { username: "name".to_string(), uuid: Uuid::new_v4(), password: hash("password", DEFAULT_COST).unwrap(), created_at: Utc::now() }
    }
}

impl GetUuid for User {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

impl Reply for User {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string_pretty(&self).unwrap().into())
    }
}
