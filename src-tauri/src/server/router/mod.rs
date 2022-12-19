use axum::Router;

mod info;

pub fn routes() -> Router {
    Router::new().merge(info::routes())
}
