use serde::Serialize;
use crate::models::RoomMember;
use uuid::Uuid;
use crate::gateway::{USERS, OutgoingMessage};
use async_trait::async_trait;

#[async_trait]
pub trait WsSend: Serialize {
    async fn send(&self, op: u32, send_to: sqlx::Result<Vec<RoomMember>>) {
        println!("in send, send_to = {:#?}", send_to);
        let send_to = match send_to {
            Ok(d) => d,
            Err(_) => { return; }
        };

        let send_to: Vec<Uuid> = send_to.iter().map(|it| it.uuid).collect();

        let users = USERS.read().await;
        let users = users.values();

        println!("User count - trait impl {}", users.len());

        // Unwrapping here because if it fails, I fucked something up
        let send_to = users.into_iter().filter(|c| send_to.contains(&c.user.as_ref().unwrap().uuid));
        let outgoing_message = OutgoingMessage {
            op,
            data: self
        };
        for con in send_to {
            // Message("Sending more data from trait default impl... will it be sent?".to_owned())
            con.tx.send(Ok(outgoing_message.to_message()));
        }
    }
}
