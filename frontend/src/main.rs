mod components;
mod services;
mod utils;
mod websocket;

use components::{Auth, Room as ShowRoom, RoomsList, UpdateProfile, UserAvatar};

use crate::utils::{asset_url, is_on_mobile};
use crate::websocket::{Connection, InternalEventBus, Request, Response};
use common::websocket::{AuthenticatedPayload, OpCode};
use common::{Message, Room, User};
use lazy_static::lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use weblog::console_log;
use yew::format::Text;
use yew::prelude::*;
use yew::services::storage::Area;
use yew::services::StorageService;
use yew_functional::{
    function_component, use_effect, use_effect_with_deps, use_ref, use_state, ContextProvider,
};
use yew_material::menu::Corner;
use yew_material::{
    drawer::MatDrawerAppContent,
    list::{GraphicType, ListIndex},
    MatDrawer, MatIcon, MatListItem, MatMenu, WeakComponentLink,
};
use yew_router::agent::RouteRequest;
use yew_router::prelude::*;
use yew_state::{SharedHandle, SharedState, SharedStateComponent};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    pub static ref CLIENT: Client = Client::new();
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AppState {
    token: Option<String>,
    rooms: Rc<RefCell<Vec<Room>>>,
    me: Option<User>,
    force_render: u32,
    prefers_dark: bool,
}

const TOKEN_KEY: &str = "token";
const PREFERS_DARK_KEY: &str = "prefersDark";

const DATA_THEME_ATTR: &str = "data-theme";

impl Default for AppState {
    fn default() -> Self {
        let service = StorageService::new(Area::Local).expect("can't initialize StorageService");

        let token = service.restore::<Text>(TOKEN_KEY).ok();
        let prefers_dark = service
            .restore::<Text>(PREFERS_DARK_KEY)
            .map(|it| it.parse::<bool>().unwrap_or(false))
            .unwrap_or_else(|_| {
                let media = yew::utils::window()
                    .match_media("(prefers-color-scheme: dark)")
                    .unwrap()
                    .unwrap();
                media.matches()
            });

        Self {
            token,
            rooms: Rc::new(RefCell::new(vec![])),
            me: None,
            force_render: 0,
            prefers_dark,
        }
    }
}

#[derive(Switch, Clone, Debug, Copy)]
pub enum AppRoute {
    #[to = "/profile/update"]
    UpdateProfile,
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
    let is_on_mobile = is_on_mobile();
    console_log!("is_on_mobile", is_on_mobile);

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

    let (menu_link, _) = use_state(WeakComponentLink::<MatMenu>::default);

    let user_avatar_action = if let Some(user) = &state.me {
        let onclick = {
            let menu_link = menu_link.clone();
            Callback::from(move |_| menu_link.show())
        };

        let onload = {
            let menu_link = menu_link.clone();
            Callback::from(move |node_ref: NodeRef| {
                let element = node_ref.cast::<web_sys::HtmlElement>().unwrap();
                menu_link.set_anchor(element);
                console_log!("set anchor");
            })
        };

        let onaction = {
            let reset_callback = props.handle.reduce_callback(move |state| {
                state.token = None;
                state.rooms = Rc::new(RefCell::new(vec![]));
                state.me = None;
            });

            Callback::from(move |list_index| {
                if let ListIndex::Single(Some(index)) = list_index {
                    console_log!("index", format!("{}", index));
                    match index {
                        0 => {
                            let route = Route::from(AppRoute::UpdateProfile);
                            router.borrow_mut().send(RouteRequest::ChangeRoute(route));
                        }
                        1 => {
                            console_log!("sign out");
                            let window = yew::utils::window();
                            window.local_storage().unwrap().unwrap().clear().unwrap();

                            reset_callback.emit(());
                        }
                        _ => unreachable!(),
                    }
                } else {
                    unreachable!("menu isn't multi so index should be single")
                }
            })
        };

        html! {<>
            <UserAvatar user=user show_details_on_click=false onclick=onclick onload=onload />
            <MatMenu menu_link=&*menu_link corner=Corner::BottomRight onaction=onaction>
                <MatListItem graphic=GraphicType::Avatar noninteractive=true>
                    <span>{ &user.username }</span>
                    <img slot="graphic" src=asset_url(user.avatar.as_ref()) />
                </MatListItem>
                <li divider=true role="separator" />

                <MatListItem graphic=GraphicType::Icon>
                    <span>{ "Update" }</span>
                    <span slot="graphic">
                        <MatIcon>{ "edit" }</MatIcon>
                    </span>
                </MatListItem>

                <MatListItem graphic=GraphicType::Icon>
                    <span>{ "Sign out" }</span>
                    <span slot="graphic">
                        <MatIcon>{ "exit_to_app" }</MatIcon>
                    </span>
                </MatListItem>
            </MatMenu>
        </>}
    } else {
        html!()
    };

