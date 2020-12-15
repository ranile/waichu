use crate::db;
use warp::test::request;
use warp::http::StatusCode;
use common::payloads::{Credentials, JwtToken};

#[tokio::test]
async fn test_signin() {
    db(|pool| Box::pin (async {
        let api = backend::auth::routes(pool);

        let resp = request()
            .method("POST")
            .path("/api/auth/signin")
            .json(&Credentials {
                username: "user5".to_string(),
                password: "pass".to_string()
            })
            .reply(&api)
            .await;

        // let json = serde_json::from_slice::<JwtToken>(resp.body()).unwrap();
        // assert_eq!(json.token.starts_with("e"), true);

        assert_eq!(resp.status(), StatusCode::OK);

    })).await
}
