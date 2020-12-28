pub mod asset;
pub mod message;
pub mod room;
pub mod user;

fn optional_value_or_err<T>(value: Result<T, sqlx::Error>) -> anyhow::Result<Option<T>> {
    match value {
        Ok(value) => Ok(Some(value)),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(anyhow::Error::from(err)),
        },
    }
}
