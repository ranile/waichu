use crate::routes::{with_db, Unauthorized};
use sqlx::PgPool;
use warp::{Filter};
use crate::services::user_service;
use crate::auth::{parse_token, ensure_authorized};
use uuid::Uuid;
use crate::models::User;

async fn get_user(username: Uuid, _pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    let user = user_service::get(&mut _pool.acquire().await.unwrap(), username).await;
    match user {
        Ok(user) => Ok(user),
        Err(_) => Err(warp::reject())
    }
}

async fn get_me(pool: PgPool, user: User) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(user)
}

pub fn users(db: PgPool) -> impl Filter<Extract=(impl warp::Reply, ), Error=warp::Rejection> + Clone {
    let get_user_route = warp::path!("api" / "users" / Uuid)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_user);

    let get_me_route = warp::path!("api" / "users"/ "@me")
        .and(warp::get())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db))
        .and_then(get_me);

    get_me_route
        .or(get_user_route)
}
