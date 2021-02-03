use crate::websocket;
use common::websocket::{MessagePayload, OpCode};
use common::{Asset, User};
use serde::export::Formatter;
use sqlx::postgres::PgDatabaseError;
use sqlx::types::Uuid;
use sqlx::PgConnection;
use std::fmt;
use std::sync::Arc;
use tracing::instrument;
use tracing::{debug, error};

macro_rules! get {
    ($db:ident, $sel_col:expr, $value:expr) => {{
        debug!("fetching user");
        let res = sqlx::query!(
            r#"
select users.username as user_username,
       users.uuid as user_uuid,
       users.password as user_password,
       users.created_at as user_created_at,
       users.avatar as "user_avatar?",
       assets.uuid as "asset_uuid?",
       assets.created_at as "asset_created_at?"
from users
         left join assets on users.avatar = assets.uuid
where "# + $sel_col
                + " = $1;",
            $value
        )
        .fetch_optional($db)
        .await;

        let user = match res {
            Ok(Some(res)) => Ok(Some(User {
                uuid: res.user_uuid,
                username: res.user_username,
                password: res.user_password,
                created_at: res.user_created_at,
                avatar: match res.user_avatar {
                    Some(_) => Some(Asset {
                        uuid: res.asset_uuid.unwrap(),
                        bytes: Default::default(),
                        created_at: res.asset_created_at.unwrap(),
                    }),
                    None => None,
                },
            })),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("error while fetching user: {}", e);
                Err(anyhow::anyhow!(e))
            }
        };

        user
    }};
}

#[derive(Debug, Clone)]
pub(crate) struct UserAlreadyExists(pub(crate) String);

impl fmt::Display for UserAlreadyExists {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "username \"{}\" already exists", self.0)
    }
}

#[instrument]
pub async fn create(db: &mut PgConnection, user: User) -> anyhow::Result<User> {
    let User {
        username,
        uuid,
        password,
        ..
    } = user;

    debug!("creating user");

    let result = sqlx::query!(
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
        Ok(res) => {
            let user = User {
                uuid: res.uuid,
                username: res.username,
                password: res.password,
                created_at: res.created_at,
                avatar: None,
            };
            debug!("created user: uuid: {}", user.uuid);
            Ok(user)
        }
        Err(sqlx::Error::Database(db_error)) => {
            let db_error = db_error.downcast::<PgDatabaseError>();

            Err(
                // duplicate key value violates unique constraint "users_username_key"
                if db_error.code() == "23505" && db_error.message().contains("users_username_key") {
                    debug!("error creating user: user already exists");
                    anyhow::Error::from(db_error).context(UserAlreadyExists(username))
                } else {
                    error!("error creating user: {}", db_error);
                    anyhow::Error::from(db_error)
                },
            )
        }
        Err(e) => Err(anyhow::Error::from(e)),
    }
}

#[instrument]
pub async fn get(db: &mut PgConnection, uuid: Uuid) -> anyhow::Result<Option<User>> {
    get!(db, "users.uuid", uuid)
}

#[instrument]
pub async fn get_by_username(
    db: &mut PgConnection,
    username: &str,
) -> anyhow::Result<Option<User>> {
    get!(db, "username", username)
}

pub async fn update(db: &mut PgConnection, user: User) -> anyhow::Result<User> {
    let User {
        uuid,
        username,
        avatar,
        ..
    } = user;

    let avatar = avatar.map(|it| it.uuid);

    sqlx::query_as!(
        User,
        "
update users
set username   = $1,
    avatar     = $2
where uuid = $3;
        ",
        username,
        avatar,
        uuid,
    )
    .execute(&mut *db)
    .await?;

    let new_user = get(db, uuid).await.map(|it| it.unwrap())?;

    websocket::send_message(
        Arc::new(MessagePayload {
            op: OpCode::UserUpdate,
            data: new_user.clone(),
        }),
        |uuid| uuid == new_user.uuid,
    )
    .await;

    Ok(new_user)
}
