use crate::models::User;
use crate::{DbPool};
use uuid::Uuid;
use sqlx::postgres::PgQueryAs;
use sqlx::pool::PoolConnection;
use sqlx::PgConnection;

pub async fn insert(pool: &DbPool, user: User) -> sqlx::Result<User> {
    sqlx::query_as("
        insert into users(username, uuid, password)
        values ($1, $2, $3)
        returning *;
    ")
        .bind(user.username)
        .bind(user.uuid)
        .bind(user.password)
        .fetch_one(pool)
        .await
}

pub async fn get(pool: &mut PoolConnection<PgConnection>, query_uuid: Uuid) -> sqlx::Result<User> {
    sqlx::query_as("select * from users where uuid = $1")
        .bind(query_uuid)
        .fetch_one(pool)
        .await
}

pub async fn get_by_username(pool: &DbPool, query_username: String) -> sqlx::Result<User> {
    sqlx::query_as("select * from users where username = $1")
        .bind(query_username)
        .fetch_one(pool)
        .await
}
