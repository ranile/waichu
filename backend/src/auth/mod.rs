mod models;

use crate::models::User;
use uuid::Uuid;
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm, decode, DecodingKey, Validation};
use jsonwebtoken::errors as jwt_errors;
use jwt_errors::Result as JwtResult;
use crate::auth::models::Claims;
use crate::services::user_service;
use crate::DbPool;
use sqlx::PgPool;
use warp::Filter;
use crate::routes::{Unauthorized, with_db};

static KEY: &[u8; 6] = b"secret";

pub fn create_jwt(user: &User) -> JwtResult<String> {
    let my_claims =
        models::Claims { sub: user.uuid.to_string(), company: "chatr".to_owned(), exp: 10000000000 };

    let mut header = Header::default();
    header.kid = Some("signing_key".to_owned());
    header.alg = Algorithm::HS512;

    let token = encode(&header, &my_claims, &EncodingKey::from_secret(KEY));
    println!("{:?}", token);
    token
}

// TODO Error handling
pub async fn parse_token(pool: &DbPool, token: &str) -> JwtResult<User> {
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(KEY),
        &Validation::new(Algorithm::HS512),
    );
    match token_data {
        Ok(token_data) => {
            println!("token data");
            let uuid = Uuid::parse_str(&token_data.claims.sub).unwrap();
            let user = user_service::get(&mut pool.acquire().await.unwrap(), uuid).await.unwrap();
            println!("user here");
            Ok(user)
        }
        Err(e) => Err(e)
    }
}

pub fn ensure_authorized(pool: PgPool) -> impl Filter<Extract=(User, ), Error=warp::Rejection> + Clone {
    warp::header::<String>("Authorization")
        .and(with_db(pool))
        .and_then(|token: String, db: PgPool| async move {
            let resp = parse_token(&db, &token).await;
            match resp {
                Ok(user) => Ok(user),
                Err(_) => Err(warp::reject::custom(Unauthorized))
            }
        })
}
