use crate::utils::asset_url;
use common::User;
use yew::prelude::*;
use yew_functional::{function_component, use_effect, use_state};
use yew_material::{
    dialog::{ActionType, MatDialogAction},
    MatButton, MatDialog, MatIconButton,
};

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
            if span.is_some() {
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
        <UserProfileDialog user=&props.user open=*open onclosed=on_dialog_closed />
    </>}
}

#[derive(Clone, Properties, PartialEq)]
pub struct UserProfileDialogProps {
    pub user: User,
    pub open: bool,
    pub onclosed: Callback<String>,
}

#[function_component(UserProfileDialog)]
pub fn user_profile_dialog(props: &UserProfileDialogProps) -> Html {
    html! {
        <span class="user-profile-dialog-container">
            <MatDialog
                onclosed=&props.onclosed
                open=props.open
            >
                <section class="profile-dialog-container">
                    <img src=asset_url(props.user.avatar.as_ref()) />
                    <span>{ &props.user.username }</span>
                </section>

                <MatDialogAction action_type=ActionType::Secondary action="cancel">
                    <MatButton label="Close" />
                </MatDialogAction>
            </MatDialog>
        </span>
    }
}
