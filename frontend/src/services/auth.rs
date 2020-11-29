use crate::services::url;
use crate::CLIENT as client;
use common::payloads::{Credentials, JwtToken};

pub async fn signin(credentials: Credentials) -> anyhow::Result<JwtToken> {
    Ok(client
        .post(&url("/api/auth/signin"))
        .body(serde_json::to_string(&credentials)?)
        .send()
        .await?
        .json::<JwtToken>()
        .await?)
}

pub async fn signup(credentials: Credentials) -> anyhow::Result<JwtToken> {
    Ok(client
        .post(&url("/api/auth/signup"))
        .body(serde_json::to_string(&credentials)?)
        .send()
        .await?
        .json::<JwtToken>()
        .await?)
}
