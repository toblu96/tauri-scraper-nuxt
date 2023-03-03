use crate::logger::LOG_FILES_PATH;
use crate::logger::LOG_FILE_NAME;
use crate::server::store::AppState;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

/// exports all routes from this module as router
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/logs", get(logs_index))
}

/// Show application logs.
///
/// Returns filtered log entries from the application.
#[utoipa::path(
        get,
        context_path = "/api",
        path = "/logs",
        tag = "logs",
        params(
            LogFilterQuery
        ),
        responses(
            (status = 200, description = "List log entries successfully", body = [Logs]),
            (status = 404, description = "Could not find any server log files", body = ServerError, example = json!(ServerError::LogFileNotFound(String::from("No logfiles found on the server")))),
            (status = 500, description = "Could not find any server log files", body = ServerError, example = json!(ServerError::LogFileInternalError(String::from("Error on logfile read"))))
        )
    )]
pub async fn logs_index(
    filter: Query<LogFilterQuery>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // get all log file names
    match get_log_file_names() {
        Ok(file_names) => {
            // no log files found
            if file_names.len() == 0 {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ServerError::LogFileNotFound(
                        "No logfiles found on the server".to_string(),
                    )),
                )
                    .into_response();
            }

            // handle rotation log files
            let mut log_lines: Vec<Logs> = Vec::new();

            for file_name in file_names {
                // Read log file and add content to local vec
                if let Ok(mut file) = File::open(file_name) {
                    let mut file_contents = String::new();
                    file.read_to_string(&mut file_contents)
                        .ok()
                        .expect("failed to read!");
                    let mut logs: Vec<Logs> = file_contents
                        .split("\n")
                        .filter_map(|s: &str| {
                            // only return successfully parsed lines
                            serde_json::from_str::<Logs>(&s).ok()
                        })
                        .collect();

                    // filter log lines according to filter params
                    if let Some(start_date) = &filter.start_date {
                        logs.retain(|log| {
                            if let Ok(time) = &log.time.parse::<DateTime<Utc>>() {
                                time >= start_date
                            } else {
                                false
                            }
                        })
                    }
                    if let Some(end_date) = &filter.end_date {
                        logs.retain(|log| {
                            if let Ok(time) = &log.time.parse::<DateTime<Utc>>() {
                                time <= end_date
                            } else {
                                false
                            }
                        })
                    }

                    if let Some(level) = &filter.level {
                        match level {
                            LogLevels::ALL => {}
                            _ => {
                                logs.retain(|log| log.level == format!("{:?}", level));
                            }
                        }
                    }

                    if let Some(message) = &filter.message {
                        logs.retain(|log| {
                            log.message.to_lowercase().contains(&message.to_lowercase())
                        });
                    }

                    // write filtered log lines to local var
                    log_lines.append(&mut logs);
                }
            }

            // sort by date (desc)
            log_lines.sort_by_key(|log| std::cmp::Reverse(log.time.clone()));

            // return data
            return (StatusCode::OK, Json(log_lines)).into_response();
        }
        Err(err) => {
            error!("[API] Could not get log files: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerError::LogFileInternalError(format!(
                    "Error on logfile read: {err}"
                ))),
            )
                .into_response();
        }
    }
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

/// Server errors
#[derive(Serialize, Deserialize, ToSchema)]
pub enum ServerError {
    /// No Logfile found on the server.
    #[schema(example = "No logfiles found on the server")]
    LogFileNotFound(String),
    /// Server error on logfile read.
    #[schema(example = "General internal log error")]
    LogFileInternalError(String),
}

#[derive(Deserialize, IntoParams)]
pub struct LogFilterQuery {
    /// Filter log entries for specific log levels
    level: Option<LogLevels>,
    /// Filter log message for specific word pattern
    message: Option<String>,
    /// Filter messages from this point in time (UTC) - example format: '2023-02-28T12:00:00Z'
    start_date: Option<DateTime<Utc>>,
    /// Filter messages to this point in time (UTC) - example format: '2023-02-28T12:00:00Z'
    end_date: Option<DateTime<Utc>>,
}

/// Log levels
#[derive(Deserialize, ToSchema, Debug)]
pub enum LogLevels {
    ALL,
    DEBUG,
    TRACE,
    INFO,
    WARN,
    ERROR,
}

fn get_log_file_names() -> Result<Vec<PathBuf>, std::io::Error> {
    let mut entries = read_dir(LOG_FILES_PATH)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    // only keep files which start with the logfile name
    entries.retain(|path| match path.file_name() {
        Some(name) => String::from(name.to_string_lossy()).starts_with(LOG_FILE_NAME),
        None => false,
    });
    Ok(entries)
}
