use common::errors::ApiError;

pub fn from_anyhow(e: anyhow::Error) -> ApiError {
    let e = match e.downcast::<ApiError>() {
        Ok(error) => return error,
        Err(e) => e,
    };
    ApiError::new_with_message(&e.to_string())
}
