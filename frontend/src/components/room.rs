use crate::components::user_avatar::PROFILE_PICTURE_URL;
use crate::components::{CreateMessage, RoomMessages};
use crate::services::room::{fetch_room_members, join_room};
use common::User;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use weblog::console_log;
use yew::prelude::*;
use yew_functional::{
    function_component, use_effect, use_effect_with_deps, use_state,
};
use yew_material::{
    dialog::{ActionType, MatDialogAction},
    top_app_bar::{MatTopAppBar, MatTopAppBarActionItems, MatTopAppBarTitle,  MatTopAppBarNavigationIcon},
    MatButton, MatDialog, MatIcon, MatTextField, TextFieldType, WeakComponentLink, MatIconButton
};
use crate::utils::use_token;

#[derive(Clone, Properties, PartialEq)]
struct UserCardProps {
    user: User,
}

#[function_component(UserCard)]
fn user_card(props: &UserCardProps) -> Html {
    html! {
        <article>
            <img src=PROFILE_PICTURE_URL />
            <span>{ &props.user.username }</span>
        </article>
    }
}

#[derive(Clone, Properties, PartialEq)]
pub struct ShowRoomProps {
    pub room: common::Room,
    pub user_avatar_action: Html,
    pub onnavigationiconclick: Callback<()>,
}

#[function_component(Room)]
pub fn show_room(props: &ShowRoomProps) -> Html {
    let (dialog_link, _) = use_state(WeakComponentLink::<MatDialog>::default);
    let (members, set_members) = use_state(Vec::new);
    let room_id = props.room.uuid;

    let (member_fetch_error, set_member_fetch_error) = use_state(|| None);

    let token = use_token();

    {
        let set_members = Rc::clone(&set_members);
        let token = Rc::clone(&token);

        use_effect_with_deps(
            move |room_id| {
                let set_members = Rc::clone(&set_members);
                let set_member_fetch_error = Rc::clone(&set_member_fetch_error);
                let room_id = *room_id;

                spawn_local(async move {
                    match fetch_room_members(&*token, room_id).await {
                        Ok(members) => set_members(members),
                        Err(e) => set_member_fetch_error(Some(e)),
                    };
                });

                || {}
            },
            room_id,
        );
    }

    let room_name_click = {
        let dialog_link = Rc::clone(&dialog_link);
        Callback::from(move |_| {
            dialog_link.show();
            console_log!(room_id.to_string())
        })
    };

    let user_cards = match &*member_fetch_error {
        Some(e) => vec![html!(e.to_string())],
        None => members
            .iter()
            .map(|member| html! { <UserCard user=&member.user /> })
            .collect::<Vec<Html>>(),
    };

    use_effect(|| {
        let func = js_sys::Function::new_no_args("document.querySelector('mwc-top-app-bar').setAttribute('style', `--mdc-top-app-bar-width: calc(100% - ${document.querySelector('body > mwc-drawer > mwc-list').offsetWidth}px)`)");
        let _ = func.call0(&yew::utils::window());

        yew::utils::window().set_onresize(Some(&func));

        || yew::utils::window().set_onresize(None)
    });

    let nav_icon_clicked = {
        let onnavigationiconclick = props.onnavigationiconclick.clone();
        Callback::from(move |_| {
            onnavigationiconclick.emit(());
        })
    };

    let (invitee_username, set_invitee_username) = use_state(String::new);
    let (invite_dialog_link, _) = use_state(WeakComponentLink::<MatDialog>::default);
    let invite_onclick = {
        let invite_dialog_link = Rc::clone(&invite_dialog_link);
        Callback::from(move |_| {
            invite_dialog_link.show();
        })
    };

    let add_member_callback = {
        let invitee_username = Rc::clone(&invitee_username);
        let token = Rc::clone(&token);
        let (members, set_members) = (Rc::clone(&members), Rc::clone(&set_members));

        Callback::from(move |_| {
            let invitee_username = Rc::clone(&invitee_username);
            let token = Rc::clone(&token);
            let (mut members, set_members) = (Rc::clone(&members), Rc::clone(&set_members));

            spawn_local(async move {
                match join_room(&*token, room_id, &**invitee_username).await {
                    Ok(member) => {
                        let members = Rc::make_mut(&mut members);
                        members.push(member);
                        set_members(members.to_vec());
                    }
                    Err(e) => weblog::console_error!(e.to_string()),
                }
            })
        })
    };

    html! {<>
        <MatTopAppBar
            onnavigationiconclick=nav_icon_clicked>
            <MatTopAppBarNavigationIcon>
                <MatIconButton icon="menu"></MatIconButton>
            </MatTopAppBarNavigationIcon>

            <MatTopAppBarTitle>
                <span class="room-name" onclick=room_name_click>
                    { &props.room.name }
                </span>
            </MatTopAppBarTitle>

            <MatTopAppBarActionItems>
                // why not &
                { props.user_avatar_action.clone() }
            </MatTopAppBarActionItems>
        </MatTopAppBar>

        <section class="room-content">
            <RoomMessages room=&props.room />
            <CreateMessage room=&props.room />
        </section>

        <MatDialog
            heading=&props.room.name
            dialog_link=&*dialog_link
            // onclosed=on_dialog_closed TODO fix yew-material coz ya boi an idiot
        >
            <section class="room-info">
                <section class="room-members-container">
                    <header>
                        <MatIcon>{ "people" }</MatIcon>
                        <h3>{ "Members" }</h3>
                    </header>
                    <article class="add-user-button-container">
                        <MatIcon>{ "person_add" }</MatIcon>
                        <span onclick=invite_onclick>{ "Add member" }</span>
                    </article>
                    { for user_cards }
                </section>

                <section class="room-timestamp">
                    <header>
                        <MatIcon>{ "access_time" }</MatIcon>
                        <h3>{ "Created at" }</h3>
                    </header>
                    <span>{ &props.room.created_at }</span>
                </section>
            </section>

            <MatDialogAction action_type=ActionType::Secondary action="cancel">
                <MatButton label="Close" />
            </MatDialogAction>
        </MatDialog>
        <MatDialog
            heading=format!("Add member in {}", props.room.name)
            dialog_link=&*invite_dialog_link
        >
            <MatTextField
                outlined=true
                required=true
                field_type=TextFieldType::Text
                label="Username"
                oninput=Callback::from(move |e: InputData| set_invitee_username(e.value))
            />

            <MatDialogAction action_type=ActionType::Primary action="add">
                <span onclick=add_member_callback>
                    <MatButton label="Add" />
                </span>
            </MatDialogAction>

            <MatDialogAction action_type=ActionType::Secondary action="cancel">
                <MatButton label="Close" />
            </MatDialogAction>
        </MatDialog>
    </>}
}
