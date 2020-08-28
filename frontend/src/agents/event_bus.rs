use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use yew::worker::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use log::*;
use yew::Dispatched;
use serde_json::Value;
use crate::app::APP_STATE;
use crate::models::{Room, User};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    EventBusMsg(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebsocketMessage {
    pub op: Op,
    pub data: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Op {
    Authenticated = 2,
    MessageCreate = 3,
    RoomCreate = 4,
}

impl From<u64> for Op {
    fn from(code: u64) -> Self {
        match code {
            2 => Self::Authenticated,
            3 => Self::MessageCreate,
            4 => Self::RoomCreate,
            _ => { panic!("Invalid value") }
        }
    }
}

#[derive(Deserialize, Debug)]
struct Authenticated {
    rooms: Vec<Room>,
    me: User,
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = String;
    type Output = WebsocketMessage;

    fn create(link: AgentLink<Self>) -> Self {
        EventBus {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    //noinspection DuplicatedCode
    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        let parsed = serde_json::from_str::<HashMap<String, Value>>(&msg).unwrap();

        match Op::from(parsed["op"].as_u64().unwrap()) {
            Op::Authenticated => {
                let data = serde_json::from_value::<Authenticated>(parsed["data"].clone()).unwrap();

                APP_STATE.with(move |f| {
                    let mut state = f.borrow_mut();

                    data.rooms.into_iter().for_each(|r| { state.rooms.insert(r); });
                });

                self.send_to_all(WebsocketMessage {
                    op: Op::Authenticated,
                    data: parsed["data"].clone()
                });
            }
            Op::MessageCreate => {
                self.send_to_all(WebsocketMessage {
                    op: Op::MessageCreate,
                    data: parsed["data"].clone()
                });
            }
            Op::RoomCreate => {
                self.send_to_all(WebsocketMessage {
                    op: Op::RoomCreate,
                    data: parsed["data"].clone()
                });
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

impl EventBus {
    fn send_to_all(&self, message: WebsocketMessage) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, message.clone());
        }
    }
}

#[derive(Serialize)]
struct AuthenticatePayload {
    op: u32,
    token: String,
}

impl AuthenticatePayload {
    fn new(token: String) -> Self {
        Self { op: 1, token }
    }

    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

pub fn start_websocket(token: String) -> Result<(), JsValue> {
    let mut event_bus = EventBus::dispatcher();

    // Connect to websocket server
    let ws = WebSocket::new("ws://localhost:3030/chat").unwrap();


    // create callback
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            info!("message event, received Text: {:?}", txt);
            event_bus.send(String::from(txt))
        }
    }) as Box<dyn FnMut(MessageEvent)>);

    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    // Error callback
    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        info!("error event: {:?}", e);
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let auth_payload = AuthenticatePayload::new(token).to_string();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        info!("socket opened");
        match cloned_ws.send_with_str(&auth_payload) {
            Ok(_) => { info!("auth msg successfully sent") }
            Err(err) => info!("error sending message: {:?}", err),
        }
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}
