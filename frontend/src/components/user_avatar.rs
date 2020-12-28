use crate::utils::asset_url;
use crate::AppState;
use common::User;
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;
use yew_functional::{function_component, use_state, use_effect};
use yew_material::{
    dialog::{ActionType, MatDialogAction},
    MatButton, MatDialog, MatIconButton, WeakComponentLink,
};
use yew_state::{SharedHandle, SharedState, SharedStateComponent};

#[derive(Clone, Properties, PartialEq)]
pub struct UserAvatarProps {
    pub user: User,
    #[prop_or(true)]
    pub show_details_on_click: bool,
    #[prop_or(None)]
    pub onclick: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub onload: Callback<NodeRef>,
}

#[function_component(UserAvatar)]
pub fn user_avatar(props: &UserAvatarProps) -> Html {
    let (open, set_open) = use_state(|| false);

    let onclick = if props.show_details_on_click {
        {
            let set_open = set_open.clone();
            Callback::from(move |_| {
                set_open(true);
            })
        }
    } else {
        props.onclick.clone().unwrap_or_default()
    };

    let on_dialog_closed = Callback::from(move |_| set_open(false));
    let (node_ref, _) = use_state(NodeRef::default);

    let _ = {
        let node_ref = node_ref.clone();
        let onload_prop = props.onload.clone();

        use_effect(move || {
            let span = yew::utils::document().get_element_by_id("user-avatar-container");
            if let Some(span) = span {
                onload_prop.emit((*node_ref).clone())
            }

            || ()
        })
    };

    html! {<>
        <span class="user-avatar" onclick=onclick id="user-avatar-container" ref=(*node_ref).clone()>
            <MatIconButton>
                <img src=asset_url(props.user.avatar.as_ref()) />
            </MatIconButton>
        </span>
        <SharedStateComponent<UserProfileDialog> user=&props.user open=*open onclosed=on_dialog_closed />
    </>}
}

#[derive(Clone, Properties, PartialEq)]
pub struct UserProfileDialogProps {
    pub user: User,
    pub open: bool,
    #[prop_or_default]
    pub handle: SharedHandle<AppState>,
    pub onclosed: Callback<()>,
}

impl SharedState for UserProfileDialogProps {
    type Handle = SharedHandle<AppState>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}

#[function_component(UserProfileDialog)]
pub fn user_profile_dialog(props: &UserProfileDialogProps) -> Html {
    let (dialog_link, _) = use_state(WeakComponentLink::<MatDialog>::default);
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

    html! {
        <span class="user-profile-dialog-container">
            <MatDialog
                dialog_link=&*dialog_link
                onclosed=&props.onclosed
                open=props.open
                // onclosed=on_dialog_closed TODO fix yew-material coz ya boi an idiot
            >
                <section class="profile-dialog-container">
                    <img src=asset_url(props.user.avatar.as_ref()) />
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
    }
}
