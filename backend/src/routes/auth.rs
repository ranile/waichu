use crate::routes::{with_db, json_body, InternalServerError, BadRequest};
use sqlx::PgPool;
use std::convert::Infallible;
use warp::{Filter, Reply};
use crate::services::user_service;
use crate::auth::{create_jwt};
use http::StatusCode;
use bcrypt::verify;
use serde::{Deserialize, Serialize};
use warp::reply::Response;
use crate::models::User;

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct TokenPayload {
    token: String,
}

impl Reply for TokenPayload {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string_pretty(&self).unwrap().into())
    }
}

async fn signup(pool: PgPool, credentials: Credentials) -> Result<impl warp::Reply, warp::Rejection> {
    let user = User::new(&credentials.username, &credentials.password);
    let user = match user {
        Ok(user) => user,
        Err(e) => { return Err(warp::reject::custom(BadRequest { msg: e.to_string() })); }
    };

    println!("user {:#?}", user);
    let inserted = user_service::insert(&pool, user).await;
    println!("inserted {:#?}", inserted);

    let inserted = match inserted {
        Ok(user) => user,
        Err(e) => { return Err(warp::reject::custom(BadRequest { msg: e.to_string() })); }
    };

    let token = create_jwt(&inserted);
    let token = match token {
        Ok(token) => token,
        Err(e) => { return Err(warp::reject::custom(InternalServerError(e.to_string()))); }
    };

    Ok(TokenPayload { token })
}

async fn signin(pool: PgPool, credentials: Credentials) -> Result<impl warp::Reply, Infallible> {
    let user = user_service::get_by_username(&pool, credentials.username.clone()).await;
    let user = match user {
        Ok(user) => user,
        Err(e) => return Ok(warp::reply::with_status(e.to_string(), StatusCode::NOT_FOUND))
    };

    if !verify(&credentials.password, &user.password).unwrap_or(false) {
        return Ok(warp::reply::with_status("".to_string(), StatusCode::NOT_FOUND));
    }

    let token = create_jwt(&user);
    let token = match token {
        Ok(token) => token,
        Err(e) => { return Ok(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)); }
    };

    Ok(warp::reply::with_status(serde_json::to_string_pretty(&TokenPayload { token }).unwrap(), StatusCode::OK))
}

pub fn auth(db: PgPool) -> impl Filter<Extract=(impl warp::Reply, ), Error=warp::Rejection> + Clone {
    let signup_route = warp::path!("api" / "auth" / "signup")
        .and(warp::post())
        .and(with_db(db.clone()))
        .and(json_body::<Credentials>())
        .and_then(signup);

    let signin_route = warp::path!("api" / "auth"/ "signin")
        .and(warp::post())
        .and(with_db(db.clone()))
        .and(json_body::<Credentials>())
        .and_then(signin);

    signup_route
        .or(signin_route)
}
