use crate::utils::requests;
use std::collections::HashMap;
use crate::utils::requests::{parse_resp_as_json, FetchResult};
use crate::models::User;
use uuid::Uuid;
use crate::app::APP_STATE;

fn get_from_cache(uuid: Uuid) -> Option<User> {
    APP_STATE.with(|refcell| {
        let state = refcell.borrow();
        let user = state.users.iter().find(|u| u.uuid == uuid);
        match user {
            Some(user) => Some(user.clone()),
            None => None
        }
    })
}

fn cache_user(user: User) -> bool {
    APP_STATE.with(|refcell| {
        let mut state = refcell.borrow_mut();
        state.users.insert(user)
    })
}

pub async fn get_me(token: String) -> FetchResult<User> {
    let mut headers = HashMap::new();
    headers.insert("Authorization", token);
    let resp = requests::get(format!("/api/users/@me"), headers).await?;
    parse_resp_as_json::<User>(resp).await
}

pub async fn get(uuid: Uuid) -> FetchResult<User> {
    match get_from_cache(uuid.clone()) {
        Some(user) => Ok(user),
        None => {
            let resp = requests::get(format!("/api/users/{}", uuid), HashMap::new()).await?;
            let user = parse_resp_as_json::<User>(resp).await?;
            cache_user(user.clone());
            Ok(user)
        }
    }
}
