use crate::services::request::request;
use crate::utils::js_to_anyhow;
use common::Asset;
use reqwasm::Method;
use web_sys::{File, FormData};

pub async fn update_avatar(token: &str, file: File) -> anyhow::Result<Asset> {
    let url = "/api/users/me/avatar";

    let form_data = FormData::new().map_err(js_to_anyhow)?;

    form_data
        .append_with_blob("file", &file)
        .map_err(js_to_anyhow)?;

    request(
        url,
        Method::PUT,
        Some(&"".to_string()),
        Some(form_data),
        Some(token),
    )
    .await
}
