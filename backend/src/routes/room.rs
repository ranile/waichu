use crate::utils::{
    ensure_authorized, error_reply, is_asset_image, json_body, json_with_status, with_db,
    with_transaction, AssetExt,
};
use crate::{bail_if_err, bail_if_err_or_404, update_fields, value_or_404};
use crate::{services, utils};
use common::payloads::{CreateRoom, JoinMembers};
use common::{Asset, Room, RoomMember, User};
use sqlx::types::Uuid;
use sqlx::PgPool;
use warp::http::StatusCode;
use warp::{Filter, Reply};

async fn get_room(
    room_id: Uuid,
    pool: PgPool,
    _: User,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = bail_if_err!(pool.acquire().await.map_err(anyhow::Error::from));
    let room = bail_if_err_or_404!(services::room::get(&mut conn, room_id).await);
    Ok(warp::reply::json(&room).into_response())
}

async fn create_room(
    data: CreateRoom,
    pool: PgPool,
    user: User,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, |conn| {
        Box::pin(async move {
            let room = Room::new(&data.name);
            println!("creating room uuid: {}", room.uuid);
            let room = services::room::create(&mut *conn, room).await?;
            println!("created room");

            services::room::join(&mut *conn, &room, &user, true).await?;
            println!("joined room");

            Ok(json_with_status(StatusCode::CREATED, &room))
        })
    })
    .await
}

async fn join_room(
    room: Uuid,
    data: JoinMembers,
    pool: PgPool,
    user: User,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |conn| {
        Box::pin(async move {
            let room = services::room::get(&mut *conn, room).await?;
            let room = value_or_404!(room);

            let is_in_room = services::room::user_in_room(&mut *conn, &room, &user).await?;

            if !is_in_room {
                return Ok(error_reply(
                    StatusCode::FORBIDDEN,
                    "either this room doesn't exist or you don't have permission to join this room",
                ));
            }

            let user = value_or_404!(services::user::get(&mut *conn, data.member).await?);
            let member =
                services::room::join(&mut *conn, &room, &user, data.with_elevated_permissions)
                    .await?;

            Ok(json_with_status(StatusCode::CREATED, &member))
        })
    })
    .await
}

async fn get_room_members(
    room: Uuid,
    pool: PgPool,
    user: User,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |conn| {
        Box::pin(async move {
            let room = services::room::get(&mut *conn, room).await?;
            let room = value_or_404!(room);

            let is_in_room = services::room::user_in_room(&mut *conn, &room, &user).await?;

            if !is_in_room {
                return Ok(error_reply(
                    StatusCode::FORBIDDEN,
                    "either this room doesn't exist or you don't have permission to see it",
                ));
            }

            let users = services::room::get_room_members(conn, room).await?;
            Ok(if users.is_empty() {
                json_with_status(StatusCode::NO_CONTENT, &Vec::<RoomMember>::new())
            } else {
                warp::reply::json(&users).into_response()
            })
        })
    })
    .await
}

pub async fn room_icon(
    room: Uuid,
    pool: PgPool,
    user: User,
    asset: Asset,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |conn| {
        Box::pin(async move {
            if !is_asset_image(&asset) {
                return Ok(error_reply(
                    StatusCode::BAD_REQUEST,
                    "asset must be a PNG, JPEG or BMP",
                ));
            }

            let mut room = value_or_404!(services::room::get(&mut *conn, room).await?);
            let is_in_room = services::room::user_in_room(&mut *conn, &room, &user).await?;

            if !is_in_room {
                return Ok(error_reply(
                    StatusCode::FORBIDDEN,
                    "either this room doesn't exist or you don't have permission to see it",
                ));
            }

            if let Some(asset) = room.icon {
                room.icon = None;
                room = services::room::update(conn, room).await?;
                services::asset::delete(conn, &asset).await?;
                asset.delete().await?;
            }

            let asset = services::asset::create(conn, asset).await?;

            update_fields!(room => icon = Some(asset.clone()));
            let room = services::room::update(conn, room).await?;

            asset.save().await?;

            Ok(warp::reply::json(&room.icon.unwrap()).into_response())
        })
    })
    .await
}

pub fn routes(
    db: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let get_room_route = warp::path!("rooms" / Uuid)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db.clone()))
        .and_then(get_room);

    let create_room_route = warp::path!("rooms")
        .and(warp::post())
        .and(json_body::<CreateRoom>())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db.clone()))
        .and_then(create_room);

    let join_room_route = warp::path!("rooms" / Uuid / "join")
        .and(warp::post())
        .and(json_body::<JoinMembers>())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db.clone()))
        .and_then(join_room);

    let get_room_members_route = warp::path!("rooms" / Uuid / "members")
        .and(warp::get())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db.clone()))
        .and_then(get_room_members);

    let room_icon_route = warp::path!("rooms" / Uuid / "icon")
        .and(warp::put())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db))
        .and(utils::multipart())
        .and_then(room_icon);

    get_room_route
        .or(create_room_route)
        .or(join_room_route)
        .or(get_room_members_route)
        .or(room_icon_route)
}
