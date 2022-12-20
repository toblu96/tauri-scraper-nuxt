use axum::Router;

pub mod info;

pub fn routes() -> Router {
    Router::new().merge(info::routes())
}
