use crate::components::SingleMessage;
use crate::services::room::fetch_room_messages;
use crate::utils::use_token;
use crate::websocket::{internal_events, InternalEventBus};
use common::{Room, Message};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use weblog::console_log;
use yew::prelude::*;
use yew_functional::{function_component, use_effect, use_effect_with_deps, use_state, use_ref};
use std::cell::RefMut;

#[derive(Clone, Properties, PartialEq)]
pub struct MessagesProps {
    pub room: Room,
}

#[derive(Copy, Clone, Debug)]
enum LoadingState<T, E, C> {
    NotLoading,
    Loading,
    Loaded(T),
    Error(E),
    NewContent(C)
}

#[function_component(RoomMessages)]
pub fn show_room_messages(props: &MessagesProps) -> Html {
    let token = use_token();

    let mut messages = use_ref(Vec::new);
    let (state, set_state) = use_state(|| LoadingState::NotLoading);

    {
        let set_state = set_state.clone();

        use_effect_with_deps(
            move |room_id| {
                set_state(LoadingState::Loading);

                let room_id = *room_id;
                spawn_local(async move {
                    match fetch_room_messages(&*token, room_id).await {
                        Ok(mut messages) => set_state(LoadingState::Loaded(messages)),
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
        let current_uuid = props.room.uuid;

        use_effect(move || {
            let producer = InternalEventBus::bridge(Callback::from(move |msg| match msg {
                internal_events::Response::NewMessage(msg) => {
                    if msg.room.uuid == current_uuid {
                        weblog::console_log!("logging new message", msg.uuid.to_string());
                        set_state(LoadingState::NewContent(msg))
                    }
                }
            }));

            || drop(producer)
        })
    };

    let list = match &*state {
        LoadingState::NotLoading => html!("not loading"),
        LoadingState::Loading => html!("loading"),
        LoadingState::Loaded(data) => {
            let mut messages = messages.borrow_mut();
            data.into_iter().for_each(|it| messages.push(it.clone()));

            display_messages(&mut messages)
        },
        LoadingState::Error(e) => html!(e.to_string()),
        LoadingState::NewContent(message) => {
            let mut messages = messages.borrow_mut();
            messages.push((**message).clone());

            display_messages(&mut messages)
        }
    };



    html! {
        <section class="messages-container">
            { list }
        </section>
    }
}

fn display_messages(messages: &mut RefMut<Vec<Message>>) -> Html {
    messages.sort_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap());
    let messages = messages.iter().map(|message| html! {<SingleMessage message=message />});

    html! { for messages }
}
