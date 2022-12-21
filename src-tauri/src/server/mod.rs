use axum::{response::Redirect, routing::get, Router};
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// import routes
mod router;

pub async fn start(port: u16) {
    // openapi things
    #[derive(OpenApi)]
    #[openapi(
        paths(
            router::info::get_info,
            router::files::files_index,
            router::files::files_create,
            router::files::files_update,
            router::files::files_delete,
        ),
        components(
            schemas(router::info::Info, router::files::File, router::files::FileUpdateParams, router::files::FileCreateParams)
        ),
        tags(
            (name = "info", description = "Information about this application"),
            (name = "files", description = "File items management API")
        )
    )]
    struct ApiDoc;

    // build our application with a route
    let app = Router::new()
        // .merge(router::info::routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route(
            "/",
            get(|| async { Redirect::permanent(&"/swagger-ui".to_string()) }),
        )
        .nest("/api", router::routes());

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
