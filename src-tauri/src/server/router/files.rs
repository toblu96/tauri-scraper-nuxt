use crate::server::store::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

static DB_KEY: &str = "files";

/// exports all routes from this module as router
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/files", get(files_index).post(files_create))
        .route("/files/:id", patch(files_update).delete(files_delete))
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
pub async fn files_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // init data if store empty
    init_state_if_necessary(&state);

    let files = state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>(DB_KEY)
        .unwrap()
        .values()
        .cloned()
        .collect::<Vec<_>>();

    // let files = db.read().unwrap().values().cloned().collect::<Vec<_>>();
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
        (status = 201, description = "File added successfully", body = File),
        (status = 404, description = "File in DB not found", body = DBError, example = json!(DBError::KeyNotFound(String::from("key not found in storage")))),
        (status = 500, description = "Error on DB write operation", body = DBError, example = json!(DBError::WriteError(String::from("Could not write data to file"))))
    )
)]
pub async fn files_create(
    State(state): State<Arc<AppState>>,
    Json(input): Json<FileCreateParams>,
) -> impl IntoResponse {
    // init data if store empty
    init_state_if_necessary(&state);

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

    // update hash map
    let mut files = state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>(DB_KEY)
        .unwrap();

    files.insert(file.id, file.clone());

    if let Err(err) = state.db.write().unwrap().put(DB_KEY, &files) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DBError::WriteError(format!(
                "{:?}",
                err.msg
                    .unwrap_or("Could not write data to file".to_string())
            ))),
        )
            .into_response();
    };

    (StatusCode::CREATED, Json(file)).into_response()
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
        (status = 404, description = "No file with this id found", body = DBError, example = json!(DBError::KeyNotFound(String::from("key not found in storage"))))
    )
)]
async fn files_update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(input): Json<FileUpdateParams>,
) -> impl IntoResponse {
    let mut files = state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>(DB_KEY)
        .unwrap();

    if let Some(file) = files.get_mut(&id) {
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
    } else {
        // no entry with this id found
        return (
            StatusCode::NOT_FOUND,
            Json(DBError::KeyNotFound("key not found in storage".to_string())),
        )
            .into_response();
    }

    // write to file db
    match state.db.write().unwrap().put(DB_KEY, &files) {
        Ok(()) => (StatusCode::OK, Json(files.get(&id))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DBError::WriteError(format!(
                "{:?}",
                err.msg
                    .unwrap_or("Could not write file data to file".to_string())
            ))),
        )
            .into_response(),
    }
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
        (status = 404, description = "File in DB not found", body = DBError, example = json!(DBError::KeyNotFound(String::from("key not found in storage")))),
        (status = 500, description = "Error on DB write operation", body = DBError, example = json!(DBError::WriteError(String::from("Could not write data to file"))))
    )
)]
async fn files_delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut files = state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>(DB_KEY)
        .unwrap();

    // try to remove locally
    if let None = files.remove(&id) {
        return (
            StatusCode::NOT_FOUND,
            Json(DBError::KeyNotFound("key not found in storage".to_string())),
        )
            .into_response();
    }

    // write to file db
    if let Err(err) = state.db.write().unwrap().put(DB_KEY, &files) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DBError::WriteError(format!(
                "{:?}",
                err.msg
                    .unwrap_or("Could not write file data to file".to_string())
            ))),
        )
            .into_response()
    } else {
        (StatusCode::NO_CONTENT, Json({})).into_response()
    }
}

/// Files schema.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct File {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub last_update_utc: String, // timestamp UTC
    pub update_state: String,    // status of update - e.g. could not read | successful
    pub last_version: String,    // latest file version
    pub path: String,
    pub mqtt_topic: String,
}
pub type Files = HashMap<Uuid, File>;

/// File DB operation errors
#[derive(Serialize, Deserialize, ToSchema)]
pub enum DBError {
    /// Key not found in storage file.
    #[schema(example = "Key not found in storage file")]
    KeyNotFound(String),
    /// DB file not writeable.
    #[schema(example = "Could not write data to file")]
    WriteError(String),
}

fn init_state_if_necessary(state: &Arc<AppState>) {
    if state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>(DB_KEY)
        .is_err()
    {
        println!("need to update inital files state");

        let _ = state.db.write().unwrap().put(DB_KEY, &Files::new());
    }
}
