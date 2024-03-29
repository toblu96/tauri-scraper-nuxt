use crate::server::store::AppState;
use axum::{
    extract::{Path, State, TypedHeader},
    headers,
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, patch},
    Json, Router,
};
use futures::stream::{self, Stream};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

static DB_KEY: &str = "files";

/// exports all routes from this module as router
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/files", get(files_index).post(files_create))
        .route("/files/:id", patch(files_update).delete(files_delete))
        .route("/files/sse", get(files_index_sse))
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
    let files = state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>(DB_KEY)
        .unwrap()
        .values()
        .cloned()
        .collect::<Vec<_>>();

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
    #[schema(example = "eh/test/topic")]
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
    let lock = state.db.write().unwrap();
    let mut files = lock.get_unwrap::<Files>(DB_KEY).unwrap();

    files.insert(file.id, file.clone());

    // log new file entry
    info!("[Files] New file added: {:?}", &file);

    if let Err(err) = lock.put(DB_KEY, &files) {
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
    #[schema(example = "eh/test/topic")]
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
    let lock = state.db.write().unwrap();
    let mut files = lock.get_unwrap::<Files>(DB_KEY).unwrap();

    if let Some(file) = files.get_mut(&id) {
        if let Some(name) = input.name {
            file.name = name;
        }

        if let Some(path) = input.path {
            file.path = path;
        }

        if let Some(enabled) = input.enabled {
            // only enable file if file path is valid
            if std::path::Path::new(&file.path).exists() {
                file.enabled = enabled;
            } else {
                file.enabled = false;
            }
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

        // log changes
        info!("[Files] File config changed to: {:?}", &file);
    } else {
        // no entry with this id found
        return (
            StatusCode::NOT_FOUND,
            Json(DBError::KeyNotFound("key not found in storage".to_string())),
        )
            .into_response();
    }

    // write to file db
    match lock.put(DB_KEY, &files) {
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
    let lock = state.db.write().unwrap();
    let mut files = lock.get_unwrap::<Files>(DB_KEY).unwrap();

    // try to remove locally
    if let None = files.remove(&id) {
        return (
            StatusCode::NOT_FOUND,
            Json(DBError::KeyNotFound("key not found in storage".to_string())),
        )
            .into_response();
    }

    // log changes
    info!("[Files] File with id '{id}' deleted.");

    // write to file db
    if let Err(err) = lock.put(DB_KEY, &files) {
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

/// Get "realtime" changes for all configured files.
///
/// Returns all configured files for this application. Updates automatically using SSE.
async fn files_index_sse(
    State(state): State<Arc<AppState>>,
    TypedHeader(_user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // A `Stream` that repeats an event every second
    let stream = stream::repeat_with(move || {
        let files = state
            .db
            .read()
            .unwrap()
            .get_unwrap::<Files>(DB_KEY)
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>();

        Event::default().data(json!(files).to_string())
    })
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

/// Files schema.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
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

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.enabled == other.enabled
            && self.mqtt_topic == other.mqtt_topic
            && self.name == other.name
            && self.path == other.path
    }
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
