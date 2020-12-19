use crate::services;
use crate::utils::{
    ensure_authorized, error_reply, json_body, json_with_status, with_db, with_transaction,
};
use crate::value_or_404;
use common::payloads::CreateMessage;
use common::{Message, User};
use sqlx::types::Uuid;
use sqlx::PgPool;
use warp::http::StatusCode;
use warp::{Filter, Reply};

async fn create_message(
    room_id: Uuid,
    data: CreateMessage,
    pool: PgPool,
    user: User,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |conn| {
        Box::pin(async move {
            if data.content.is_empty() {
                return Ok(error_reply(
                    StatusCode::BAD_REQUEST,
                    "message content can't be empty",
                ));
            }
            let room = value_or_404!(services::room::get(conn, room_id).await?);
            if !services::room::user_in_room(conn, &room, &user).await? {
                return Ok(error_reply(
                    StatusCode::FORBIDDEN,
                    "you do not have permission to message here",
                ));
            };

            let message = Message::new(user, room, data.content);
            let message = services::message::create(conn, message).await?;

            Ok(json_with_status(StatusCode::CREATED, &message))
        })
    })
    .await
}

async fn get_messages(
    room_id: Uuid,
    pool: PgPool,
    user: User,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |conn| {
        Box::pin(async move {
            let room = value_or_404!(services::room::get(conn, room_id).await?);
            if !services::room::user_in_room(conn, &room, &user).await? {
                return Ok(error_reply(
                    StatusCode::FORBIDDEN,
                    "you must be in the room to get its messages",
                ));
            };

            let messages = services::message::get_all(conn, &room).await?;

            let status = if messages.is_empty() {
                StatusCode::NO_CONTENT
            } else {
                StatusCode::OK
            };

            Ok(json_with_status(status, &messages))
        })
    })
    .await
}

pub fn routes(
    pool: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let create_message = warp::path!("rooms" / Uuid / "messages")
        .and(warp::post())
        .and(json_body::<CreateMessage>())
        .and(with_db(pool.clone()))
        .and(ensure_authorized(pool.clone()))
        .and_then(create_message);

    let get_messages = warp::path!("rooms" / Uuid / "messages")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and(ensure_authorized(pool))
        .and_then(get_messages);

    // let get_message = warp::path!("rooms" / Uuid / "messages" / Uuid)
    //     .and(warp::get())
    //     .and(with_db(pool.clone()))
    //     .and(ensure_authorized(pool.clone()))
    //     .and_then(get_message);

    create_message.or(get_messages)
}
