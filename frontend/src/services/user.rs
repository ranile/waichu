use crate::utils::js_to_anyhow;
use common::Asset;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FormData, Headers, Request, RequestInit, Response};
use yew::utils::window;

pub async fn update_avatar(token: &str, file: File) -> anyhow::Result<Asset> {
    let url = "/api/users/me/avatar";

    let form_data = FormData::new().map_err(js_to_anyhow)?;

    form_data
        .append_with_blob("file", &file)
        .map_err(js_to_anyhow)?;

    let mut init = RequestInit::new();
    init.body(Some(&form_data));
    init.method("PUT");

    let headers = Headers::new().map_err(js_to_anyhow)?;
    headers
        .append("Authorization", token)
        .map_err(js_to_anyhow)?;

    init.headers(&headers);

    let request = Request::new_with_str_and_init(url, &init).map_err(js_to_anyhow)?;

    let promise = window().fetch_with_request(&request);
    let resp = JsFuture::from(promise).await.map_err(js_to_anyhow)?;
    assert!(resp.is_instance_of::<Response>());
    let resp: Response = resp.dyn_into().unwrap();
    let json = JsFuture::from(resp.json().map_err(js_to_anyhow)?)
        .await
        .map_err(js_to_anyhow)?;

    let asset: Asset = json.into_serde()?;

    Ok(asset)
}
