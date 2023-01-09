use std::sync::Arc;

use axum::{
    routing::{get, Router},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::server::store::AppState;

/// exports all routes from this module as router
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/info", get(get_info))
}

/// Get information about application
///
/// Returns all relevant information for this application.
#[utoipa::path(
        get,
        context_path = "/api",
        path = "/info",
        tag = "info",
        responses(
            (status = 200, description = "List all information successfully", body = Info)
        )
    )]
pub async fn get_info() -> Json<Info> {
    let info = Info {
        up: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(info)
}

/// Application information schema.
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Info {
    /// Application state
    up: bool,
    /// Current application version
    #[schema(example = "v0.0.1")]
    version: String,
}
