use crate::{create_authenticated_user, create_room, create_user, db, join_user};
use common::payloads::{CreateRoom, JoinMembers};
use common::{Room, RoomMember};
use sqlx::types::Uuid;
use warp::http::StatusCode;
use warp::test::request;

#[tokio::test]
async fn test_get_room() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (_, token) = create_authenticated_user(&mut conn, username, password).await;
            let room = create_room(&mut conn, "room_name").await;

            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/rooms/{}/", room.uuid))
                .header("Authorization", token)
                .reply(&api)
                .await;

            let body =
                serde_json::from_slice::<Room>(resp.body()).expect("failed to parse response");

            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(body.uuid, room.uuid);
            assert_eq!(body.name, room.name);
            assert_eq!(body.created_at, room.created_at);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_non_existent_room_should_404() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (_, token) = create_authenticated_user(&mut conn, username, password).await;
            let room_id = Uuid::new_v4();

            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/rooms/{}/", room_id))
                .header("Authorization", token)
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        })
    })
    .await
}

#[tokio::test]
async fn test_create_room() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (_, token) = create_authenticated_user(&mut conn, username, password).await;

            let room_name = "room_name";

            let api = backend::api(pool);

            let resp = request()
                .method("POST")
                .path("/api/rooms/")
                .header("Authorization", token)
                .json(&CreateRoom {
                    name: room_name.to_string(),
                })
                .reply(&api)
                .await;

            let room =
                serde_json::from_slice::<Room>(resp.body()).expect("failed to parse response");

            assert_eq!(resp.status(), StatusCode::CREATED);
            assert_eq!(room.name, room_name);
            assert_eq!(room.icon, None); // there shouldn't be any icon at first
        })
    })
    .await
}

#[tokio::test]
async fn test_join_room() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (user, token) = create_authenticated_user(&mut conn, username, password).await;
            let user_to_join = create_user(&mut conn, "username_to_join", "password_to_join").await;
            let room = create_room(&mut conn, "room_name").await;
            join_user(&mut conn, &user, &room, true).await;
            let api = backend::api(pool);

            let resp = request()
                .method("POST")
                .path(&format!("/api/rooms/{}/join", room.uuid))
                .header("Authorization", token)
                .json(&JoinMembers {
                    member: user_to_join.uuid,
                    with_elevated_permissions: true,
                })
                .reply(&api)
                .await;

            let body = serde_json::from_slice::<RoomMember>(resp.body())
                .expect("failed to parse response");

            assert_eq!(resp.status(), StatusCode::CREATED);
            assert_eq!(body.room.uuid, room.uuid);
            assert_eq!(body.user.uuid, user_to_join.uuid);
            assert_eq!(body.has_elevated_permissions, true);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_room_members() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (user, token) = create_authenticated_user(&mut conn, username, password).await;
            let room = create_room(&mut conn, "room_name").await;
            join_user(&mut conn, &user, &room, true).await;

            let mut joined_users = Vec::with_capacity(10);
            for i in 0..10 {
                let user = create_user(
                    &mut conn,
                    &format!("username_to_join_{}", i),
                    "password_to_join",
                )
                .await;
                let joined = join_user(&mut conn, &user, &room, false).await;
                joined_users.push(joined);
            }

            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/rooms/{}/members", room.uuid))
                .header("Authorization", token)
                .reply(&api)
                .await;

            let body = serde_json::from_slice::<Vec<RoomMember>>(resp.body())
                .expect("failed to parse response");

            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(body.len(), joined_users.len() + 1); // + 1 being for the currently authenticated user
        })
    })
    .await
}

#[tokio::test]
async fn test_no_perms_403() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");
            let username = "user";
            let password = "password";

            let (_, token) = create_authenticated_user(&mut conn, username, password).await;
            let room = create_room(&mut conn, "room_name").await;

            let api = backend::api(pool);

            let resp = request()
                .method("GET")
                .path(&format!("/api/rooms/{}/members", room.uuid))
                .header("Authorization", token)
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        })
    })
    .await
}
