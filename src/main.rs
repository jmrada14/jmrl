use axum::{response::Redirect, Router};
use tower_http::services::ServeDir;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route(
            "/blog",
            axum::routing::get(|| async { Redirect::permanent("/blog.html") }),
        )
        .nest_service("/", ServeDir::new("assets"));

    Ok(router.into())
}
