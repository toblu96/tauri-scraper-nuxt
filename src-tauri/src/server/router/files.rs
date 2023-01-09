use crate::server::store::{self, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// exports all routes from this module as router
pub fn routes() -> Router<Arc<AppState>> {
    // build local state
    let db = Db::default();

    let data_init = store::load_files_data();
    match data_init {
        Ok(files) => {
            for file in files.iter() {
                db.write().unwrap().insert(file.id, file.clone());
            }
        }
        Err(err) => {
            println!("{}", err)
        }
    }

    Router::new()
        .route("/files", get(files_index).post(files_create))
        .route("/files/:id", patch(files_update).delete(files_delete))
        .with_state(db)
}

/// List all configured files.
///
/// Returns all configured files for this application.
#[utoipa::path(
        get,
        context_path = "/api",
        path = "/files",
        tag = "files",
        responses(
            (status = 200, description = "List all files successfully", body = [File])
        )
    )]
pub async fn files_index(State(db): State<Db>) -> impl IntoResponse {
    let files = db.read().unwrap().values().cloned().collect::<Vec<_>>();
    (StatusCode::OK, Json(files))
}

/// Body params for creating a new file
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct FileCreateParams {
    /// Filename
    #[schema(example = "ExampleFile.dll")]
    name: String,
    /// Scraper enable state for this file
    #[schema(example = "false")]
    enabled: bool,
    /// Path to file
    #[schema(example = "C:\\win\\doof")]
    path: String,
    /// Mqtt topic on which the current file version gets sent
    #[schema(example = "/eh/test/topic")]
    mqtt_topic: String,
}
/// Add a new file.
///
/// Adds a new file which contains all relevant information for scraping the version of it.
#[utoipa::path(
    post,
    context_path = "/api",
    path = "/files",
    tag = "files",
    request_body = FileCreateParams,
    responses(
        (status = 201, description = "File added successfully", body = File)
    )
)]
pub async fn files_create(
    State(db): State<Db>,
    Json(input): Json<FileCreateParams>,
) -> impl IntoResponse {
    let file = File {
        id: Uuid::new_v4(),
        name: input.name,
        enabled: input.enabled,
        last_update_utc: "".to_string(),
        update_state: "".to_string(),
        last_version: "".to_string(),
        path: input.path,
        mqtt_topic: input.mqtt_topic,
    };

    db.write().unwrap().insert(file.id, file.clone());

    // store changes to local file.
    store_to_system_file(db);
    // if let Err(err) =
    //     store::save_files_data(db.read().unwrap().values().cloned().collect::<Vec<_>>())
    // {
    //     println!("Got an error while storing data to file..");
    //     println!("{:?}", err);
    // }

    (StatusCode::CREATED, Json(file))
}

/// Parameters for updating a file
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct FileUpdateParams {
    /// Filename
    #[schema(example = "ExampleFile.dll")]
    name: Option<String>,
    /// Scraper enable state for this file
    #[schema(example = "false")]
    enabled: Option<bool>,
    /// Path to file
    #[schema(example = "C:\\win\\doof")]
    path: Option<String>,
    /// Mqtt topic on which the current file version gets sent
    #[schema(example = "/eh/test/topic")]
    mqtt_topic: Option<String>,
    /// Timestamp when the fileversion was last read
    #[schema(example = "2022-12-21 13:38:22.948780400 UTC")]
    last_update_utc: Option<String>,
    /// Status of file update - e.g. "could not read" | "successful"
    #[schema(example = "successful")]
    update_state: Option<String>,
    /// Latest file version
    #[schema(example = "7.2.0.0")]
    last_version: Option<String>,
}
/// Update a file.
///
/// Update a specific file by its id.
#[utoipa::path(
    patch,
    context_path = "/api",
    path = "/files/{id}",
    tag = "files",
    request_body = FileUpdateParams,
    params(
        ("id" = Uuid, Path, description = "File database id")
    ),
    responses(
        (status = 200, description = "File updated successfully"),
        (status = 404, description = "No file with this id found")
    )
)]
async fn files_update(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
    Json(input): Json<FileUpdateParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut file = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = input.name {
        file.name = name;
    }

    if let Some(enabled) = input.enabled {
        file.enabled = enabled;
    }

    if let Some(path) = input.path {
        file.path = path;
    }

    if let Some(mqtt_topic) = input.mqtt_topic {
        file.mqtt_topic = mqtt_topic;
    }

    if let Some(last_update_utc) = input.last_update_utc {
        file.last_update_utc = last_update_utc;
    }

    if let Some(update_state) = input.update_state {
        file.update_state = update_state;
    }

    if let Some(last_version) = input.last_version {
        file.last_version = last_version;
    }

    db.write().unwrap().insert(file.id, file.clone());

    // store changes to local file.
    store_to_system_file(db);

    Ok(Json(file))
}

/// Delete a file.
///
/// Deletes a single file by its id.
#[utoipa::path(
    delete,
    context_path = "/api",
    path = "/files/{id}",
    tag = "files",
    params(
        ("id" = Uuid, Path, description = "File database id")
    ),
    responses(
        (status = 204, description = "File deleted successfully"),
        (status = 404, description = "No file with this id found")
    )
)]
async fn files_delete(Path(id): Path<Uuid>, State(db): State<Db>) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        // store changes to local file.
        store_to_system_file(db);

        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Files schema.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct File {
    id: Uuid,
    name: String,
    enabled: bool,
    last_update_utc: String, // timestamp UTC
    update_state: String,    // status of update - e.g. could not read | successful
    last_version: String,    // latest file version
    path: String,
    mqtt_topic: String,
}

/// Local state for file routes
type Db = Arc<RwLock<HashMap<Uuid, File>>>;

/// Local function to store changes to file system.
fn store_to_system_file(db: Db) {
    if let Err(err) =
        store::save_files_data(db.read().unwrap().values().cloned().collect::<Vec<_>>())
    {
        println!("Got an error while storing data to file..");
        println!("{:?}", err);
    }
}
