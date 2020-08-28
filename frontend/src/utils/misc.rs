use wasm_bindgen_futures::spawn_local;
use std::future::Future;
use yew::{Component, ComponentLink};

pub fn send_future<COMP: Component, F>(link: ComponentLink<COMP>, future: F)
    where
        F: Future<Output=COMP::Message> + 'static,
{
    spawn_local(async move {
        link.send_message(future.await);
    });
}
