use serde::{Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use std::fmt::{Formatter, Error, Debug};

pub type FetchResult<T> = Result<T, FetchError>;

/// Something wrong has occurred while fetching an external resource.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    err: JsValue,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Debug::fmt(&self.err, f)
    }
}

impl std::error::Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        FetchError { err: value }
    }
}

/// Make a `GET` request
pub async fn get(url: String, headers: HashMap<&'static str, String>) -> Result<Response, FetchError> {
    fetch("GET", url, headers, None).await
}

/// Make a `POST` request
pub async fn post<TBody: Serialize>(url: &str, json: TBody, headers: HashMap<&'static str, String>,) -> Result<Response, FetchError> {
    let mut headers = headers;
    if !headers.contains_key("Content-Type") {
        headers.insert("Content-Type", "application/json".to_string());
    }

    let body = serde_json::to_string(&json).unwrap();
    fetch("POST", url.to_string(), headers, Some(body)).await
}

async fn fetch(method: &'static str, url: String, headers: HashMap<&'static str, String>, body: Option<String>) -> Result<Response, FetchError> {
    let mut opts = RequestInit::new();
    opts.method(method)
        .mode(RequestMode::Cors);

    if let Some(body) = body {
        let js_val = JsValue::from(&body);
        opts.body(Some(&js_val));
    }

    let request = Request::new_with_str_and_init(&url, &opts)?;

    // Add headers
    let req_headers = request.headers();
    req_headers.set("Accept", "application/json")?;
    for (header, value) in headers {
        req_headers.set(header, &value)?;
    }

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into()?;
    Ok(resp)
}

pub async fn parse_resp_as_json<TRet: DeserializeOwned>(resp: Response) -> Result<TRet, FetchError> {

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Use serde to parse the JSON into a struct.
    let branch_info: TRet = json.into_serde().unwrap();

    Ok(branch_info)
}

