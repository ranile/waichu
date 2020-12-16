use backend::services::{message as message_service, room as room_service};
use common::{Message, Room, RoomMember, User};
use sqlx::PgConnection;

pub async fn create_room(conn: &mut PgConnection, name: &str) -> Room {
    let room = Room::new(name);
    room_service::create(conn, room)
        .await
        .expect("failed to create room")
}

pub async fn join_user(
    conn: &mut PgConnection,
    user: &User,
    room: &Room,
    has_elevated_perms: bool,
) -> RoomMember {
    room_service::join(conn, room, user, has_elevated_perms)
        .await
        .expect("failed to create room")
}

pub async fn create_room_with_user(
    conn: &mut PgConnection,
    name: &str,
    user: &User,
    has_elevated_perms: bool,
) -> (Room, RoomMember) {
    let room = create_room(conn, name).await;
    let member = join_user(conn, user, &room, has_elevated_perms).await;

    (room, member)
}

pub async fn send_message(
    conn: &mut PgConnection,
    content: &str,
    user: &User,
    room: &Room,
) -> Message {
    let message = Message::new(user.clone(), room.clone(), content.to_string());
    message_service::create(conn, message)
        .await
        .expect("failed to create message")
}
