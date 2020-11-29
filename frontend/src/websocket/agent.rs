use common::websocket::{AuthenticatePayload, MessagePayload, OpCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::rc::Rc;
use yew::format::Text;
use yew::services::websocket::{WebSocketStatus, WebSocketTask};
use yew::services::WebSocketService;
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Connect(String),
    Authenticate(String),
    Disconnect,
}

#[derive(Clone, Debug)]
pub enum Response {
    Connected,
    Closed,
    Error(Rc<anyhow::Error>),
    Message(Rc<MessagePayload<Value>>),
}

pub enum Message {
    Text(Text),
    StatusNotification(WebSocketStatus),
}

pub struct Connection {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    task: Option<WebSocketTask>,
}

impl Agent for Connection {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            task: None,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::Text(msg) => {
                let msg = match parse_message(msg) {
                    Ok(msg) => msg,
                    Err(e) => {
                        let error = Rc::new(e);
                        return self.send_to_all_subs(|| Response::Error(error.clone()));
                    }
                };

                let msg = Rc::new(msg);

                self.send_to_all_subs(move || Response::Message(msg.clone()));
            }
            Message::StatusNotification(status) => match status {
                WebSocketStatus::Opened => self.send_to_all_subs(|| Response::Connected),
                WebSocketStatus::Error => self.send_to_all_subs(|| {
                    Response::Error(Rc::new(anyhow::anyhow!("Websocket error")))
                }),
                WebSocketStatus::Closed => self.send_to_all_subs(|| Response::Closed),
            },
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::Connect(url) => {
                let task = WebSocketService::connect_text(
                    &url,
                    self.link.callback(Message::Text),
                    self.link.callback(Message::StatusNotification),
                )
                .map_err(anyhow::Error::from);

                let task = match task {
                    Ok(task) => task,
                    Err(e) => {
                        let error = Rc::new(e);
                        self.send_to_all_subs(|| Response::Error(error.clone()));
                        return;
                    }
                };

                self.task = Some(task)
            }
            Request::Authenticate(token) => {
                weblog::console_log!("sending auth");
                self.send_to_ws(&MessagePayload {
                    op: OpCode::Authenticate,
                    data: AuthenticatePayload { token },
                });
            }
            Request::Disconnect => {
                // TODO yew limitation
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

impl Connection {
    fn send_to_all_subs(&self, message: impl Fn() -> Response) {
        for id in &self.subscribers {
            self.link.respond(*id, message());
        }
    }

    fn send_to_ws<T>(&mut self, message: &MessagePayload<T>)
    where
        T: for<'de> Deserialize<'de> + Serialize,
    {
        let payload = serde_json::to_string(&message).unwrap();
        self.task
            .as_mut()
            .expect("no connection to websocket")
            .send(Ok(payload))
    }
}

fn parse_message(msg: Text) -> anyhow::Result<MessagePayload<Value>> {
    let msg = msg?;
    let payload = serde_json::from_str::<MessagePayload<Value>>(&msg)?;
    Ok(payload)
}
