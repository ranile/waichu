use yew::prelude::*;
use log::*;
use yew_router::route::Route;
use yew_router::prelude::RouteAgentDispatcher;
use crate::app::{AppRoute, APP_STATE};
use yew_router::agent::RouteRequest;

use crate::utils::{send_future};
use crate::agents::event_bus::start_websocket;
use crate::services::auth_service::{Credentials};
use crate::services::{auth_service, user_service};


pub struct LoginComponent {
    link: ComponentLink<Self>,
    credentials: Credentials,
    router: RouteAgentDispatcher<()>,
    state: CurrentlyShowingState,
}

pub enum Msg {
    UsernameUpdate(String),
    PasswordUpdate(String),
    Login,
    Authenticate(String),
    Error,
    Signup,
    FlipState
}

pub enum CurrentlyShowingState {
    Signin,
    Signup,
}

impl Component for LoginComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        LoginComponent {
            link,
            credentials: Credentials {
                username: "".to_string(),
                password: "".to_string(),
            },
            router: RouteAgentDispatcher::new(),
            state: CurrentlyShowingState::Signin,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Login => {
                let creds = self.credentials.clone();

                send_future(self.link.clone(), async move {
                    let data = auth_service::signin(creds).await;
                    return Msg::Authenticate(data.token);
                });
                false
            }
            Msg::UsernameUpdate(username) => {
                self.credentials.username = username;
                false
            }
            Msg::PasswordUpdate(password) => {
                self.credentials.password = password;
                false
            }
            Msg::Authenticate(token) => {
                let window: web_sys::Window = web_sys::window().unwrap();
                window.local_storage().unwrap().unwrap().set_item("token", &token);
                let token_clone = token.clone();
                send_future(self.link.clone(), async move {
                    // TODO make this a function
                    // Code duplication = bad
                    let me = user_service::get_me(token_clone.clone()).await.unwrap();
                    APP_STATE.with(|refcell| {
                        let mut state = refcell.borrow_mut();
                        state.token = Some(token_clone);
                        state.me = Some(me.clone());
                        state.users.insert(me);
                    });
                    Msg::Error
                });

                start_websocket(token);
                let route = Route::from(AppRoute::Main);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
            Msg::Signup => {
                // TODO
                false
            }
            Msg::FlipState => {
                match self.state {
                    CurrentlyShowingState::Signin => self.state = CurrentlyShowingState::Signup,
                    CurrentlyShowingState::Signup => self.state = CurrentlyShowingState::Signin
                };
                true
            }
            Msg::Error => {
                info!("Fucked");
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let flip_state = self.link.callback(|_| Msg::FlipState);
        yew::include_html!("frontend/src/components/login/login.html")
    }
}
