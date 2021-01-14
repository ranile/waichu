use crate::{Room, User};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "T: for<'a> Deserialize<'a>"))]
pub struct MessagePayload<T>
where
    T: for<'a> Deserialize<'a>,
{
    pub op: OpCode,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticatePayload {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticatedPayload {
    pub me: User,
    pub rooms: Vec<Room>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum OpCode {
    Authenticate,
    Authenticated,
    BadPayload,
    InvalidOp,
    ServerError,
    MessageCreate,
    RoomCreate,
    RoomUpdate,
    RoomJoin,
    UserUpdate,
}

impl From<u32> for OpCode {
    fn from(value: u32) -> Self {
        u32_to_opcode(value)
    }
}

fn u32_to_opcode(value: u32) -> OpCode {
    match value {
        // server side => receive only for client
        0 => OpCode::Authenticated,
        1 => OpCode::MessageCreate,
        2 => OpCode::RoomCreate,
        3 => OpCode::RoomUpdate,
        4 => OpCode::RoomJoin,
        5 => OpCode::UserUpdate,

        // client side => send only for client
        100 => OpCode::Authenticate,

        // invalid
        _ => OpCode::InvalidOp,
    }
}
