use crate::services;
use common::User;
use serde::export::Formatter;
use sqlx::postgres::PgDatabaseError;
use sqlx::types::Uuid;
use sqlx::PgConnection;
use std::fmt;

#[derive(Debug, Clone)]
pub(crate) struct UserAlreadyExists(pub(crate) String);

impl fmt::Display for UserAlreadyExists {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "username \"{}\" already exists", self.0)
    }
}

pub async fn create(db: &mut PgConnection, user: User) -> anyhow::Result<User> {
    let User {
        username,
        uuid,
        password,
        ..
    } = user;

    let result = sqlx::query_as!(
        User,
        "
            insert into users(username, uuid, password)
            values ($1, $2, $3)
            returning *;
        ",
        username,
        uuid,
        password
    )
    .fetch_one(db)
    .await;

    match result {
        Ok(user) => Ok(user),
        Err(sqlx::Error::Database(db_error)) => {
            let db_error = db_error.downcast::<PgDatabaseError>();

            Err(
                // duplicate key value violates unique constraint "users_username_key"
                if db_error.code() == "23505" && db_error.message().contains("users_username_key") {
                    anyhow::Error::from(db_error).context(UserAlreadyExists(username))
                } else {
                    anyhow::Error::from(db_error)
                },
            )
        }
        Err(e) => Err(anyhow::Error::from(e)),
    }
}

pub async fn get(db: &mut PgConnection, uuid: Uuid) -> anyhow::Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        "
            select *
            from users
            where uuid = $1;
        ",
        uuid
    )
    .fetch_one(db)
    .await;

    services::optional_value_or_err(user)
}

pub async fn get_by_username(
    db: &mut PgConnection,
    username: &str,
) -> anyhow::Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        "
            select *
            from users
            where username = $1;
        ",
        username
    )
    .fetch_one(db)
    .await;

    services::optional_value_or_err(user)
}
