use crate::components::SingleMessage;
use crate::services::room::fetch_room_messages;
use crate::websocket::{internal_events, InternalEventBus};
use common::Room;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use weblog::console_log;
use yew::prelude::*;
use yew_functional::{function_component, use_effect, use_effect_with_deps, use_state};
use yew_material::MatList;
use crate::utils::use_token;

#[derive(Clone, Properties, PartialEq)]
pub struct MessagesProps {
    pub room: Room,
}

#[function_component(RoomMessages)]
pub fn show_room_messages(props: &MessagesProps) -> Html {
    let token = use_token();

    let (messages, set_messages) = use_state(Vec::new);
    let (error, set_error) = use_state(|| None);

    {
        let set_messages = set_messages.clone();

        use_effect_with_deps(
            move |room_id| {
                let room_id = *room_id;
                spawn_local(async move {
                    match fetch_room_messages(&*token, room_id).await {
                        Ok(mut messages) => {
                            messages
                                .sort_by(|a, b| a.created_at.partial_cmp(&b.created_at).unwrap());
                            set_messages(messages);
                        }
                        Err(e) => set_error(Some(e)),
                    }
                });

                || ()
            },
            props.room.uuid,
        );
    }

    console_log!(JsValue::from_serde(&*messages).unwrap());

    {
        let messages = Rc::clone(&messages);
        let current_uuid = props.room.uuid;

        use_effect(move || {
            let producer = InternalEventBus::bridge(Callback::from(move |msg| match msg {
                internal_events::Response::NewMessage(msg) => {
                    if msg.room.uuid == current_uuid {
                        weblog::console_log!("logging new message", msg.uuid.to_string());
                        let mut messages = (*messages).clone();
                        messages.push((*msg).clone());
                        messages.sort_by(|a, b| a.created_at.partial_cmp(&b.created_at).unwrap());
                        set_messages(messages)
                    }
                }
            }));

            || drop(producer)
        })
    };

    let messages = messages.iter().map(|message| {
        html! { <>
        // <MatListItem twoline=true graphic=GraphicType::Avatar key=message.uuid.to_string()>
        //     <mwc-icon slot="graphic">{"folder"}</mwc-icon>
        //     <span>{ &message.author.username }</span>
        //     <span slot="secondary">{ &message.content }</span>
        // </MatListItem>
        <SingleMessage message=message />
        </>}
    });

    let list = match &*error {
        Some(e) => html!(e.to_string()),
        None => html! {
            <MatList noninteractive=true>
                { for messages }
            </MatList>
        },
    };

    html! {
        <section class="messages-container">
            { list }
        </section>
    }
}
