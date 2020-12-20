use crate::request;
use common::payloads::{Credentials, JwtToken};

pub async fn signin(credentials: Credentials) -> anyhow::Result<JwtToken> {
    request!(method = POST, url = "/api/auth/signin", body = &credentials).await
}

pub async fn signup(credentials: Credentials) -> anyhow::Result<JwtToken> {
    request!(method = POST, url = "/api/auth/signup", body = &credentials).await
}
