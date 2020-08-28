use log::*;
use crate::utils::{send_future};
use crate::agents::event_bus::{EventBus, WebsocketMessage, Op};
use crate::models::{Room, User};
use uuid::Uuid;
use yew::prelude::*;
use yew::{Html, InputData};
use crate::components::{MessageListComponent, NewMessageComponent};
use crate::app::{APP_STATE, AppRoute};
use crate::services::{rooms_service};
use yew_router::agent::{RouteAgentDispatcher, RouteRequest};
use yew_router::route::Route;

pub struct MainComponent {
    link: ComponentLink<Self>,
    _producer: Box<dyn Bridge<EventBus>>,
    new_room_name: String,
    selected_room: Option<Uuid>,
    is_loading: bool,
    router: RouteAgentDispatcher<()>,
    should_show_dialog: bool
}

pub enum Msg {
    NewMessage(WebsocketMessage),
    RoomNameInput(String),
    RoomSelect(Uuid),
    CreateRoom,
    Ignore,
    FlipDialogState
}

#[derive(Properties, Clone)]
pub struct Props {
    pub room_id: Option<Uuid>,
}

impl Component for MainComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(Msg::NewMessage);
        let _producer = EventBus::bridge(callback);

        MainComponent {
            link,
            _producer,
            new_room_name: "".to_string(),
            selected_room: props.room_id,
            is_loading: true,
            router: RouteAgentDispatcher::new(),
            should_show_dialog: false
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(msg) => {
                if msg.op == Op::Authenticated {
                    self.is_loading = false;

                    APP_STATE.with(move |f| {
                        let state = f.borrow();

                        if let None = self.selected_room {
                            self.select_room(state.rooms.iter().next().map(|r| r.uuid).unwrap());
                        };
                    });
                    true
                } else if msg.op == Op::RoomCreate {
                    let room = serde_json::from_value::<Room>(msg.data).unwrap();
                    APP_STATE.with(move |f| {
                        let mut state = f.borrow_mut();
                        state.rooms.insert(room.clone());
                        self.select_room(room.uuid);
                    });
                    true
                } else { false }
            }
            Msg::RoomSelect(uuid) => {
                self.select_room(uuid);
                true
            }
            Msg::FlipDialogState => {
                self.should_show_dialog = !self.should_show_dialog;
                true
            }
            Msg::RoomNameInput(name) => {
                self.new_room_name = name;
                false
            }
            Msg::CreateRoom => {
                let name = self.new_room_name.clone();
                send_future(self.link.clone(), async move {
                    rooms_service::create(name).await;
                    Msg::Ignore
                });
                false
            }
            _ => { false }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {

        let html = if self.is_loading {
            yew::include_html!("frontend/src/components/main/loading.html")
        } else {
            let mut rooms = vec![];
            let mut me: Option<User> = None;
            APP_STATE.with(|f| {
                let state = f.borrow();
                state.rooms.iter().for_each(|r| rooms.push(r.clone()));
                me = state.me.clone()
            });
            info!("MEEE {:?}", me);

            let rooms_clone = rooms.clone();
            let selected_room: Vec<&Room> = rooms_clone.iter().filter(|r| r.uuid == self.selected_room.unwrap()).collect();

            let new_room_onclick = self.link.callback(|_| {
                Msg::FlipDialogState
            });

            let on_input = self.link.callback(|e: InputData| Msg::RoomNameInput(e.value));
            let onclick = self.link.callback(|_| Msg::CreateRoom);
            let dialog_class = if self.should_show_dialog { "dialog dialog-open" } else { "dialog" };

            yew::include_html!("frontend/src/components/main/main.html")
        };
        html
    }
}

impl MainComponent {
    fn select_room(&mut self, room: Uuid) {
        // Kinda hacky but it works so ¯\_(ツ)_/¯
        let route = Route::from(AppRoute::MainWithRoomSelected(room));
        self.router.send(RouteRequest::ChangeRoute(route));
        self.selected_room = Some(room);
    }
}

fn room_name_class(name: Uuid, selected: Uuid) -> &'static str {
    if name == selected {
        "room-selected"
    } else {
        ""
    }
}
