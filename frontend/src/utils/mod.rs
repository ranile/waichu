use std::rc::Rc;
use yew_functional::{use_context, use_state};

#[allow(clippy::rc_buffer)] // this needs to be Rc so I'm not cloning it a billion times
pub fn use_token() -> Rc<String> {
    let token = use_context::<Rc<Option<String>>>().unwrap();
    let token = ((**token).as_ref().unwrap()).clone();
    Rc::new(token)
}
