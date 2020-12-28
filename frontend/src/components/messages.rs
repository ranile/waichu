use crate::components::SingleMessage;
use crate::services::room::fetch_room_messages;
use crate::utils::use_token;
use crate::websocket::{internal_events, InternalEventBus};
use common::{Message, Room};
use std::cell::Ref;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_functional::{function_component, use_effect, use_effect_with_deps, use_ref, use_state};

#[derive(Clone, Properties, PartialEq)]
pub struct MessagesProps {
    pub room: Room,
}

#[derive(Copy, Clone, Debug)]
enum LoadingState<E> {
    NotLoading,
    Loading,
    Loaded,
    Error(E),
}

#[function_component(RoomMessages)]
pub fn show_room_messages(props: &MessagesProps) -> Html {
    let token = use_token();

    let messages = use_ref(Vec::new);
    let (state, set_state) = use_state(|| LoadingState::NotLoading);

    {
        let set_state = set_state.clone();
        let messages = messages.clone();

        use_effect_with_deps(
            move |room_id| {
                set_state(LoadingState::Loading);

                let room_id = *room_id;
                spawn_local(async move {
                    match fetch_room_messages(&*token, room_id).await {
                        Ok(rec_messages) => {
                            let mut messages = messages.borrow_mut();
                            messages.extend(rec_messages);
                            drop(messages);
                            set_state(LoadingState::Loaded)
                        }
                        Err(e) => set_state(LoadingState::Error(e)),
                    }
                });

                || ()
            },
            props.room.uuid,
        );
    }

    {
        let set_state = Rc::clone(&set_state);
        let messages = Rc::clone(&messages);
        let current_uuid = props.room.uuid;

        use_effect(move || {
            let producer = InternalEventBus::bridge(Callback::from(move |msg| match msg {
                internal_events::Response::NewMessage(msg) => {
                    if msg.room.uuid == current_uuid {
                        weblog::console_log!("logging new message", msg.uuid.to_string());
                        messages.borrow_mut().insert(0, (*msg).clone());
                        set_state(LoadingState::Loaded)
                    }
                }
            }));

            || drop(producer)
        })
    };

    let list = match &*state {
        LoadingState::NotLoading => html!("not loading"),
        LoadingState::Loading => {
            messages.borrow_mut().clear();
            html!("loading")
        }
        LoadingState::Loaded => display_messages(messages.borrow()),
        LoadingState::Error(e) => html!(e.to_string()),
    };

    html! {
        <section class="messages-container">
            { list }
        </section>
    }
}

fn display_messages(messages: Ref<Vec<Message>>) -> Html {
    let messages = messages
        .iter()
        .map(|message| html! { <SingleMessage key=message.uuid.to_string() message=message /> });

    html! { for messages }
}
