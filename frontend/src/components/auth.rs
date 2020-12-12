use crate::services::auth::{signin, signup};
use crate::{AppState, TOKEN_KEY};
use common::payloads::Credentials;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::services::storage::Area;
use yew::services::StorageService;
use yew_functional::{function_component, use_state};
use yew_material::{
    MatButton, MatCheckbox, MatFormfield, MatIcon, MatLinearProgress, MatTab, MatTabBar,
    MatTextField, TextFieldType,
};
use yew_state::{SharedHandle, SharedStateComponent};

#[function_component(Signin)]
pub fn signin_comp(handle: &SharedHandle<AppState>) -> Html {
    let (username, set_username) = use_state(|| "".to_owned());
    let (password, set_password) = use_state(|| "".to_owned());

    let (error, set_error) = use_state(|| None);

    let (has_sent_request, set_has_sent_request) = use_state(|| false);

    let (remember_me, set_remember_me) = use_state(|| false);

    let onclick = {
        let set_token = Rc::new(handle.reduce_callback_with(|s, (token, remember_me)| {
            if remember_me {
                let mut service =
                    StorageService::new(Area::Local).expect("can't initialize StorageService");

                service.store(TOKEN_KEY, Ok(String::from(&token)));
            }

            s.token = Some(token);
        }));

        let set_has_sent_request = Rc::clone(&set_has_sent_request);

        Callback::from(move |_| {
            let remember_me = Rc::clone(&remember_me);

            let credentials = Credentials {
                username: (*username).clone(),
                password: (*password).clone(),
            };

            let set_token = Rc::clone(&set_token);
            let set_error = Rc::clone(&set_error);
            let set_has_sent_request = Rc::clone(&set_has_sent_request);

            set_has_sent_request(true);

            spawn_local(async move {
                match signin(credentials).await {
                    Ok(token) => (*set_token).emit((token.token, *remember_me)),
                    Err(e) => {
                        set_has_sent_request(false);
                        set_error(Some(e))
                    }
                }
            })
        })
    };

    let error_html = if let Some(error) = &*error {
        html! {
            <div class="error">
                <MatIcon>{"error"}</MatIcon>
                <span>{ error }</span>
            </div>
        }
    } else {
        html!()
    };

    let progress_bar = if *has_sent_request {
        html! {
            <MatLinearProgress indeterminate=true />
        }
    } else {
        html!()
    };

    html! {<>
        // <div class="mdc-card">
            {progress_bar}

            <div class="card-content">
                <MatTextField
                    outlined=true
                    required=true
                    disabled=*has_sent_request
                    field_type=TextFieldType::Text
                    label="Username"
                    oninput=Callback::from(move |e: InputData| set_username(e.value))
                />

                <MatTextField
                    outlined=true
                    required=true
                    disabled=*has_sent_request
                    field_type=TextFieldType::Password
                    label="Password"
                    oninput=Callback::from(move |e: InputData| set_password(e.value))
                 />


                <MatFormfield label="Remember me?">
                    <MatCheckbox
                        onchange=Callback::from(move |state| set_remember_me(state))
                    />
                </MatFormfield>

                 {error_html}
            </div>

            <div class="mdc-card__action-buttons">
                <span onclick=onclick>
                    <MatButton raised=true label="Sign in" disabled=*has_sent_request />
                </span>
            </div>
        // </div>
    </>}
}

//noinspection DuplicatedCode
#[function_component(Signup)]
pub fn signup_comp(handle: &SharedHandle<AppState>) -> Html {
    let (username, set_username) = use_state(|| "".to_owned());
    let (password, set_password) = use_state(|| "".to_owned());
    let (confirm_password, set_confirm_password) = use_state(|| "".to_owned());

    let (error, set_error) = use_state(|| None);

    let (has_sent_request, set_has_sent_request) = use_state(|| false);

    let onclick = {
        let set_token = Rc::new(handle.reduce_callback_with(|s, resp| s.token = Some(resp)));
        let set_has_sent_request = Rc::clone(&set_has_sent_request);

        Callback::from(move |_| {
            if *password != *confirm_password {
                set_error(Some(anyhow::anyhow!("Passwords do not match")));
                return;
            }

            let credentials = Credentials {
                username: (*username).clone(),
                password: (*password).clone(),
            };

            let set_token = set_token.clone();
            let set_error = set_error.clone();
            let set_has_sent_request = Rc::clone(&set_has_sent_request);

            set_has_sent_request(true);

            spawn_local(async move {
                match signup(credentials).await {
                    Ok(token) => set_token.emit(token.token),
                    Err(e) => {
                        set_has_sent_request(false);
                        set_error(Some(e))
                    }
                }
            });
        })
    };

    let error_html = match &*error {
        Some(error) => {
            html! {
                <div class="error">
                    <MatIcon>{"error"}</MatIcon>
                    <span>{ error }</span>
                </div>
            }
        }
        _ => {
            html!()
        }
    };

    let progress_bar = if *has_sent_request {
        html! {
            <MatLinearProgress indeterminate=true />
        }
    } else {
        html!()
    };

    html! {<>
        // <div class="mdc-card">
            {progress_bar}

            <div class="card-content">
                <MatTextField
                    outlined=true
                    required=true
                    disabled=*has_sent_request
                    field_type=TextFieldType::Text
                    label="Username"
                    oninput=Callback::from(move |e: InputData| set_username(e.value))
                />

                <MatTextField
                    outlined=true
                    required=true
                    disabled=*has_sent_request
                    field_type=TextFieldType::Password
                    label="Password"
                    oninput=Callback::from(move |e: InputData| set_password(e.value))
                 />

                <MatTextField
                    outlined=true
                    required=true
                    disabled=*has_sent_request
                    field_type=TextFieldType::Password
                    label="Confirm password"
                    oninput=Callback::from(move |e: InputData| set_confirm_password(e.value))
                />

                 {error_html}
            </div>

            <div class="mdc-card__action-buttons">
                <span onclick=onclick>
                    <MatButton raised=true label="Sign up" disabled=*has_sent_request />
                </span>
            </div>
        // </div>
    </>}
}

#[derive(Clone, Copy)]
enum Tabs {
    SignIn,
    SignUp,
}

#[function_component(Auth)]
pub fn auth_comp() -> Html {
    let (activated_tab, set_activated_tab) = use_state(|| Tabs::SignIn);

    let on_activated = {
        let set_activated_tab = Rc::clone(&set_activated_tab);

        Callback::from(move |index| match index {
            0 => set_activated_tab(Tabs::SignIn),
            1 => set_activated_tab(Tabs::SignUp),
            num => unreachable!("{}", num),
        })
    };

    let displayed_comp = match *activated_tab {
        Tabs::SignIn => html! { <SharedStateComponent<Signin> /> },
        Tabs::SignUp => html! { <SharedStateComponent<Signup> /> },
    };

    html! {
        <div id="auth-wrapper-container">
            <div id="auth-wrapper">
                <div class="mdc-card">

                    <MatTabBar onactivated=on_activated>
                        <MatTab is_fading_indicator=true label="Sign in" />
                        <MatTab is_fading_indicator=true label="Sign up" />
                    </MatTabBar>
                    { displayed_comp }
                </div>
            </div>
        </div>
    }
}
