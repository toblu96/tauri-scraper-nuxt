use super::store::AppState;
use axum::Router;
use std::sync::Arc;

pub mod files;
pub mod info;
pub mod logs;
pub mod settings;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .merge(info::routes())
        .merge(files::routes())
        .merge(logs::routes())
        .merge(settings::routes())
}
