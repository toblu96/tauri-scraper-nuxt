use axum::{response::Redirect, routing::get, Router};
use log::info;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod plugins;
mod router;
mod store;

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
            router::settings::settings_index,
            router::settings::settings_update,
            router::logs::logs_index
        ),
        components(
            schemas(
                router::info::Info, 
                router::files::File, 
                router::files::FileUpdateParams, 
                router::files::FileCreateParams, 
                router::settings::Broker, 
                router::settings::BrokerUpdateParams, 
                router::settings::DBError, 
                router::logs::Logs,
                router::logs::ServerError,
                router::logs::LogLevels
            )
        ),
        tags(
            (name = "info", description = "Information about this application"),
            (name = "files", description = "File items management API"),
            (name = "settings", description = "Application settings management API"),
            (name = "logs", description = "Application logs API")
        )
    )]
    struct ApiDoc;

    // init application state
    let app_state = store::init_state();

    // build our application with a route
    let mut doc = ApiDoc::openapi();
    doc.info.title = String::from("EH Version Monitoring");
    let app = Router::new()
        // .merge(router::info::routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", doc))
        .route(
            "/",
            get(|| async { Redirect::permanent(&"/swagger-ui".to_string()) }),
        )
        .nest("/api", router::routes())
        .with_state(app_state.clone())
        .layer(
            // CorsLayer::new()
            //     .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            //     .allow_methods(Any)
            //     .allow_headers(Any),
            CorsLayer::permissive(),
        );

    // init plugins
    plugins::init(app_state);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("[Server] Server started, listening on {}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
