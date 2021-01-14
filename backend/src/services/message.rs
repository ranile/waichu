use crate::websocket;
use common::websocket::{MessagePayload, OpCode};
use common::{Asset, Message, MessageType, Room, User};
use sqlx::PgConnection;
use std::sync::Arc;

pub async fn create(db: &mut PgConnection, message: Message) -> anyhow::Result<Message> {
    let Message {
        uuid,
        author,
        room,
        content,
        created_at: _,
        type_,
    } = message;

    let inserted = sqlx::query!(
        r#"
            insert into messages(uuid, author, room, content, type)
            values ($1, $2, $3, $4, $5)
            returning uuid, content, room, created_at, type as "type_: MessageType";
        "#,
        uuid,
        author.uuid,
        room.uuid,
        content,
        type_ as _,
    )
    .fetch_one(db)
    .await?;

    let message = Message {
        uuid: inserted.uuid,
        author,
        room,
        content: inserted.content,
        created_at: inserted.created_at,
        type_: inserted.type_,
    };

    websocket::send_message(
        Arc::new(MessagePayload {
            op: OpCode::MessageCreate,
            data: message.clone(), // maybe find a way to do this without cloning
        }),
        |_uuid| true, /* TODO only send to users in a room */
    )
    .await;

    Ok(message)
}

pub async fn get_all(conn: &mut PgConnection, room: &Room) -> anyhow::Result<Vec<Message>> {
    let returned = sqlx::query!(
        r#"
select messages.uuid,
       messages.content,
       messages.room,
       messages.created_at,
       messages.type as "type_: MessageType",
       u.username    as author_username,
       u.uuid        as author_uuid,
       u.password    as author_password,
       u.created_at  as author_created_at,
       u.avatar      as author_avatar,
       a.uuid        as "asset_uuid?",
       a.created_at  as "asset_created_at?"
from messages
         left join users u on u.uuid = messages.author
         left join assets a on u.avatar = u.avatar
where room = $1
order by messages.created_at desc ;
    "#,
        room.uuid
    )
    .fetch_all(conn)
    .await?;

    let messages = returned
        .into_iter()
        .map(|value| Message {
            uuid: value.uuid,
            author: User {
                uuid: value.author_uuid,
                username: value.author_username,
                password: value.author_password,
                created_at: value.author_created_at,
                avatar: match value.asset_uuid {
                    Some(_) => Some(Asset {
                        uuid: value.asset_uuid.unwrap(),
                        bytes: Arc::new(vec![]),
                        created_at: value.asset_created_at.unwrap(),
                    }),
                    None => None,
                },
            },
            room: room.clone(),
            content: value.content,
            created_at: value.created_at,
            type_: value.type_,
        })
        .collect::<Vec<Message>>();

    Ok(messages)
}
