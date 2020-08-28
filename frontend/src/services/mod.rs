use crate::app::APP_STATE;

pub mod message_service;
pub mod auth_service;
pub mod user_service;
pub mod rooms_service;

pub fn get_token() -> Option<String> {
    APP_STATE.with(|f| {
        let state = f.borrow();
        state.token.clone()
    })
}
