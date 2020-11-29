use crate::services;
use common::payloads::JwtToken;
use common::User;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::PgConnection;

const KEY: &[u8; 6] = b"secret";

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub subject: String,
    pub company: String,
    pub exp: usize,
}

pub fn create_jwt(user: &User) -> anyhow::Result<JwtToken> {
    let my_claims = Claims {
        subject: user.uuid.to_string(),
        company: "waichu".to_owned(),
        exp: 10000000000,
    };

    let mut header = Header::new(Algorithm::HS512);
    header.kid = Some("signing_key".to_owned());

    let token = encode(&header, &my_claims, &EncodingKey::from_secret(KEY))?;
    Ok(JwtToken { token })
}

pub async fn parse_token(db: &mut PgConnection, token: &str) -> anyhow::Result<Option<User>> {
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(KEY),
        &Validation::new(Algorithm::HS512),
    )?;

    let uuid = Uuid::parse_str(&token_data.claims.subject).unwrap();

    Ok(services::user::get(db, uuid).await?)
}
