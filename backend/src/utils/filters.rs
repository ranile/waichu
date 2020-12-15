use crate::auth::parse_token;
use crate::utils::CustomRejection;
use common::User;
use http_api_problem::HttpApiProblem;
use serde::Deserialize;
use sqlx::PgPool;
use std::path::PathBuf;
use warp::http::StatusCode;
use warp::path::FullPath;
use warp::Filter;

pub fn json_body<T: for<'de> Deserialize<'de> + Send>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body  (and to reject huge payloads)
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn with_db(
    pool: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

pub fn ensure_authorized(
    pool: PgPool,
) -> impl Filter<Extract = (User,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization")
        .and(with_db(pool))
        .and_then(|token: String, db: PgPool| async move {
            let mut conn = db.acquire().await.map_err(|_e| {
                warp::reject::custom(CustomRejection(
                    HttpApiProblem::new("failed to acquire pool")
                        .set_status(StatusCode::INTERNAL_SERVER_ERROR),
                ))
            })?;

            let user = parse_token(&mut conn, &token).await.map_err(|e| {
                warp::reject::custom(CustomRejection(
                    HttpApiProblem::new(e.to_string())
                        .set_status(StatusCode::INTERNAL_SERVER_ERROR),
                ))
            })?;

            let user = match user {
                Some(user) => user,
                None => {
                    return Err(warp::reject::custom(CustomRejection(
                        HttpApiProblem::new("Invalid token").set_status(StatusCode::UNAUTHORIZED),
                    )));
                }
            };

            Ok(user)
        })
}

pub fn single_page_application(
    dist_dir: impl Into<PathBuf>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let dist_dir = dist_dir.into();

    let index_fallback = warp::path::full()
        .and(warp::fs::file(dist_dir.join("index.html")))
        .and_then(|p: FullPath, index| async move {
            if p.as_str().starts_with("/api") {
                Err(warp::reject())
            } else {
                Ok(index)
            }
        });
    warp::fs::dir(dist_dir).or(index_fallback)
}
