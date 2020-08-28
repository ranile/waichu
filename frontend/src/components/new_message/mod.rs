use yew::prelude::*;
use uuid::Uuid;
use crate::utils::send_future;
use crate::services::message_service;
use log::*;
use yew::events::KeyboardEvent;

pub struct NewMessageComponent {
    link: ComponentLink<Self>,
    room_id: Uuid,
    message_content: String,
}

pub enum Msg {
    SendMessage,
    SentSuccessfully,
    MessageInput(String),
    Ignore,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub room_id: Uuid,
}

impl Component for NewMessageComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            room_id: props.room_id,
            message_content: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::MessageInput(data) => {
                self.message_content = data;
                false
            }
            Msg::SendMessage => {
                info!("Sending {}", self.message_content);
                let room_id = self.room_id.clone();
                let content = self.message_content.clone();
                send_future(self.link.clone(), async move {
                    let m = message_service::send_message(room_id, content).await.unwrap();
                    info!("sent message {:?}", m);
                    Msg::SentSuccessfully
                });
                false
            }
            Msg::SentSuccessfully => {
                self.message_content = "".to_string();
                true
            }
            _ => { false }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let on_input = self.link.callback(|e: InputData| { Msg::MessageInput(e.value) });
        let on_keypress = self.link.callback(|e: KeyboardEvent| -> Msg {
            if e.which() == 13 {
                e.prevent_default();
                Msg::SendMessage
            } else { Msg::Ignore }
        });
        html! {
<section class="message-input-container">
    <textarea
            rows="1"
            placeholder="Message... Press enter to send"
            class="message-input"
            value=&self.message_content
            oninput=on_input
            onkeypress=on_keypress>
    </textarea>
</section>
        }
        // yew::include_html!("frontend/src/components/new_message/message_create.html")
    }
}
