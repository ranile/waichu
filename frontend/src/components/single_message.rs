use crate::components::UserAvatar;
use common::Message;
use yew::prelude::*;
use yew_functional::function_component;
use yew_md::Markdown;
use yew_state::SharedStateComponent;

#[derive(Clone, Properties, PartialEq)]
pub struct SingleMessageProp {
    pub message: Message,
}

#[function_component(SingleMessage)]
pub fn show_single_message(props: &SingleMessageProp) -> Html {
    let message = &props.message;
    html! {
        // <article class="message-card">
        // <MatListItem twoline=true graphic=GraphicType::Control key=message.uuid.to_string()>
        //     <mwc-icon slot="graphic">
        //         <SharedStateComponent<UserAvatar> user=&message.author />
        //     </mwc-icon>
        //     <span>{ &message.author.username }</span>
        //     <span slot="secondary">
        //         <Markdown content=&message.content />
        //     </span>
        // </MatListItem>
        // </article>
        <article class="message-card">
            <SharedStateComponent<UserAvatar> user=&message.author />
            <div class="content-container">
                <span class="author">{ &message.author.username }</span>
                <span class="content">
                    <Markdown content=&message.content />
                </span>
            </div>
        </article>
    }
}
