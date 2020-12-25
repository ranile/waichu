use crate::{create_authenticated_user, create_room, create_room_with_user, db, send_message};
use common::payloads::CreateMessage;
use common::{Message, MessageType};
use warp::http::StatusCode;
use warp::test::request;

#[tokio::test]
async fn test_get_messages() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");

            let (user, token) = create_authenticated_user(&mut conn, "user", "password").await;
            let (room, _) = create_room_with_user(&mut conn, "room_name", &user, false).await;

            let mut messages = Vec::with_capacity(10);
            for i in 0..10 {
                let content = format!("message_content {}", i);
                let message = send_message(&mut conn, &content, &user, &room).await;
                messages.push(message);
            }

            let api = backend::api(pool);
            let resp = request()
                .method("GET")
                .path(&format!("/api/rooms/{}/messages", room.uuid))
                .header("Authorization", token)
                .reply(&api)
                .await;

            let parsed_json = serde_json::from_slice::<Vec<Message>>(resp.body())
                .expect("failed to parse response");

            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(parsed_json.len(), messages.len() + 1); // + 1 for `room_join` message
        })
    })
    .await
}

// This test ensures that the join room message is sent and
// there is only 1 message at the time of room creation
#[tokio::test]
async fn test_get_messages_with_no_messages_sent() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");

            let (user, token) = create_authenticated_user(&mut conn, "user", "password").await;
            let (room, _) = create_room_with_user(&mut conn, "room_name", &user, false).await;

            let api = backend::api(pool);
            let resp = request()
                .method("GET")
                .path(&format!("/api/rooms/{}/messages", room.uuid))
                .header("Authorization", token)
                .reply(&api)
                .await;

            let parsed_json = serde_json::from_slice::<Vec<Message>>(resp.body())
                .expect("failed to parse response");

            // there should only be one message, the `room_join` one
            // one message returns `OK`
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(parsed_json.len(), 1);

            let first = parsed_json.first().unwrap();

            assert_eq!(first.type_, MessageType::RoomJoin);
            assert_eq!(first.author, user);
        })
    })
    .await
}

#[tokio::test]
async fn test_get_messages_without_being_in_403() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");

            let (_, token) = create_authenticated_user(&mut conn, "user", "password").await;
            let room = create_room(&mut conn, "room_name").await;

            let api = backend::api(pool);
            let resp = request()
                .method("GET")
                .path(&format!("/api/rooms/{}/messages", room.uuid))
                .header("Authorization", token)
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        })
    })
    .await
}

#[tokio::test]
async fn test_create_messages() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");

            let (user, token) = create_authenticated_user(&mut conn, "user", "password").await;
            let (room, _) = create_room_with_user(&mut conn, "room_name", &user, false).await;
            let message_content = "message_content".to_string();

            let api = backend::api(pool);
            let resp = request()
                .method("POST")
                .path(&format!("/api/rooms/{}/messages", room.uuid))
                .header("Authorization", token)
                .json(&CreateMessage {
                    content: message_content.clone(),
                })
                .reply(&api)
                .await;

            let parsed_json =
                serde_json::from_slice::<Message>(resp.body()).expect("failed to parse response");

            assert_eq!(resp.status(), StatusCode::CREATED);
            assert_eq!(parsed_json.content, message_content);
            assert_eq!(parsed_json.author, user);
            assert_eq!(parsed_json.room, room);
        })
    })
    .await
}

#[tokio::test]
async fn test_create_blank_message_fails() {
    db(|pool| {
        Box::pin(async {
            let mut conn = pool.acquire().await.expect("can't acquire pool");

            let (user, token) = create_authenticated_user(&mut conn, "user", "password").await;
            let (room, _) = create_room_with_user(&mut conn, "room_name", &user, false).await;
            let message_content = "".to_string();

            let api = backend::api(pool);
            let resp = request()
                .method("POST")
                .path(&format!("/api/rooms/{}/messages", room.uuid))
                .header("Authorization", token)
                .json(&CreateMessage {
                    content: message_content.clone(),
                })
                .reply(&api)
                .await;

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        })
    })
    .await
}
