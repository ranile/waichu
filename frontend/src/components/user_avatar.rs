use crate::AppState;
use common::User;
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;
use yew::web_sys::MouseEvent;
use yew_functional::{function_component, use_state};
use yew_material::{
    dialog::{ActionType, MatDialogAction},
    MatButton, MatDialog, MatIconButton, WeakComponentLink,
};
use yew_state::{SharedHandle, SharedState};

pub const PROFILE_PICTURE_URL: &str = "https://i.redd.it/j04fpwy2ea261.png";

#[derive(Clone, Properties, PartialEq)]
pub struct UserAvatarProps {
    pub user: User,
    #[prop_or_default]
    pub show_details_on_click: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    handle: SharedHandle<AppState>,
}

impl SharedState for UserAvatarProps {
    type Handle = SharedHandle<AppState>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}

#[function_component(UserAvatar)]
pub fn user_avatar(props: &UserAvatarProps) -> Html {
    let (dialog_link, _) = use_state(WeakComponentLink::<MatDialog>::default);

    let onclick = {
        let dialog_link = Rc::clone(&dialog_link);
        Callback::from(move |_| {
            dialog_link.show();
        })
    };

    let logout_button = match props.handle.state().me.as_ref() {
        Some(user) if user.uuid == props.user.uuid => html! {
            <MatButton label="Sign out" icon="exit_to_app" />
        },
        _ => html!(),
    };

    let logout_callback = {
        let dialog_link = Rc::clone(&dialog_link);
        let reset_callback = props.handle.reduce_callback(move |state| {
            state.token = None;
            state.rooms = Rc::new(RefCell::new(vec![]));
            state.me = None;
        });

        Callback::from(move |_| {
            weblog::console_log!("logout");
            let window = yew::utils::window();
            window.local_storage().unwrap().unwrap().clear().unwrap();

            dialog_link.close();
            reset_callback.emit(());
            window.location().reload().unwrap();
        })
    };

    html! {<>
        <span class="user-avatar" onclick=onclick>
            <MatIconButton>
                <img src=PROFILE_PICTURE_URL />
            </MatIconButton>
        </span>
        <span class="user-profile-dialog-container">
            <MatDialog
                dialog_link=&*dialog_link
                // onclosed=on_dialog_closed TODO fix yew-material coz ya boi an idiot
            >
                <section class="profile-dialog-container">
                    <img src=PROFILE_PICTURE_URL />
                    <span>{ &props.user.username }</span>
                    <span onclick=logout_callback>
                        { logout_button }
                    </span>
                </section>

                <MatDialogAction action_type=ActionType::Secondary action="cancel">
                    <MatButton label="Close" />
                </MatDialogAction>
            </MatDialog>
        </span>
    </>}
}
