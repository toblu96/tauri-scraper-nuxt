use crate::server::store::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

static DB_KEY: &str = "broker";

/// exports all routes from this module as router
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/settings/broker", get(settings_index))
        .route("/settings/broker", patch(settings_update))
}

/// Show application broker settings.
///
/// Returns the configured application broker settings.
#[utoipa::path(
        get,
        context_path = "/api",
        path = "/settings/broker",
        tag = "settings",
        responses(
            (status = 200, description = "List broker settings successfully", body = Broker),
            (status = 404, description = "Settings in DB not found", body = DBError, example = json!(DBError::KeyNotFound(String::from("key not found in storage"))))
        )
    )]
pub async fn settings_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // init data if store empty
    init_state_if_necessary(&state);

    match state.db.read().unwrap().get_unwrap::<Broker>(DB_KEY) {
        Ok(broker) => (StatusCode::OK, Json(broker)).into_response(),
        Err(err) => {
            println!("Error: {err:?}");
            (
                StatusCode::NOT_FOUND,
                Json(DBError::KeyNotFound(format!(
                    "{:?}",
                    err.msg.unwrap_or("key not found in storage".to_string())
                ))),
            )
                .into_response()
        }
    }
}

/// Parameters for updating the broker settings
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct BrokerUpdateParams {
    /// Client id
    #[schema(example = "mqtt-client-1")]
    client_id: Option<String>,
    /// Device group
    #[schema(example = "autogroup_Monitor")]
    device_group: Option<String>,
    /// Device id
    #[schema(example = "FC_0103")]
    device_id: Option<String>,
    /// MQTT broker host address
    #[schema(example = "mqtt.fni.qas.endress.com")]
    host: Option<String>,
    /// MQTT broker port
    #[schema(example = "1883")]
    port: Option<u16>,
    /// MQTT broker auth username
    #[schema(example = "admin")]
    username: Option<String>,
    /// MQTT broker auth password
    #[schema(example = "not1234")]
    password: Option<String>,
    /// MQTT broker protocol
    #[schema(example = "mqtt://")]
    protocol: Option<String>,
}
/// Update broker settings.
///
/// Update the application broker settings.
#[utoipa::path(
    patch,
    context_path = "/api",
    path = "/settings/broker",
    tag = "settings",
    request_body = BrokerUpdateParams,
    responses(
        (status = 200, description = "Broker settings updated successfully", body = Broker),
        (status = 404, description = "Settings in DB not found", body = DBError, example = json!(DBError::KeyNotFound(String::from("key not found in storage")))),
        (status = 500, description = "Error on DB write operation", body = DBError, example = json!(DBError::WriteError(String::from("Could not write data to file"))))
    )
)]
async fn settings_update(
    State(state): State<Arc<AppState>>,
    Json(input): Json<BrokerUpdateParams>,
) -> impl IntoResponse {
    let data = state.db.read().unwrap().get_unwrap::<Broker>(DB_KEY);
    match data {
        Ok(mut broker) => {
            // check for changes on each provided input param
            if let Some(client_id) = input.client_id {
                broker.client_id = client_id;
            }

            if let Some(device_group) = input.device_group {
                broker.device_group = device_group;
            }

            if let Some(device_id) = input.device_id {
                broker.device_id = device_id;
            }

            if let Some(host) = input.host {
                broker.host = host;
            }

            if let Some(password) = input.password {
                broker.password = password;
            }

            if let Some(protocol) = input.protocol {
                broker.protocol = protocol;
            }

            if let Some(username) = input.username {
                broker.username = username;
            }

            if let Some(port) = input.port {
                broker.port = port;
            }

            // write to file db
            match state.db.write().unwrap().put(DB_KEY, &broker) {
                Ok(()) => (StatusCode::OK, Json(broker)).into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DBError::WriteError(format!(
                        "{:?}",
                        err.msg
                            .unwrap_or("Could not write data to file".to_string())
                    ))),
                )
                    .into_response(),
            }
        }
        Err(err) => {
            println!("Error: {err:?}");
            (
                StatusCode::NOT_FOUND,
                Json(DBError::KeyNotFound(format!(
                    "{:?}",
                    err.msg.unwrap_or("key not found in storage".to_string())
                ))),
            )
                .into_response()
        }
    }
}

/// Broker schema.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
pub struct Broker {
    // id: Uuid,
    client_id: String,
    device_group: String,
    device_id: String,
    host: String,
    password: String,
    port: u16,
    protocol: String,
    username: String,
}

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
        .get_unwrap::<Broker>(DB_KEY)
        .is_err()
    {
        println!("need to update inital broker state");
        let broker = Broker {
            ..Default::default()
        };
        let _ = state.db.write().unwrap().put(DB_KEY, &broker);
    }
}
