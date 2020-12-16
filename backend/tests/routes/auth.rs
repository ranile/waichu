use crate::{create_user, db};
use backend::auth::parse_token;
use common::payloads::{Credentials, JwtToken};
use warp::http::StatusCode;
use warp::test::request;

#[tokio::test]
async fn test_signin() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            create_user(&mut conn, username, password).await;

            let api = backend::api(pool);

            let resp = request()
                .method("POST")
                .path("/api/auth/signin")
                .json(&Credentials {
                    username: username.to_string(),
                    password: password.to_string(),
                })
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::OK);
            let jwt =
                serde_json::from_slice::<JwtToken>(resp.body()).expect("failed to parse response");
            let user = parse_token(&mut conn, &jwt.token)
                .await
                .expect("failed to parse token")
                .unwrap();

            assert_eq!(user.username, username);
        })
    })
    .await
}

#[tokio::test]
async fn test_signin_with_invalid_credentials() {
    db(|pool| {
        Box::pin(async {
            let username = "user";
            let password = "password";

            let api = backend::api(pool);
            let resp = request()
                .method("POST")
                .path("/api/auth/signin")
                .json(&Credentials {
                    username: username.to_string(),
                    password: password.to_string(),
                })
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        })
    })
    .await
}

#[tokio::test]
async fn test_signup() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let api = backend::api(pool);

            let resp = request()
                .method("POST")
                .path("/api/auth/signup")
                .json(&Credentials {
                    username: username.to_string(),
                    password: password.to_string(),
                })
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::CREATED);
            let jwt =
                serde_json::from_slice::<JwtToken>(resp.body()).expect("failed to parse response");
            let user = parse_token(&mut conn, &jwt.token)
                .await
                .expect("failed to parse token")
                .unwrap();

            assert_eq!(user.username, username);
        })
    })
    .await
}
