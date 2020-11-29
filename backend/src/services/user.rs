use crate::services;
use common::User;
use sqlx::types::Uuid;
use sqlx::PgConnection;

pub async fn create(db: &mut PgConnection, user: User) -> anyhow::Result<User> {
    let User {
        username,
        uuid,
        password,
        ..
    } = user;

    Ok(sqlx::query_as!(
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
    .await?)
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
