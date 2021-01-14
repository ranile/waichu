use crate::utils::{asset_url, use_token};
use crate::{services, AppState};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew::services::reader::File;
use yew_functional::{function_component, use_state};
use yew_material::text_inputs::TextFieldType;
use yew_material::{MatIcon, MatTextField};
use yew_state::SharedHandle;

#[function_component(UpdateProfile)]
pub fn update_profile(handle: &SharedHandle<AppState>) -> Html {
    let user = match handle.state().me.as_ref() {
        Some(user) => user,
        None => {
            return html!("Loading");
        }
    };

    let (editing_username, set_editing_username) = use_state(|| false);
    let (editing_password, set_editing_password) = use_state(|| false);

    let (new_username, set_new_username) = {
        let username = user.username.clone();
        use_state(|| username)
    };

    let (new_password, set_new_password) = use_state(String::new);

    let edit_username = if *editing_username {
        html! {
            <section class="edit-field-container">
                <MatTextField
                    outlined=true
                    required=true
                    field_type=TextFieldType::Text
                    label="New Username"
                    value=&*new_username
                    oninput=Callback::from(move |e: InputData| set_new_username(e.value))
                />

                <div role="separator" class="separator"></div>

                <span class="icon-wrapper">
                    <MatIcon>{ "save" }</MatIcon>
                </span>
            </section>
        }
    } else {
        html! {
            <section class="edit-field-container">
                <span>{ &user.username }</span>
                <div role="separator" class="separator"></div>
                <span class="icon-wrapper" onclick=Callback::from(move |_| set_editing_username(true))>
                    <MatIcon>{ "edit" }</MatIcon>
                </span>
            </section>
        }
    };
    let edit_password = if *editing_password {
        html! {
            <section class="edit-field-container">
                <MatTextField
                    outlined=true
                    required=true
                    field_type=TextFieldType::Password
                    label="New Password"
                    value=&*new_password
                    oninput=Callback::from(move |e: InputData| set_new_password(e.value))
                />

                <div role="separator" class="separator"></div>

                <span class="icon-wrapper">
                    <MatIcon>{ "save" }</MatIcon>
                </span>
            </section>
        }
    } else {
        html! {
            <section class="edit-field-container">
                <span>{ "*******" }</span>
                <div role="separator" class="separator"></div>
                <span class="icon-wrapper" onclick=Callback::from(move |_| set_editing_password(true))>
                    <MatIcon>{ "edit" }</MatIcon>
                </span>
            </section>
        }
    };

    let (hidden, set_hidden) = use_state(|| true);

    let on_avatar_mouseover = {
        let set_hidden = Rc::clone(&set_hidden);
        Callback::from(move |_| {
            set_hidden(false);
        })
    };

    let on_avatar_mouseout = {
        let set_hidden = Rc::clone(&set_hidden);
        Callback::from(move |_| {
            set_hidden(true);
        })
    };

    let hide_class = if *hidden { Some("hide") } else { None };

    let (avatar_input_ref, _) = use_state(NodeRef::default);

    let on_file_input_click = {
        let hidden = Rc::clone(&hidden);
        let avatar_input_ref = Rc::clone(&avatar_input_ref);
        Callback::from(move |_| {
            avatar_input_ref.cast::<HtmlElement>().unwrap().click();
            if !*hidden {
                weblog::console_log!("click")
            }
        })
    };

    let token = use_token();
    let avatar = user.avatar.clone();
    let (avatar, set_avatar) = use_state(|| avatar);

    let on_file_change = Callback::from(move |value| {
        weblog::console_log!("file select 1");
        let mut result = Vec::new();
        if let ChangeData::Files(files) = value {
            weblog::console_log!("file select 2");
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| File::from(v.unwrap()));
            result.extend(files);
            let file = result.first().unwrap().clone();
            let token = Rc::clone(&token);
            let set_avatar = Rc::clone(&set_avatar);

            spawn_local(async move {
                let asset = services::user::update_avatar(&*token, file).await.unwrap();
                set_avatar(Some(asset));
            });
        }
    });

    html! {
        <div id="update-profile-container">
            <div class="mdc-card">
                <section class="avatar-username-container">
                    <div class="avatar" onmouseover=on_avatar_mouseover onmouseout=on_avatar_mouseout onclick=on_file_input_click>
                        <img src=asset_url((*avatar).as_ref()) />
                        <div class=classes!("file-input", hide_class)>{ "Update profile picture" }</div>
                    </div>
                    <span>{ &user.username }</span>
                </section>

                { edit_username }

                { edit_password }
            </div>
            <input id="avatar-input" type="file" style="display: none;" ref=(*avatar_input_ref).clone() onchange=on_file_change />
        </div>
    }
}
