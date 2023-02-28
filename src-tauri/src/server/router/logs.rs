use crate::server::store::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// exports all routes from this module as router
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/logs", get(logs_index))
}

/// Show application broker settings.
///
/// Returns the configured application broker settings.
#[utoipa::path(
        get,
        context_path = "/api",
        path = "/logs",
        tag = "logs",
        responses(
            (status = 200, description = "List log entries successfully", body = [Logs]),
        )
    )]
pub async fn logs_index(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let logs: Vec<Logs> = Vec::new();
    (StatusCode::OK, Json(logs))
}

/// Logs schema.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
pub struct Logs {
    /// Timestamp of the log entry
    #[schema(example = "2023-02-28T07:11:07.440591800+01:00")]
    time: String,
    /// Log message
    message: String,
    /// Module path specifier
    #[schema(example = "app::server")]
    module_path: String,
    /// Path to file
    #[schema(example = "src\\server\\mod.rs")]
    file: String,
    // Line from which log got triggered
    line: u16,
    /// Log level
    level: String,
    /// Log target
    target: String,
    /// Current thread
    thread: String,
    /// Current thread id
    thread_id: u16,
}
