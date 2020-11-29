pub mod auth;
pub mod room;

fn url(route: &str) -> String {
    // let base = "http://localhost:9090/"
    // let base = "http://localhost:9090";
    let base = yew::utils::window().location().origin().unwrap();
    format!("{}{}", base, route)
}
