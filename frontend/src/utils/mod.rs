use chrono::{DateTime, Datelike, Local, Utc};
use common::Asset;
use std::rc::Rc;
use yew_functional::use_context;

pub const PROFILE_PICTURE_URL: &str = "https://i.redd.it/j04fpwy2ea261.png";

#[allow(clippy::rc_buffer)] // this needs to be Rc so I'm not cloning it a billion times
pub fn use_token() -> Rc<String> {
    let token = use_context::<Rc<Option<String>>>().unwrap();
    let token = ((**token).as_ref().unwrap()).clone();
    Rc::new(token)
}

// todo profile its performance
/*pub fn _use_on_mobile_listener() -> bool {
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
*/
pub fn is_on_mobile() -> bool {
    yew::utils::window()
        .match_media("(max-width: 600px)")
        .unwrap()
        .unwrap()
        .matches()
}

pub fn format_time(time: &DateTime<Utc>) -> String {
    let local = time.with_timezone(&Local).naive_local();
    let now = Local::now().naive_local();

    let is_today = local.date() == now.date();
    let is_yesterday = local.date() == now.date().pred();
    let in_current_week = local.iso_week() == now.iso_week();

    let mut output = if is_today {
        "Today".to_string()
    } else if is_yesterday {
        "Yesterday".to_string()
    } else if in_current_week {
        local.date().format("%A").to_string()
    } else {
        "".to_string()
    };

    if !in_current_week {
        output += local.date().format("%e %B").to_string().trim();
        if local.year() != now.year() {
            output += &*format!(" {}", local.date().format("%Y"));
        }
    }

    output += &*format!(", {}", local.time().format("%l:%M %p"));

    output.trim().to_string()
}

pub fn asset_url(asset: Option<&Asset>) -> String {
    asset
        .map(|asset| format!("http://localhost:9090/api/assets/{}", asset.uuid))
        .unwrap_or_else(|| PROFILE_PICTURE_URL.to_string())
}
