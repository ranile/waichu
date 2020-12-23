use crate::components::{UserAvatar, UserProfileDialog};
use crate::utils::format_time;
use common::{Message, MessageType};
use yew::prelude::*;
use yew_functional::{function_component, use_state};
use yew_md::Markdown;
use yew_state::SharedStateComponent;

#[derive(Clone, Properties, PartialEq)]
pub struct SingleMessageProp {
    pub message: Message,
}

#[function_component(SingleMessage)]
pub fn show_single_message(props: &SingleMessageProp) -> Html {
    let message = &props.message;
    let (dialog_open, set_dialog_open) = use_state(|| false);

    let join_click = {
        let set_dialog_open = set_dialog_open.clone();
        Callback::from(move |_| set_dialog_open(true))
    };

    let on_dialog_closed = Callback::from(move |_| set_dialog_open(false));

    let time = format_time(&props.message.created_at);
    match message.type_ {
        MessageType::Default => html! {
            <article class="message-card" data_type="default">
                <UserAvatar user=&message.author />
                <section class="content-container">
                    <section>
                        <span class="author">{ &message.author.username }</span>
                        <span class="timestamp">{ time }</span>
                    </section>
                    <span class="content">
                        <Markdown content=&message.content />
                    </span>
                </section>
            </article>
        },
        MessageType::RoomJoin => html! {
            <article class="message-card" data_type="join" onclick=join_click>
                <UserAvatar user=&message.author show_details_on_click=false />
                <span>{ &message.author.username }{ " just joined" }</span>
                <span class="timestamp">{ time }</span>
                <SharedStateComponent<UserProfileDialog> user=&message.author open=*dialog_open onclosed=on_dialog_closed />
            </article>
        },
        MessageType::RoomLeave => html! {},
    }
}
