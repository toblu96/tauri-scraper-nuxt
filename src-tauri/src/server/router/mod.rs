use axum::Router;

pub mod files;
pub mod info;
pub mod settings;

pub fn routes() -> Router {
    Router::new()
        .merge(info::routes())
        .merge(files::routes())
        .merge(settings::routes())
}
