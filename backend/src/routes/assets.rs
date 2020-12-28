use crate::utils::{with_db, with_transaction, ASSETS_PATH};
use crate::{services, value_or_404};
use sqlx::types::Uuid;
use sqlx::PgPool;
use warp::http::header::CONTENT_TYPE;
use warp::http::{HeaderValue, Response};
use warp::hyper::Body;
use warp::Filter;
use warp::Reply;

async fn serve_assets(uuid: Uuid, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    with_transaction(pool, move |conn| {
        Box::pin(async move {
            let assets_path = ASSETS_PATH.lock().await;
            let assets_path = assets_path.as_ref().unwrap();
            let asset = value_or_404!(services::asset::get(conn, uuid).await?, "asset not found");

            let bytes = tokio::fs::read(format!("{}/{}.jpeg", assets_path, asset.uuid)).await?;

            let mut res = Response::new(Body::from(bytes));
            res.headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_str("image/jpeg").unwrap());

            Ok(res)
        })
    })
    .await
}

pub fn routes(
    pool: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("assets" / Uuid)
        .and(warp::get())
        .and(with_db(pool))
        .and_then(serve_assets)
}
