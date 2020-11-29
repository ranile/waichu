use std::rc::Rc;
use yew_functional::use_context;

pub fn use_token() -> Rc<String> {
    let mut token = use_context::<Rc<Option<String>>>().unwrap();

    Rc::new((**token).as_ref().expect("No token set").as_str().to_owned())
}
