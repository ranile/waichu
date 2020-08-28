use crate::routes::{Unauthorized, Forbidden, InternalServerError, json_body, with_db};
use crate::auth::{parse_token, ensure_authorized};
use crate::services::{room_service, message_service};
use uuid::Uuid;
use sqlx::{PgPool, Connection};
use warp::Filter;
use serde::Deserialize;
use crate::models::User;

#[derive(Deserialize)]
pub struct MessagePayload {
    content: String
}

pub async fn create_message(room_id: Uuid, data: MessagePayload, pool: PgPool, user: User) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pool.begin().await.unwrap();
    let is_in_room = room_service::user_is_in_room(&mut conn, &room_id, &user.uuid)
        .await
        .unwrap_or(false);
    conn.close();
    if !is_in_room {
        return Err(warp::reject::custom(Forbidden));
    }

    let message = message_service::create(
        &pool,
        &data.content,
        &user.uuid,
        &room_id,
    ).await;

    let message = match message {
        Ok(message) => message,
        Err(e) => { return Err(warp::reject::custom(InternalServerError(e.to_string()))); }
    };

    Ok(message)
}

async fn get_messages(room_id: Uuid, pool: PgPool, user: User) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pool.acquire().await.unwrap();

    let is_in_room = room_service::user_is_in_room(&mut conn, &room_id, &user.uuid)
        .await
        .unwrap_or(false);
    if !is_in_room {
        return Err(warp::reject::custom(Forbidden));
    }

    let messages = message_service::get_all(conn, &room_id).await;

    let messages = match messages {
        Ok(messages) => messages,
        Err(e) => { return Err(warp::reject::custom(InternalServerError(e.to_string()))); }
    };

    Ok(warp::reply::json(&messages))
}


pub fn messages(pool: PgPool) -> impl Filter<Extract=(impl warp::Reply, ), Error=warp::Rejection> + Clone {
    let create_message = warp::path!("api"/ "rooms" / Uuid / "messages")
        .and(warp::post())
        .and(json_body::<MessagePayload>())
        .and(with_db(pool.clone()))
        .and(ensure_authorized(pool.clone()))
        .and_then(create_message);

    let get_messages = warp::path!("api"/ "rooms" / Uuid / "messages")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and(ensure_authorized(pool.clone()))
        .and_then(get_messages);

    create_message
        .or(get_messages)
}

