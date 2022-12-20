use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// import routes
mod router;

pub async fn start(port: u16) {
    // build local state
    let db = Db::default();

    // init values
    db.write().unwrap().insert("counter".to_string(), 0.clone());

    // openapi things
    #[derive(OpenApi)]
    #[openapi(
        paths(
            router::info::get_info,
            // todo::list_todos,
            // todo::search_todos,
            // todo::create_todo,
            // todo::mark_done,
            // todo::delete_todo,
        ),
        components(
            schemas(router::info::Info)
        ),
        tags(
            (name = "info", description = "Information about this application")
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
        // .route("/", get(handler))
        .with_state(db)
        .merge(router::routes());

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(State(db): State<Db>) -> Result<impl IntoResponse, StatusCode> {
    // get counter state
    let mut counter = db
        .read()
        .unwrap()
        .get("counter")
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    counter += 1;

    // store new value to db
    db.write()
        .unwrap()
        .insert("counter".to_string(), counter.clone())
        .expect("Error while insterting in db..");

    let todo = Todo {
        id: "öjasdf790asfd".to_string(),
        text: "my first todo topic".to_string(),
        completed: false,
        count: counter,
    };

    Ok(Json(todo))
}

type Db = Arc<RwLock<HashMap<String, u16>>>;

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
    count: u16,
}
