use axum::routing::{get, Router};

/// exports all routes from this module as router
pub fn routes() -> Router {
    Router::new().route("/info", get(get_info))
}

/// Get information about application
///
/// Returns all relevant information for this application.
#[utoipa::path(
        get,
        path = "/info",
        tag = "info",
        responses(
            (status = 200, description = "List all information successfully")
        )
    )]
pub async fn get_info() -> &'static str {
    "Hello, World! Some other thest data for OpenAPI docs."
}
