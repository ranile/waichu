mod components;
mod services;
mod websocket;
mod utils;

use components::{Auth, Room as ShowRoom, RoomsList, UserAvatar};

use crate::websocket::{Connection, InternalEventBus, Request, Response};
use common::websocket::{AuthenticatedPayload, OpCode};
use common::{Message, Room, User};
use lazy_static::lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use weblog::console_log;
use yew::prelude::*;
use yew::services::storage::Area;
use yew_functional::{
    function_component, use_effect, use_effect_with_deps, use_ref, use_state, ContextProvider,
};
use yew_material::{MatDrawer, MatDrawerAppContent, MatDrawerTitle, WeakComponentLink};
use yew_router::agent::RouteRequest;
use yew_router::prelude::*;
use yew_state::{SharedHandle, SharedState, SharedStateComponent};
use yew::services::StorageService;
use yew::format::Text;

lazy_static! {
    pub static ref CLIENT: Client = Client::new();
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AppState {
    token: Option<String>,
    rooms: Rc<RefCell<Vec<Room>>>,
    me: Option<User>,
    force_render: u32,
}

const TOKEN_KEY: &str = "token";

impl Default for AppState {
    fn default() -> Self {
        let service = StorageService::new(Area::Local)
            .expect("can't initialize StorageService");

        let token = service.restore::<Text>(TOKEN_KEY).ok();

        Self {
            token,
            rooms: Rc::new(RefCell::new(vec![])),
            me: None,
            force_render: 0,
        }
    }
}

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/login"]
    Auth,
    #[to = "/room/{id}"]
    Rooms(Uuid),
    #[to = "/room"]
    Home,
}

type AppRouter = Router<AppRoute>;

#[derive(Clone, Properties, PartialEq)]
struct HomeProps {
    #[prop_or_default]
    room: Option<Uuid>,
    #[prop_or_default]
    handle: SharedHandle<AppState>,
}

impl SharedState for HomeProps {
    type Handle = SharedHandle<AppState>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}

#[function_component(Home)]
fn home(props: &HomeProps) -> Html {
    let router = use_ref(RouteAgentDispatcher::<()>::new);

    let state = props.handle.state();

    console_log!(format!("state: {:?}", state));

    let rooms = state.rooms.borrow();
    let current = if let Some(room_id) = props.room {
        rooms.iter().find(|it| it.uuid == room_id)
    } else if let Some(room) = rooms.first() {
        let route = Route::from(AppRoute::Rooms(room.uuid));
        router.borrow_mut().send(RouteRequest::ChangeRoute(route));
        Some(room)
    } else {
        None
    };

    let user_avatar_action = if let Some(user) = &state.me {
        html! {
            <SharedStateComponent<UserAvatar> user=user />
        }
    } else {
        html!()
    };

    let (drawer_link, _) = use_state(WeakComponentLink::<MatDrawer>::default);

    let on_nav_click = {
        let drawer_link = Rc::clone(&drawer_link);

        Callback::from(move |_| {
            drawer_link.flip_open_state()
        })
    };

    let room = if let Some(room) = current {
        html! {
            <ShowRoom room=room user_avatar_action=user_avatar_action onnavigationiconclick=on_nav_click />
        }
    } else {
        html! { "Nothing selected" }
    };

    html! {
        <MatDrawer
            drawer_type="modal"
            drawer_link=(*drawer_link).clone()
            has_header=true>

            <MatDrawerTitle>{"Rooms"}</MatDrawerTitle>

            <SharedStateComponent<RoomsList> />

            <MatDrawerAppContent>
                { room }
            </MatDrawerAppContent>
        </MatDrawer>
    }
}

