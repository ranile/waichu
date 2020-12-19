use crate::services::room::send_message;
use crate::utils::use_token;
use common::payloads::CreateMessage as CreateMessagePayload;
use common::Room;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_functional::{function_component, use_state};
use yew_material::{MatFormfield, MatIconButton, MatTextArea};

#[derive(Clone, Properties, PartialEq)]
pub struct CreateMessageProps {
    pub room: Room,
}

#[function_component(CreateMessage)]
pub fn create_message(props: &CreateMessageProps) -> Html {
    let (message, set_message) = use_state(|| "".to_string());
    let token = use_token();
    let (error, set_error) = use_state(|| None);

    let onclick = {
        let message = Rc::clone(&message);
        let set_message = Rc::clone(&set_message);
        let room_id = props.room.uuid;

        Callback::from(move |_| {
            let token = Rc::clone(&token);
            let message = Rc::clone(&message);
            let set_message = Rc::clone(&set_message);
            let set_error = Rc::clone(&set_error);

            spawn_local(async move {
                let result = send_message(
                    &*token,
                    room_id,
                    &CreateMessagePayload {
                        content: (*message).clone(),
                    },
                )
                .await;
                match result {
                    Ok(_) => set_message(String::new()),
                    Err(e) => set_error(Some(e)),
                }
            });
        })
    };

    const TEXT_AREA_SELECTOR: &str =
        ".new-message-form-container > mwc-formfield:nth-child(1) > mwc-textarea";

    let oninput = Callback::from(move |event: InputData| {
        let value = event.value;
        let line_break_count = value.split('\n').count();

        // min-height + lines x line-height + padding (0) + border (0)
        let new_height = 40 + line_break_count * 20;

        // maybe find a way to handle this without making DOM API calls
        yew::utils::document()
            .query_selector(TEXT_AREA_SELECTOR)
            .unwrap()
            .unwrap()
            .set_attribute("style", &format!("height: {}px;", new_height))
            .unwrap();

        set_message(value);
    });

    let error_node = if let Some(e) = &*error {
        html!(e.to_string())
    } else {
        html!()
    };

    html! {<>
        <article class="new-message-form-container">
            <MatFormfield>
                <MatTextArea
                    outlined=true
                    value=&*message
                    label="Message..."
                    oninput=oninput
                 />
            </MatFormfield>
            { error_node }
            <span onclick=onclick>
                <MatIconButton icon="send"/>
            </span>
        </article>
    </>}
}
