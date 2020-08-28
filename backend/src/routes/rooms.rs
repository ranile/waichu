use crate::routes::{with_db, Unauthorized, json_body, Forbidden, InternalServerError};
use sqlx::PgPool;
use warp::{Filter};
use crate::services::{room_service};
use crate::models::{Room, User};
use crate::auth::{parse_token, ensure_authorized};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use http::StatusCode;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateRoom {
    name: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JoinMembers {
    room: Uuid,
    members: Vec<Uuid>,
}

pub async fn get_room(room_id: Uuid, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    let room = room_service::get(&mut pool.acquire().await.unwrap(), &room_id).await;
    match room {
        Ok(user) => Ok(user),
        Err(_) => Err(warp::reject())
    }
}

pub async fn create_room(data: CreateRoom, pool: PgPool, user: User) -> Result<impl warp::Reply, warp::Rejection> {
    let room = Room::new(&data.name, user);
    let room = room_service::insert(&pool, room).await;
    match room {
        Ok(user) => Ok(user),
        Err(e) => {
            eprintln!("Error creating room. {}", e);
            Err(warp::reject::custom(Unauthorized))
        }
    }
}

pub async fn join_room(data: JoinMembers, pool: PgPool, user: User) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pool.begin().await.unwrap();

    let is_in_room = room_service::user_is_in_room(&mut conn, &data.room, &user.uuid)
        .await
        .unwrap_or(false);

    if !is_in_room {
        println!("is in room 1 {}", is_in_room);
        return Err(warp::reject::custom(Forbidden));
    }


    for m in &data.members {
        let res = room_service::join(&mut conn, m, &data.room).await;
        match res {
            Ok(_)  => { },
            Err(e) => return Err(warp::reject::custom(InternalServerError(e.to_string())))
        }
    }
    conn.commit().await.unwrap();
    Ok(warp::reply::with_status("", StatusCode::CREATED))
}


pub fn routes(db: PgPool) -> impl Filter<Extract=(impl warp::Reply, ), Error=warp::Rejection> + Clone {
    let get_room = warp::path!("api"/ "rooms" / Uuid)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_room);

    let create_room = warp::path!("api"/ "rooms")
        .and(warp::post())
        .and(json_body::<CreateRoom>())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db.clone()))
        .and_then(create_room);

    let join_room = warp::path!("api"/ "rooms" / "join")
        .and(warp::post())
        .and(json_body::<JoinMembers>())
        .and(with_db(db.clone()))
        .and(ensure_authorized(db.clone()))
        .and_then(join_room);

    get_room
        .or(create_room)
        .or(join_room)
}