#[function_component(Main)]
fn main_application(handle: &SharedHandle<AppState>) -> Html {
    let (has_sent_connect, set_has_sent_connect) = use_state(|| false);
    let (has_authenticated, set_has_authenticated) = use_state(|| false);

    let router = use_ref(RouteAgentDispatcher::<()>::new);
    let dispatcher = use_ref(websocket::Connection::dispatcher);
    let events_dispatcher = use_ref(InternalEventBus::dispatcher);

    let (token, set_token) = use_state(|| None);

    {
        let set_token = Rc::clone(&set_token);
        use_effect_with_deps(
            move |token| {
                set_token(token.clone());
                || ()
            },
            handle.state().token.clone(),
        );
    }

    {
        let dispatcher = Rc::clone(&dispatcher);
        let router = Rc::clone(&router);
        let handle = handle.clone();

        use_effect(move || {
            let bridge = Connection::bridge(handle.reduce_callback_with(
                move |state, msg: websocket::Response| {
                    match msg {
                        Response::Connected => {
                            dispatcher
                                .borrow_mut()
                                .send(Request::Authenticate(state.token.as_ref().unwrap().clone()));
                        }
                        Response::Message(m) => {
                            match m.op {
                                OpCode::Authenticated => {
                                    let data = serde_json::from_value::<AuthenticatedPayload>(
                                        m.data.clone(),
                                    )
                                    .unwrap();

                                    state.rooms = Rc::new(RefCell::new(data.rooms));
                                    state.me = Some(data.me);
                                    set_has_authenticated(true);

                                    let route = Route::from(AppRoute::Home);
                                    router.borrow_mut().send(RouteRequest::ChangeRoute(route));
                                }
                                OpCode::RoomJoin => {
                                    let data =
                                        serde_json::from_value::<Room>(m.data.clone()).unwrap();

                                    // maybe try making another Agent that sends a message
                                    // when we're added to a new room
                                    // same will be used for notifying messages
                                    // I'll probably switch to that approach when I write the messages Agent
                                    // since it'll be a requirement unless I wanna make spaghetti
                                    // or I could use multiple states, will see

                                    state.rooms.borrow_mut().push(data);
                                    state.force_render += 1;
                                }
                                OpCode::MessageCreate => {
                                    let data =
                                        serde_json::from_value::<Message>(m.data.clone()).unwrap();
                                    events_dispatcher
                                        .borrow_mut()
                                        .send(websocket::internal_events::Request::NewMessage(data))
                                }
                                _ => panic!("fucked"),
                            }
                            console_log!(JsValue::from_serde(&*m).unwrap());
                        }
                        Response::Error(e) => {
                            console_log!(e.to_string());
                            dispatcher.borrow_mut().send(Request::Disconnect);
                        }
                        Response::Closed => {
                            dispatcher.borrow_mut().send(Request::Disconnect);
                        }
                    }
                },
            ));

            || drop(bridge)
        })
    };

    match &handle.state().token {
        Some(_) => {
            if !*has_sent_connect {
                let base = yew::utils::window().location().host().unwrap();
                dispatcher
                    .borrow_mut()
                    .send(websocket::Request::Connect(format!("ws://{}/api/ws", base)));
                set_has_sent_connect(true)
            }
        }
        None => {
            let route = Route::from(AppRoute::Auth);
            router.borrow_mut().send(RouteRequest::ChangeRoute(route));
        }
    };
    let ret = if *has_sent_connect && !*has_authenticated {
        html! {
            "Loading "
        }
    } else {
        html! {
        <ContextProvider<Rc<Option<String>>> context=token>
            <AppRouter
                render=AppRouter::render(switch)
            />
        </ContextProvider<Rc<Option<String>>>>
        }
    };

    ret
}

fn switch(switch: AppRoute) -> Html {
    match switch {
        AppRoute::Auth => html! { <Auth /> },
        AppRoute::Rooms(room) => html! { <SharedStateComponent<Home> room=room /> },
        AppRoute::Home => html! { <SharedStateComponent<Home> /> },
    }
}

#[function_component(Show)]
fn show(handle: &SharedHandle<AppState>) -> Html {
    html! { <pre style="font-family: monospace;">{ format!("{:#?}", handle.state()) }</pre> }
}

#[function_component(Application)]
fn application() -> Html {
    html! {
        <>
            // <SharedStateComponent<Show> />
            <SharedStateComponent<Main> />
        </>
    }
}

fn main() {
    yew::start_app::<Application>()
}
