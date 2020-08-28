use crate::models::{Room, RoomMember, User};
use sqlx::prelude::PgQueryAs;
use crate::DbPool;
use sqlx::{PgConnection, FromRow, PgPool};
use sqlx::pool::PoolConnection;
use uuid::Uuid;
use crate::services::user_service;
use crate::gateway::WsEventHooks;

pub async fn insert(pool: &PgPool, room: Room) -> sqlx::Result<Room> {
    let mut conn = pool.acquire().await?;

    let query = sqlx::query!("
        insert into rooms(uuid, name, owner) VALUES ($1, $2, $3)
        returning *;
    ", room.uuid, room.name, room.owner.uuid)
        .fetch_one(&mut conn)
        .await?;

    let user = user_service::get(&mut conn, query.owner).await?;
    let room = Room {
        uuid: query.uuid,
        name: query.name,
        owner: user,
        created_at: query.created_at,
        member_count: 1,
    };
    room.on_create(&mut conn).await;
    Ok(room)
}

pub async fn join(conn: &mut PoolConnection<PgConnection>, user_id: &Uuid, room_id: &Uuid) -> sqlx::Result<u64> {
    sqlx::query("
        insert into room_members(user_id, room_id)
        values ($1, $2)
        returning *;
    ")
        .bind(user_id)
        .bind(room_id)
        .execute(conn)
        .await
}

#[derive(FromRow)]
struct Count {
    count: i64
}

pub async fn user_is_in_room(conn: &mut PoolConnection<PgConnection>, room: &Uuid, member: &Uuid) -> sqlx::Result<bool> {
    let row: Count = sqlx::query_as("
        select count(*) from room_members
        where room_id = $1 and user_id = $2;
    ")
        .bind(room)
        .bind(member)
        .fetch_one(conn)
        .await?;

    println!("count {}", row.count);

    Ok(row.count == 1)
}


pub async fn get(conn: &mut PoolConnection<PgConnection>, room: &Uuid) -> sqlx::Result<Room> {
    let row = sqlx::query!(r#"
select rooms.uuid as room_uuid,
       rooms.created_at as room_created_at,
       rooms.name as room_name,
       users.username as user_username,
       users.uuid as user_uuid,
       users.created_at as user_created_at,
       users.password as user_password,
       (select count(*) from room_members where room_members.room_id = rooms.uuid) as member_count
from rooms
         inner join users
                    on rooms.owner = users.uuid
where rooms.uuid = $1
limit 1;
    "#, room.clone())
        .fetch_one(conn)
        .await?;

    Ok(Room {
        uuid: row.room_uuid,
        name: row.room_name,
        owner: User {
            username: row.user_username,
            uuid: row.user_uuid,
            created_at: row.user_created_at,
            password: row.user_password,
        },
        created_at: row.room_created_at,
        member_count: row.member_count.unwrap_or(0),
    })
}

pub async fn get_with_user(pool: &DbPool, user_id: &Uuid) -> sqlx::Result<Vec<Room>> {
    let mut tx = pool.begin().await?;

    let rows = sqlx::query!(r#"
select rooms.uuid                                                                  as room_uuid,
       rooms.created_at                                                            as room_created_at,
       rooms.name                                                                  as room_name,
       users.username                                                              as user_username,
       users.uuid                                                                  as user_uuid,
       users.created_at                                                            as user_created_at,
       users.password                                                              as user_password,
       (select count(*) from room_members where room_members.room_id = rooms.uuid) as member_count
from rooms
         inner join users
                    on rooms.owner = users.uuid
where rooms.uuid = any
      (select room_id from room_members where room_members.user_id = $1);
    "#, user_id.clone())
        .fetch_all(&mut tx)
        .await?;


    let mut rooms = Vec::with_capacity(rows.len());
    for row in rows {
        rooms.push(Room {
            uuid: row.room_uuid,
            name: row.room_name,
            owner: User {
                username: row.user_username,
                uuid: row.user_uuid,
                created_at: row.user_created_at,
                password: row.user_password,
            },
            created_at: row.room_created_at,
            member_count: row.member_count.unwrap_or(0),
        })
    }

    Ok(rooms)
}

pub async fn get_room_members(conn: &mut PoolConnection<PgConnection>, room_id: &Uuid) -> sqlx::Result<Vec<RoomMember>> {
    sqlx::query_as(r#"
select user_id as uuid, username, created_at, joined_at
from room_members
         left join users on room_members.user_id = users.uuid
where room_id = $1;
    "#)
        .bind(room_id)
        .fetch_all(conn)
        .await
}
