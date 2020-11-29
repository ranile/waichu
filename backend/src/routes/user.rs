use crate::services;
use crate::utils::{ensure_authorized, with_db};
use crate::{bail_if_err, bail_if_err_or_404};
use common::User;
use sqlx::types::Uuid;
use sqlx::PgPool;
use warp::{Filter, Reply};

async fn get_user(uuid: Uuid, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = bail_if_err!(pool.acquire().await.map_err(anyhow::Error::from));
    let user = bail_if_err_or_404!(services::user::get(&mut conn, uuid).await);

    Ok(warp::reply::json(&user).into_response())
}

async fn get_by_username(
    username: String,
    pool: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = bail_if_err!(pool.acquire().await.map_err(anyhow::Error::from));

    let user = bail_if_err_or_404!(services::user::get_by_username(&mut conn, &username).await);

    Ok(warp::reply::json(&user).into_response())
}

async fn get_me(user: User) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&user))
}

pub fn routes(
    db: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let get_user_route = warp::path!("api" / "users" / Uuid)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_user);

    let get_by_username_route = warp::path!("api" / "users" / "by_username" / String)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_by_username);

    let get_me_route = warp::path!("api" / "users" / "me")
        .and(warp::get())
        .and(ensure_authorized(db))
        .and_then(get_me);

    get_me_route.or(get_user_route).or(get_by_username_route)
}
