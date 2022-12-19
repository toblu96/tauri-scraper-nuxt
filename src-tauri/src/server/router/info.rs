use axum::routing::{get, Router};

pub fn routes() -> Router {
    Router::new().route("/info", get(get_info).post(post_info))
}

async fn get_info() -> &'static str {
    "Hello, World!"
}
async fn post_info() -> &'static str {
    "Hello, World!"
}
