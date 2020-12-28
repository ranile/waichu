use crate::request;
use crate::services::request::NoContent;
use common::payloads::{CreateMessage, CreateRoom, JoinMembers};
use common::{Message, Room, RoomMember, User};
use uuid::Uuid;

pub async fn create_room(token: &str, name: &str) -> anyhow::Result<Room> {
    let data = CreateRoom {
        name: name.to_string(),
    };

    request!(
        method = POST,
        url = "/api/rooms",
        body = &data,
        token = token
    )
    .await
}

pub async fn fetch_room_members(token: &str, room_id: Uuid) -> anyhow::Result<Vec<RoomMember>> {
    request!(
        method = GET,
        url = format!("/api/rooms/{}/members", room_id),
        token = token
    )
    .await
}

pub async fn join_room(token: &str, room_id: Uuid, username: &str) -> anyhow::Result<RoomMember> {
    let user: User = request!(
        method = GET,
        url = format!("/api/users/by_username/{}", username),
        token = token
    )
    .await?;

    let body = JoinMembers {
        member: user.uuid,
        with_elevated_permissions: false,
    };

    let member = request!(
        method = POST,
        url = format!("/api/rooms/{}/join", room_id),
        body = &body,
        token = token
    )
    .await?;

    Ok(member)
}

pub async fn fetch_room_messages(token: &str, room_id: Uuid) -> anyhow::Result<Vec<Message>> {
    let res = request!(
        method = GET,
        url = format!("/api/rooms/{}/messages", room_id),
        token = token
    )
    .await;

    weblog::console_log!(format!("res {:?}", res));
    match res {
        Ok(res) => Ok(res),
        Err(e) => match e.downcast::<NoContent>() {
            Ok(_) => Ok(vec![]),
            Err(e) => Err(e),
        },
    }
}

pub async fn send_message(
    token: &str,
    room_id: Uuid,
    message: &CreateMessage,
) -> anyhow::Result<Message> {
    request!(
        method = POST,
        url = format!("/api/rooms/{}/messages", room_id),
        body = message,
        token = token
    )
    .await
}
