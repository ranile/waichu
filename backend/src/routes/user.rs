use crate::utils::{
    ensure_authorized, error_reply, is_asset_image, with_db, with_transaction, AssetExt,
};
use crate::{bail_if_err, bail_if_err_or_404, update_fields};
use crate::{services, utils};
use common::{Asset, User};
use sqlx::types::Uuid;
use sqlx::PgPool;
use warp::http::StatusCode;
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

async fn update_avatar(
    pool: PgPool,
    mut user: User,
    asset: Asset,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, |conn| {
        Box::pin(async move {
            if !is_asset_image(&asset) {
                return Ok(error_reply(
                    StatusCode::BAD_REQUEST,
                    "asset must be a PNG, JPEG or BMP",
                ));
            }

            if let Some(asset) = user.avatar {
                user.avatar = None;
                user = services::user::update(conn, user).await?;
                services::asset::delete(conn, &asset).await?;
                asset.delete().await?;
            }

            let asset = services::asset::create(conn, asset).await?;

            update_fields!(user => avatar = Some(asset.clone()));
            let user = services::user::update(conn, user).await?;

            asset.save().await.map_err(|e| {
                println!("e: {}", e);
                e
            })?;

            Ok(warp::reply::json(&user.avatar.unwrap()).into_response())
        })
    })
    .await
}

pub fn routes(
    db: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let get_user_route = warp::path!("users" / Uuid)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_user);

    let get_by_username_route = warp::path!("users" / "by_username" / String)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_by_username);

    let get_me_route = warp::path!("users" / "me")
        .and(warp::get())
        .and(ensure_authorized(db.clone()))
        .and_then(get_me);

    let update_avatar_route = warp::path!("users" / "me" / "avatar")
        .and(warp::put())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db))
        .and(utils::multipart())
        .and_then(update_avatar);

    get_me_route
        .or(get_user_route)
        .or(get_by_username_route)
        .or(update_avatar_route)
}
