use common::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::rc::Rc;
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    NewMessage(Message),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    NewMessage(Rc<Message>),
}

pub struct InternalEventBus {
    link: AgentLink<InternalEventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for InternalEventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::NewMessage(message) => {
                let message = Rc::new(message);
                for sub in self.subscribers.iter() {
                    self.link
                        .respond(*sub, Response::NewMessage(message.clone()));
                }
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
