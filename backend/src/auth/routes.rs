use crate::auth::jwt::create_jwt;
use crate::auth::BCRYPT_COST;
use crate::utils::{json_body, with_db, with_transaction, json_with_status};
use crate::{bail_if_err, bail_if_err_or_404, services};
use common::payloads::Credentials;
use common::User;
use http_api_problem::HttpApiProblem;
use sqlx::PgPool;
use warp::http::StatusCode;
use warp::{reply, Filter, Reply};

fn hash_password(password: &str) -> anyhow::Result<String> {
    Ok(bcrypt::hash(password, BCRYPT_COST)?)
}

fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    Ok(bcrypt::verify(password, hash)?)
}

async fn signup(
    pool: PgPool,
    credentials: Credentials,
) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |transaction| {
        Box::pin(async move {
            let password = (hash_password(&credentials.password))?;
            let user = User::new(credentials.username, password);

            let user = services::user::create(&mut *transaction, user).await?;

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

    let user = bail_if_err_or_404!(
        services::user::get_by_username(&mut db, &credentials.username).await,
        "Invalid username or password"
    );

    Ok(
        if bail_if_err!(verify_password(&credentials.password, &user.password)) {
            let token = bail_if_err!(create_jwt(&user));

            reply::json(&token).into_response()
        } else {
            let status = StatusCode::UNAUTHORIZED;
            reply::with_status(
                warp::reply::json(
                    &HttpApiProblem::new("Invalid username or password").set_status(status),
                ),
                status,
            )
            .into_response()
        },
    )
}

pub fn auth(
    pool: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let signup_route = warp::path!("api" / "auth" / "signup")
        .and(warp::post())
        .and(with_db(pool.clone()))
        .and(json_body::<Credentials>())
        .and_then(signup);

    let signin_route = warp::path!("api" / "auth" / "signin")
        .and(warp::post())
        .and(with_db(pool))
        .and(json_body::<Credentials>())
        .and_then(signin);

    signup_route.or(signin_route)
}
