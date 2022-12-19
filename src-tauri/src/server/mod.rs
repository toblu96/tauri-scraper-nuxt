use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

// import routes
mod router;

pub async fn start(port: u16) {
    // build local state
    let db = Db::default();

    // init values
    db.write().unwrap().insert("counter".to_string(), 0.clone());

    // build our application with a route
    let app = Router::new()
        // .merge(router::info::routes())
        .route("/", get(handler))
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

// async fn handler() -> Html<&'static str> {

//     Html("<h1>Hello, World!</h1><p>Whats up?</p>")
// }