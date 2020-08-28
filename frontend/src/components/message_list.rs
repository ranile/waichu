use yew::prelude::*;
use uuid::Uuid;
use crate::utils::{send_future};
use crate::models::{Message};
use log::*;
use crate::agents::event_bus::{EventBus, WebsocketMessage, Op};
use crate::services::{message_service};
use crate::services::message_service::FetchedMessage;

pub struct MessageListComponent {
    link: ComponentLink<Self>,
    room_id: Uuid,
    messages: Vec<Message>,
    producer: Box<dyn Bridge<EventBus>>,
}

pub enum Msg {
    MessagesFetched(Vec<Message>),
    NewMessage(WebsocketMessage),
    MessageDataProcessed(Message),
    Ignore,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub room_id: Uuid
}

impl Component for MessageListComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let room_id = props.room_id;

        info!("rendering for {} 1", room_id);

        let callback = link.callback(Msg::NewMessage);
        let producer = EventBus::bridge(callback);

        populate_data(room_id, link.clone());


        MessageListComponent {
            link,
            room_id,
            messages: vec![],
            producer,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::MessagesFetched(messages) => {
                self.messages = messages;
                self.messages.sort_by_key(|m| m.created_at);
                true
            }
            Msg::NewMessage(msg) => {
                if msg.op != Op::MessageCreate {
                    return false
                }
                info!("received message {:?}", msg.data);

                let fetched_message = serde_json::from_value::<FetchedMessage>(msg.data).unwrap();

                send_future(self.link.clone(), async move {
                    let message = fetched_message.to_message().await.unwrap();
                    Msg::MessageDataProcessed(message)
                });

                false
            }
            Msg::MessageDataProcessed(message) => {
                self.messages.push(message);
                let mapped: Vec<&String> = self.messages.iter().map(|it| &it.content).collect();
                info!("messages 1 {:?}", mapped);
                true
            }
            _ => { false }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        populate_data(props.room_id, self.link.clone());
        true
    }

    fn view(&self) -> Html {
        info!("rendering now");

        let mapped: Vec<&String> = self.messages.iter().map(|it| &it.content).collect();
        info!("messages 2 {:?}", mapped);


        let render_message = |author: String, content: String| {
            html! {
                <div class="single-message">
                    <span style="font-weight: bold">{ author }</span>
                    <p>{ content }</p>
                </div>
            }
        };

        html! {
            <>
                { for self.messages.iter().map(|m| render_message(m.author.username.clone(), m.content.clone())) }
            </>
        }
    }
}


pub fn populate_data(room_id: Uuid, link: ComponentLink<MessageListComponent>) {
    send_future(link.clone(), async move {
        let new_data = message_service::get_room_messages(room_id).await.unwrap();

        return Msg::MessagesFetched(new_data);
    });
}
