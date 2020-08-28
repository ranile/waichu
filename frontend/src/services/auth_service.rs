use serde::{Serialize, Deserialize};
use crate::utils::requests::parse_resp_as_json;
use crate::utils::requests;
use std::collections::HashMap;


#[derive(Debug, Serialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct TokenData {
    pub token: String
}

pub async fn signin(credentials: Credentials) -> TokenData {
    let resp = requests::post("/api/auth/signin", credentials, HashMap::new()).await.unwrap();
    parse_resp_as_json::<TokenData>(resp).await.unwrap()
}
