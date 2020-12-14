use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::MediaQueryList;
use weblog::wasm_bindgen::JsCast;
use yew_functional::{use_context, use_effect, use_state};

#[allow(clippy::rc_buffer)] // this needs to be Rc so I'm not cloning it a billion times
pub fn use_token() -> Rc<String> {
    let token = use_context::<Rc<Option<String>>>().unwrap();
    let token = ((**token).as_ref().unwrap()).clone();
    Rc::new(token)
}

pub fn use_on_mobile_listener() -> bool {
    let (is_on_mobile, set_is_on_mobile) = use_state(|| {
        yew::utils::window()
            .match_media("(max-width: 600px)")
            .unwrap()
            .unwrap()
            .matches()
    });

    use_effect(move || {
        let media_query_list = yew::utils::window()
            .match_media("(max-width: 600px)")
            .unwrap()
            .unwrap();

        let function =
            Closure::wrap(
                Box::new(move |event: MediaQueryList| set_is_on_mobile(event.matches()))
                    as Box<dyn FnMut(MediaQueryList)>,
            );

        let _ = media_query_list
            .add_listener_with_opt_callback(Some(function.as_ref().unchecked_ref()));

        move || {
            media_query_list
                .remove_listener_with_opt_callback(Some(function.as_ref().unchecked_ref()))
                .unwrap()
        }
    });

    *is_on_mobile
}
