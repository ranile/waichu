use uuid::Uuid;
use std::collections::HashMap;
use crate::models::Message;
use crate::utils::requests;
use crate::utils::requests::{parse_resp_as_json, FetchResult};
use serde::{Deserialize, Serialize};
use crate::services::{get_token, user_service};

#[derive(Deserialize, Serialize, Debug)]
pub struct FetchedMessage {
    pub uuid: Uuid,
    pub content: String,
    pub author: Uuid,
    pub room: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl FetchedMessage {
    pub async fn to_message(&self) -> FetchResult<Message> {
        Ok(Message {
            uuid: self.uuid,
            content: self.content.clone(),
            author: user_service::get(self.author).await?,
            room: self.room,
            created_at: self.created_at,
        })
    }
}

pub async fn get_room_messages(room_id: Uuid) -> FetchResult<Vec<Message>> {
    let mut headers = HashMap::new();
    headers.insert("Authorization", get_token().unwrap());

    let url = format!("/api/rooms/{}/messages", room_id);
    let resp = requests::get(url, headers).await?;
    let data = parse_resp_as_json::<Vec<FetchedMessage>>(resp).await?;

    let mut new_data = Vec::with_capacity(data.len());
    for val in data {
        new_data.push(val.to_message().await?)
    }

    Ok(new_data)
}

#[derive(Serialize)]
struct CreateMessagePayload {
    content: String
}

pub async fn send_message(room_id: Uuid, content: String) -> FetchResult<Message> {
    let mut headers = HashMap::new();
    headers.insert("Authorization", get_token().unwrap());

    let url = format!("/api/rooms/{}/messages", room_id);
    let resp = requests::post(&url, CreateMessagePayload { content }, headers).await?;

    let data = parse_resp_as_json::<FetchedMessage>(resp).await?;

    data.to_message().await
}
