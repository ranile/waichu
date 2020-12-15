use crate::{create_user, db};
use backend::auth::create_jwt;
use common::User;
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
            let token = create_jwt(&user).unwrap();

            let api = backend::routes::user::routes(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/users/{}", user.uuid))
                .header("Authorization", token.token)
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
async fn test_get_by_username() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let user = create_user(&mut conn, username, password).await;
            let token = create_jwt(&user).unwrap();

            let api = backend::routes::user::routes(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/users/by_username/{}", user.username))
                .header("Authorization", token.token)
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
async fn test_get_me() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let user = create_user(&mut conn, username, password).await;
            let token = create_jwt(&user).unwrap();

            let api = backend::routes::user::routes(pool);

            let resp = request()
                .method("GET")
                .path("/api/users/me")
                .header("Authorization", token.token)
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
