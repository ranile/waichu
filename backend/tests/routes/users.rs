use crate::{create_authenticated_user, create_user, db};
use common::User;
use sqlx::types::Uuid;
use warp::http::StatusCode;
use warp::test::request;

#[tokio::test]
async fn test_get_by_uuid() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let user = create_user(&mut conn, username, password).await;

            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/users/{}", user.uuid))
                .reply(&api)
                .await;

            let parsed_json =
                serde_json::from_slice::<User>(resp.body()).expect("failed to parse response");
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(parsed_json.uuid, user.uuid);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_by_invalid_uuid() {
    db(|pool| {
        Box::pin(async {
            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/users/{}", Uuid::new_v4()))
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_by_username() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (user, token) = create_authenticated_user(&mut conn, username, password).await;

            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/users/by_username/{}", user.username))
                .header("Authorization", token)
                .reply(&api)
                .await;

            let parsed_json =
                serde_json::from_slice::<User>(resp.body()).expect("failed to parse response");
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(parsed_json.username, user.username);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_by_invalid_username() {
    db(|pool| {
        Box::pin(async {
            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/users/by_username/{}", "username"))
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_me() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (user, token) = create_authenticated_user(&mut conn, username, password).await;

            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path("/api/users/me")
                .header("Authorization", token)
                .reply(&api)
                .await;

            let parsed_json =
                serde_json::from_slice::<User>(resp.body()).expect("failed to parse response");
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(parsed_json.username, user.username);
            assert_eq!(parsed_json.uuid, user.uuid);
            assert_eq!(parsed_json.created_at, user.created_at);
            assert_eq!(parsed_json.password, "");
        })
    })
    .await
}

#[tokio::test]
async fn test_get_me_without_token() {
    db(|pool| {
        Box::pin(async {
            let api = backend::api(pool);
            let resp = request()
                .method("GET")
                .path("/api/users/me")
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_me_with_bad_token() {
    db(|pool| {
        Box::pin(async {
            let api = backend::api(pool);
            let resp = request()
                .method("GET")
                .path("/api/users/me")
                .header("Authorization", "bad token")
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        })
    })
    .await
}
