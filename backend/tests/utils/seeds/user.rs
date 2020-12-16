use backend::auth::{create_jwt, BCRYPT_COST};
use backend::services::user as service;
use common::User;
use sqlx::PgConnection;

pub async fn create_user(conn: &mut PgConnection, username: &str, password: &str) -> User {
    let password = bcrypt::hash(password, BCRYPT_COST).expect("failed to hash");
    let user = User::new(username.to_string(), password);
    service::create(conn, user)
        .await
        .expect("failed to create user")
}

pub async fn create_authenticated_user(
    conn: &mut PgConnection,
    username: &str,
    password: &str,
) -> (User, String) {
    let user = create_user(conn, username, password).await;
    let token = create_jwt(&user).expect("failed to create jwt");

    (user, token.token)
}
