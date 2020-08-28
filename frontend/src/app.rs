use log::*;
use yew::prelude::*;
use yew_router::{Switch, router::Router};
use crate::components::MainComponent;
use crate::components::LoginComponent;

use crate::agents::event_bus::{start_websocket};

use std::cell::RefCell;
use crate::models::{Room, User};
use std::collections::HashSet;
use crate::utils::send_future;
use crate::services::user_service;

use yew_router::route::Route;
use yew_router::prelude::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use uuid::Uuid;

pub struct App {
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
    #[to = "/{room_id}"]
    MainWithRoomSelected(Uuid),
    #[to = "/"]
    Main,
}

pub enum Msg {
    TokenValidated(String, User),
    NavigateToLogin
}

#[derive(Clone)]
pub struct AppState {
    pub users: HashSet<User>,
    pub rooms: HashSet<Room>,
    pub me: Option<User>,
    pub token: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: HashSet::new(),
            rooms: HashSet::new(),
            me: None,
            token: None,
        }
    }
}

thread_local!(pub static APP_STATE: RefCell<AppState> = RefCell::new(AppState::new()));

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
            router: RouteAgentDispatcher::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::TokenValidated(token, user) => {
                let token_clone = token.clone();
                APP_STATE.with(|f| {
                    let mut state = f.borrow_mut();
                    state.token = Some(token);
                    state.me = Some(user.clone());
                    state.users.insert(user);
                });
                start_websocket(token_clone).unwrap();
            },
            Msg::NavigateToLogin => {
                let route = Route::from(AppRoute::Login);
                self.router.send(RouteRequest::ChangeRoute(route));
            }
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <Router<AppRoute, ()>
                render = Router::render(|switch: AppRoute| {
                    match switch {
                        AppRoute::Login => html!{<LoginComponent />},
                        AppRoute::MainWithRoomSelected(room_id) => html!{<MainComponent room_id = Some(room_id) />},
                        AppRoute::Main => html!{<MainComponent room_id = None />},
                    }
                })
            />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let window: web_sys::Window = web_sys::window().unwrap();
            let data = window.local_storage().unwrap().unwrap().get_item("token");
            info!("token 1 {:?}", data);
            match data {
                Ok(Some(token)) => {
                    info!("token 2 {}", token);
                    send_future(self.link.clone(), async {
                        let resp = user_service::get_me(token.clone()).await;
                        match resp {
                            Ok(user) => Msg::TokenValidated(token, user),
                            Err(_) => Msg::NavigateToLogin
                        }
                    })
                }
                _ => {
                    info!("token 2 NONE");
                    self.link.send_message( Msg::NavigateToLogin);
                }
            };
        }
    }
}
