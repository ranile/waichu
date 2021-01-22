use anyhow::Context;
use common::errors::ApiError;
use reqwasm::{Request, Method};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;
use web_sys::FormData;

const AUTHORIZATION: &str = "Authorization";

fn to_sentence_case(input: &str) -> String {
    input
        .split('.')
        .map(|it| {
            let it = it.trim();
            if !it.is_ascii() || it.is_empty() {
                return it.to_string();
            }
            let (head, tail) = it.split_at(1);
            head.to_uppercase() + tail + "."
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[derive(Debug, Clone)]
pub struct NoContent;

impl fmt::Display for NoContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server returned 204")
    }
}

pub async fn request<T: Serialize, R: DeserializeOwned>(
    url: impl Into<String>,
    method: Method,
    body: Option<&T>,
    form_data: Option<FormData>,
    auth_token: Option<&str>,
) -> anyhow::Result<R> {
    let url = &url.into();
    let mut builder = match method {
        Method::POST => Request::post(url)
            .body(serde_json::to_string(body.as_ref().unwrap())?)
            .header("Content-Type", "application/json"),
        Method::GET => Request::get(url),
        Method::PUT => Request::put(url),
        _ => unreachable!(),
    };

    if let Some(form_data) = form_data {
        builder = builder.body(form_data);
    }

    if let Some(token) = auth_token {
        if !token.is_empty() {
            builder = builder.header(AUTHORIZATION, token);
        }
    }

    let resp = builder.send().await?;
    let status = resp.status();
    if 300 > status && status >= 200 {
        let res = resp.json::<R>().await;
        if status == 204 {
            res.context(NoContent)
        } else {
            Ok(res?)
        }
    } else {
        let error = resp.json::<ApiError>().await?;
        Err(anyhow::anyhow!("{}", to_sentence_case(&error.message)))
    }
}

#[macro_export]
macro_rules! request {
    (method = $method:ident, url = $url:expr) => {
        crate::services::request::request(
            $url,
            ::reqwasm::Method::$method,
            ::std::option::Option::None,
            ::std::option::Option::None,
            ::std::option::Option::None,
        )
    };
    (method = $method:ident, url = $url:expr, body = $body:expr) => {
        crate::services::request::request(
            $url,
            ::reqwasm::Method::$method,
            ::std::option::Option::Some($body),
            ::std::option::Option::None,
            ::std::option::Option::None,
        )
    };

    (method = $method:ident, url = $url:expr, token = $token:expr) => {
        crate::services::request::request(
            $url,
            ::reqwasm::Method::$method,
            ::std::option::Option::Some(&"".to_string()),
            ::std::option::Option::None,
            ::std::option::Option::Some($token),
        )
    };
    (method = $method:ident, url = $url:expr, body = $body:expr, token = $token:expr) => {
        crate::services::request::request(
            $url,
            ::reqwasm::Method::$method,
            ::std::option::Option::Some($body),
            ::std::option::Option::None,
            ::std::option::Option::Some($token),
        )
    };
    (method = $method:ident, url = $url:expr, form_data = $form_data:expr, token = $token:expr) => {
        crate::services::request::request(
            $url,
            ::reqwasm::Method::$method,
            ::std::option::Option::Some(&"".to_string()),
            ::std::option::Option::Some($form_data),
            ::std::option::Option::Some($token),
        )
    };
}
