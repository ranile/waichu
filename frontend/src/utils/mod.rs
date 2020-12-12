use std::rc::Rc;
use yew_functional::{use_context, use_state};

#[allow(clippy::rc_buffer)] // this needs to be Rc so I'm not cloning it a billion times
pub fn use_token() -> Rc<String> {
    let (token, _) = use_state(|| {
        let token = use_context::<Rc<Option<String>>>().unwrap();
        (**token)
            .as_ref()
            .expect("No token set")
            .as_str()
            .to_owned()
    });

    token
}
