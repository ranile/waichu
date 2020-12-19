use crate::auth::jwt::create_jwt;
use crate::auth::BCRYPT_COST;
use crate::services::user::UserAlreadyExists;
use crate::utils::{json_body, json_with_status, with_db, with_transaction};
use crate::{bail_if_err, services};
use common::errors::ApiError;
use common::payloads::Credentials;
use common::User;
use http_api_problem::HttpApiProblem;
use sqlx::PgPool;
use warp::http::StatusCode;
use warp::{reply, Filter, Reply};

fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    Ok(bcrypt::verify(password, hash)?)
}

async fn signup(
    pool: PgPool,
    credentials: Credentials,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |transaction| {
        Box::pin(async move {
            let password = bcrypt::hash(credentials.password, BCRYPT_COST)?;
            let user = User::new(credentials.username, password);

            let user = services::user::create(&mut *transaction, user).await;

            let user = match user {
                Ok(user) => user,
                Err(err) => {
                    let api_error = match err.downcast::<UserAlreadyExists>() {
                        Ok(username) => ApiError::new_with_message_and_status(
                            &username.to_string(),
                            StatusCode::BAD_REQUEST,
                        ),
                        Err(e) => ApiError::new_with_message(&e.to_string()),
                    };
                    return Ok(api_error.into_response());
                }
            };

            let token = create_jwt(&user)?;

            Ok(json_with_status(StatusCode::CREATED, &token))
        })
    })
    .await
}

async fn signin(
    pool: PgPool,
    credentials: Credentials,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db = bail_if_err!(pool.acquire().await.map_err(anyhow::Error::from));

    let user = bail_if_err!(services::user::get_by_username(&mut db, &credentials.username).await);

    let user = match user {
        Some(user) => user,
        None => {
            let status = StatusCode::UNAUTHORIZED;
            return Ok(json_with_status(
                status,
                &HttpApiProblem::new("Invalid username or password").set_status(status),
            ));
        }
    };

    Ok(
        if bail_if_err!(verify_password(&credentials.password, &user.password)) {
            let token = bail_if_err!(create_jwt(&user));

            reply::json(&token).into_response()
        } else {
            let status = StatusCode::UNAUTHORIZED;
            json_with_status(
                status,
                &HttpApiProblem::new("Invalid username or password").set_status(status),
            )
        },
    )
}

pub fn auth(
    pool: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(with_db(pool.clone()))
        .and(json_body::<Credentials>())
        .and_then(signup);

    let signin_route = warp::path!("auth" / "signin")
        .and(warp::post())
        .and(with_db(pool))
        .and(json_body::<Credentials>())
        .and_then(signin);

    signup_route.or(signin_route)
}
