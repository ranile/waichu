use crate::components::UserAvatar;
use common::Message;
use yew::prelude::*;
use yew_functional::function_component;
use yew_md::Markdown;
use yew_state::SharedStateComponent;
use crate::utils::format_time;

#[derive(Clone, Properties, PartialEq)]
pub struct SingleMessageProp {
    pub message: Message,
}

#[function_component(SingleMessage)]
pub fn show_single_message(props: &SingleMessageProp) -> Html {
    let message = &props.message;

    let time = format_time(&props.message.created_at);
    html! {
        <article class="message-card">
            <SharedStateComponent<UserAvatar> user=&message.author />
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
    }
}
