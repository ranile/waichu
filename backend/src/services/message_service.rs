use crate::DbPool;
use crate::models::{Message, User};
use sqlx::postgres::PgQueryAs;
use crate::gateway::WsEventHooks;
use sqlx::{FromRow, PgConnection};
use sqlx::types::Uuid;
use chrono::{DateTime, Utc};
use crate::services::{room_service, user_service};
use sqlx::pool::PoolConnection;


#[derive(FromRow, Debug)]
struct ReturnedMessage {
    pub uuid: Uuid,
    pub content: String,
    pub author: Uuid,
    pub room: Uuid,
    pub created_at: DateTime<Utc>,
}

pub async fn create(
    pool: &DbPool,
    content: &str,
    author_uuid: &Uuid,
    room_uuid: &Uuid,
) -> sqlx::Result<Message> {
    println!("message create 1");
    let mut conn = pool.begin().await?;

    let m: ReturnedMessage = sqlx::query_as(r#"
        insert into messages (uuid, content, author, room)
        values ($1, $2, $3, $4)
        returning *;
    "#)
        .bind(Uuid::new_v4())
        .bind(content)
        .bind(author_uuid)
        .bind(room_uuid)
        .fetch_one(&mut conn)
        .await?;

    println!("message create 2");

    let room = room_service::get(&mut conn, &m.room).await?;
    let user = user_service::get(&mut conn, m.author).await?;

    println!("message create 3");
    let message = Message {
        uuid: m.uuid,
        content: m.content,
        author: user,
        room,
        created_at: m.created_at,
    };
    println!("message create 4");
    message.on_create(&mut conn).await;
    conn.commit().await?;
    println!("message create 5");
    Ok(message)
}

pub async fn get_all(mut conn: PoolConnection<PgConnection>, room_id: &Uuid) -> sqlx::Result<Vec<Message>> {
    let results = sqlx::query!(r#"
select
       messages.uuid as message_id,
       messages.content as message_content,
       messages.created_at as message_created_at,
       messages.room as message_room_id,
       users.uuid as author_uuid,
       users.username as author_username,
       users.created_at as author_created_at
from messages
         left join users on messages.author = users.uuid
where room = $1
order by messages.created_at desc;
    "#, room_id.clone())
        .fetch_all(&mut conn)
        .await?;
    if results.len() == 0 {
        return Ok(vec![]);
    }
    let room = room_service::get(&mut conn, &results[0].message_room_id).await?;

    let mut vec = Vec::with_capacity(results.len());
    for resp in results {
        let m = Message {
            uuid: resp.message_id,
            content: resp.message_content,
            author: User {
                username: resp.author_username,
                uuid: resp.author_uuid,
                password: "".to_string(),
                created_at: resp.author_created_at,
            },
            room: room.clone(),
            created_at: resp.message_created_at,
        };
        vec.push(m);
    }
    Ok(vec)
}
