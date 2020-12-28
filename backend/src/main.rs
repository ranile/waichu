use backend::utils::single_page_application;
use backend::{
    balanced_or_tree, debug_boxed, setup_assets_directory, setup_database, setup_logger,
};
use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    setup_logger().expect("unable to setup logger");

    setup_assets_directory().await.unwrap();

    let pool = setup_database().await.expect("unable to setup database");

    let dist_dir = env::var("DIST_DIR").expect("`DIST_DIR` isn't set");

    let api = backend::api(pool.clone());
    let spa = single_page_application(dist_dir);

    let routes = balanced_or_tree!(api, spa);
    // .with(warp::compression::gzip());

    #[cfg(debug_assertions)]
    let routes = routes.with(
        warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_headers(vec!["authorization"]),
    );

    let port = env::var("PORT")
        .map(|it| it.parse().expect("invalid port"))
        .unwrap_or(9090);

    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        });

    log::info!("running server on http://{}/", addr);
    server.await;
}
