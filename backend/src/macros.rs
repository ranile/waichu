pub const ANYHOW_RESULT_TYPE_NAME: &str = "anyhow::Error>";
pub const RESULT_TYPE_NAME: &str = "core::result::Result";

#[macro_export]
macro_rules! setup_rejection {
    ($err:ident $code_ident: ident $message: ident $($path:path, $code:expr);*) => {
        if let Some(_) = Option::<&str>::None {}
        $(
            else if let Some(e) = $err.find::<$path>() {
                $code_ident = $code;
                $message = e.to_string();
            }
        )+
    };
}

#[macro_export]
macro_rules! bail_if_err {
    ($res:expr) => {{
        let result = $res
            .map_err(crate::utils::from_anyhow)
            .map_err(|p| crate::utils::problem_to_reply(p).into_response());

        match result {
            Ok(value) => value,
            Err(e) => return Ok(e),
        }
    }};
}

#[macro_export]
macro_rules! value_or_404 {
    ($expr:expr, $message:expr) => {{
        match $expr {
            Some(value) => value,
            None => {
                return Ok(warp::reply::with_status(
                    warp::reply::json(
                        &http_api_problem::HttpApiProblem::new($message)
                            .set_status(warp::http::StatusCode::NOT_FOUND),
                    ),
                    warp::http::StatusCode::NOT_FOUND,
                )
                .into_response())
            }
        }
    }};
    ($expr:expr) => {{
        crate::value_or_404!($expr, "Requested resource not found")
    }};
}

#[macro_export]
macro_rules! bail_if_err_or_404 {
    ($res:expr, $message:expr) => {{
        let value = crate::bail_if_err!($res);
        let value = crate::value_or_404!(value, $message);
        value
    }};
    ($res:expr) => {{
        let value = crate::bail_if_err!($res);
        let value = crate::value_or_404!(value);
        value
    }};
}