    let (drawer_link, _) = use_state(WeakComponentLink::<MatDrawer>::default);

    let on_nav_click = if is_on_mobile {
        let drawer_link = Rc::clone(&drawer_link);

        Some(Callback::from(move |_| drawer_link.flip_open_state()))
    } else {
        None
    };

    let room = html! {
        <ShowRoom room=current.cloned() user_avatar_action=user_avatar_action onnavigationiconclick=on_nav_click />
    };

    let drawer_type = if is_on_mobile { "modal" } else { "" };

    html! {
        <MatDrawer
            drawer_type=drawer_type
            drawer_link=(*drawer_link).clone()>
            // has_header=true>

            <div id="drawer-sidebar">
                <h2>{"Rooms"}</h2>

                <SharedStateComponent<RoomsList> />
            </div>

            <MatDrawerAppContent>
                { room }
            </MatDrawerAppContent>
        </MatDrawer>
    }
}

#[function_component(Main)]
fn main_application(handle: &SharedHandle<AppState>) -> Html {
    let theme = if handle.state().prefers_dark {
        "dark"
    } else {
        "default"
    };

    yew::utils::document()
        .body()
        .unwrap()
        .set_attribute(DATA_THEME_ATTR, theme)
        .expect("failed to set theme attribute");

    let (has_sent_connect, set_has_sent_connect) = use_state(|| false);
    let (has_authenticated, set_has_authenticated) = use_state(|| false);

    let router = use_ref(RouteAgentDispatcher::<()>::new);
    let route_service = use_ref(RouteService::<()>::new);
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
        let route_service = Rc::clone(&route_service);
        let handle = handle.clone();

        use_effect(move || {
            let bridge = Connection::bridge(handle.reduce_callback_with(
                move |state, msg: websocket::Response| match msg {
                    Response::Connected => {
                        dispatcher
                            .borrow_mut()
                            .send(Request::Authenticate(state.token.as_ref().unwrap().clone()));
                    }
                    Response::Message(m) => {
                        match m.op {
                            OpCode::Authenticated => {
                                let data =
                                    serde_json::from_value::<AuthenticatedPayload>(m.data.clone())
                                        .unwrap();

                                state.rooms = Rc::new(RefCell::new(data.rooms));
                                state.me = Some(data.me);
                                set_has_authenticated(true);

                                let uuid = route_service
                                    .borrow()
                                    .get_route()
                                    .route
                                    .replace("/room/", "");
                                let uuid = Uuid::from_str(&uuid);

                                let current_route = route_service.borrow().get_route().route;
                                // todo remove this when using query params -- see next todo
                                #[allow(clippy::if_same_then_else)]
                                let route = if current_route.starts_with("/room") {
                                    match uuid {
                                        Ok(uuid) => AppRoute::Rooms(uuid),
                                        Err(_) => AppRoute::Home,
                                    }
                                } else if current_route.starts_with("/profile/update") {
                                    AppRoute::UpdateProfile
                                } else if current_route.starts_with("/login") {
                                    AppRoute::Home // todo check for redirect query
                                } else if current_route == "/" {
                                    AppRoute::Home
                                } else {
                                    panic!(
                                        "already handled route ({}) - should never panic",
                                        current_route
                                    )
                                };
                                let route = Route::from(route);
                                router.borrow_mut().send(RouteRequest::ChangeRoute(route));
                            }
                            OpCode::RoomJoin => {
                                let data = serde_json::from_value::<Room>(m.data.clone()).unwrap();

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
                            OpCode::UserUpdate => {
                                let data = serde_json::from_value::<User>(m.data.clone()).unwrap();
                                if let Some(me) = &state.me {
                                    if me.uuid == data.uuid {
                                        state.me = Some(data);
                                    }
                                }
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
                },
            ));

            || drop(bridge)
        })
    };

    match &handle.state().token {
        Some(_) => {
            if !*has_sent_connect {
                let base = yew::utils::window().location().host().unwrap();
                let protocol = yew::utils::window().location().protocol().unwrap();

                let ws_protocol = if protocol.starts_with("https") {
                    "wss"
                } else {
                    "ws"
                };

                dispatcher
                    .borrow_mut()
                    .send(websocket::Request::Connect(format!(
                        "{}://{}/api/ws",
                        ws_protocol, base
                    )));
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
    console_log!("route", format!("{:?}", switch));
    match switch {
        AppRoute::UpdateProfile => html! { <SharedStateComponent<UpdateProfile> /> },
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
