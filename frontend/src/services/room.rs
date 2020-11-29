use crate::services::url;
use crate::CLIENT as client;
use common::payloads::{CreateMessagePayload, CreateRoom, JoinMembers};
use common::{Message, Room, RoomMember, User};
use reqwest::header::AUTHORIZATION;
use uuid::Uuid;

pub async fn create_room(token: &str, name: &str) -> anyhow::Result<Room> {
    let data = CreateRoom {
        name: name.to_string(),
    };

    Ok(client
        .post(&url("/api/rooms"))
        .body(serde_json::to_string(&data)?)
        .header(AUTHORIZATION, token)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn fetch_room_members(token: &str, room_id: Uuid) -> anyhow::Result<Vec<RoomMember>> {
    Ok(client
        .get(&url(&format!("/api/rooms/{}/members", room_id)))
        .header(AUTHORIZATION, token)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn join_room(token: &str, room_id: Uuid, username: &str) -> anyhow::Result<RoomMember> {
    let user: User = client
        .get(&url(&format!("/api/users/by_username/{}", username)))
        .header(AUTHORIZATION, token)
        .send()
        .await?
        .json()
        .await?;

    let body = JoinMembers {
        member: user.uuid,
        with_elevated_permissions: false,
    };

    let member = client
        .post(&url(&format!("/api/rooms/{}/join", room_id)))
        .header(AUTHORIZATION, token)
        .body(serde_json::to_string(&body)?)
        .send()
        .await?
        .json()
        .await?;

    Ok(member)
}

pub async fn fetch_room_messages(token: &str, room_id: Uuid) -> anyhow::Result<Vec<Message>> {
    Ok(client
        .get(&url(&format!("/api/rooms/{}/messages", room_id)))
        .header(AUTHORIZATION, token)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn send_message(
    token: &str,
    room_id: Uuid,
    message: &CreateMessagePayload,
) -> anyhow::Result<Message> {
    Ok(client
        .post(&url(&format!("/api/rooms/{}/messages", room_id)))
        .body(serde_json::to_string(message)?)
        .header(AUTHORIZATION, token)
        .send()
        .await?
        .json()
        .await?)
}
