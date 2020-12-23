use crate::{Room, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use serde::export::TryFrom;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub uuid: Uuid,
    pub author: User,
    pub room: Room,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[serde(with = "message_type_serializer_deserializer")]
    #[serde(rename = "type")]
    pub type_: MessageType,
}

impl Message {
    pub fn new(author: User, room: Room, content: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            author,
            content,
            room,
            created_at: Utc::now(),
            type_: MessageType::Default,
        }
    }

    pub fn new_with_type(author: User, room: Room, content: String, type_: MessageType) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            author,
            content,
            room,
            created_at: Utc::now(),
            type_,
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::Type))]
#[cfg_attr(not(target_arch = "wasm32"), sqlx(rename = "message_type"))]
#[cfg_attr(not(target_arch = "wasm32"), sqlx(rename_all = "lowercase"))]
pub enum MessageType {
    Default,
    RoomJoin,
    RoomLeave,
}

pub struct ParseMessageTypeError(String);

impl FromStr for MessageType {
    type Err = ParseMessageTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(MessageType::Default),
            "room_join" => Ok(MessageType::RoomJoin),
            "room_leave" => Ok(MessageType::RoomLeave),
            _ => Err(ParseMessageTypeError(s.to_string())),
        }
    }
}

impl TryFrom<&str> for MessageType {
    type Error = ParseMessageTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        MessageType::from_str(value)
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MessageType::Default => "DEFAULT",
            MessageType::RoomJoin => "ROOM_JOIN",
            MessageType::RoomLeave => "ROOM_LEAVE",
        };

        write!(f, "{}", s)
    }
}

mod message_type_serializer_deserializer {
    use super::MessageType;
    use serde::export::Formatter;
    use serde::{de, Deserializer, Serializer};
    use std::fmt;
    use std::str::FromStr;

    struct MessageTypeVisitor;

    impl<'de> de::Visitor<'de> for MessageTypeVisitor {
        type Value = MessageType;

        fn expecting(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "a valid message type")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            MessageType::from_str(v).map_err(|_e| de::Error::custom("invalid message type"))
        }
    }

    pub fn serialize<S>(input: &MessageType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(input.to_string().as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<MessageType, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(MessageTypeVisitor)
    }
}
