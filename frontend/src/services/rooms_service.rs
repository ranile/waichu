use crate::models::Room;
use crate::services::get_token;
use std::collections::HashMap;
use crate::utils::requests;
use crate::utils::requests::{parse_resp_as_json, FetchError};
use serde::Serialize;

#[derive(Serialize)]
struct CreateRoom {
    name: String
}

pub async fn create(name: String) -> Result<Room, FetchError> {
    let mut headers = HashMap::new();
    headers.insert("Authorization", get_token().unwrap());

    let resp = requests::post("/api/rooms/", CreateRoom { name }, headers).await?;
    parse_resp_as_json::<Room>(resp).await
}
