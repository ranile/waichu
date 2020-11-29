use crate::services::room::create_room;
use crate::{AppRoute, AppState};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use weblog::console_log;
use yew::prelude::*;
use yew::services::DialogService;
use yew_functional::{function_component, use_ref, use_state};
use yew_material::{
    dialog::{ActionType, MatDialogAction},
    MatButton, MatDialog, MatList, MatListItem, MatTextField, WeakComponentLink,
};
use yew_router::agent::RouteRequest;
use yew_router::prelude::*;
use yew_state::SharedHandle;

#[function_component(RoomsList)]
pub fn rooms_list(handle: &SharedHandle<AppState>) -> Html {
    let router = use_ref(RouteAgentDispatcher::<()>::new);

    let rooms = handle
        .state()
        .rooms
        .borrow()
        .iter()
        .map(|room| {
            let onclick = {
                let router = Rc::clone(&router);
                let uuid = room.uuid;
                Callback::from(move |_| {
                    let route = Route::from(AppRoute::Rooms(uuid));
                    router.borrow_mut().send(RouteRequest::ChangeRoute(route));
                })
            };

            html! {
                // MatListItem must be outside for activatable to work
                <span onclick=&onclick>
                     <MatListItem>{ &room.name }</MatListItem>
                </span>
            }
        })
        .collect::<Vec<Html>>();

    let (room_name, set_room_name) = use_state(String::new);
    let (dialog_link, _) = use_state(WeakComponentLink::<MatDialog>::default);
    let new_onclick = {
        let dialog_link = Rc::clone(&dialog_link);
        Callback::from(move |_| dialog_link.show())
    };

    let on_room_create_click = {
        let room_name = Rc::clone(&room_name);
        let token = handle.state().token.clone();
        Callback::from(move |_| {
            console_log!("room name: ", &*room_name);

            let room_name = Rc::clone(&room_name);
            let token = token.as_ref().unwrap().clone();

            spawn_local(async move {
                let token = token;
                if let Err(e) = create_room(&token, &**room_name).await {
                    DialogService::alert(&format!("Error creating room: {}", e))
                };
            })
        })
    };

    html! {<>
        <MatList activatable=true>
            { for rooms }
            <span class="new-room" onclick=new_onclick>
                <MatButton label="New" />
            </span>
        </MatList>

        <MatDialog
            heading="Create room"
            dialog_link=&*dialog_link
            // onclosed=on_dialog_closed TODO fix yew-material coz ya boi an idiot
        >
            <section class="create-room-container">
                <MatTextField
                    label="Room name"
                    value=&*room_name
                    oninput=Callback::from(move |e: InputData| set_room_name(e.value))
                />
            </section>

            <MatDialogAction action_type=ActionType::Primary action="create">
                <span onclick=on_room_create_click>
                    <MatButton label="Create" />
                </span>
            </MatDialogAction>
            <MatDialogAction action_type=ActionType::Secondary action="cancel">
                <MatButton label="Cancel" />
            </MatDialogAction>
        </MatDialog>
    </>}
}
