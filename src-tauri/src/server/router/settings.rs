use crate::server::store;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// exports all routes from this module as router
pub fn routes() -> Router {
    // build local state
    let db = Db::default();

    let data_init = store::load_broker_data();
    match data_init {
        Ok(broker) => {
            let mut n = db.write().unwrap();
            *n = broker;
        }
        Err(err) => {
            println!("{}", err);
            // Not able to parse broker part in json, create default one
            if err.to_string().contains("missing field `broker`") {
                println!("broker field is missing");
            }
        }
    }

    // check if broker settings are present in file, init with data
    if db.read().unwrap().id.to_string() == "00000000-0000-0000-0000-000000000000" {
        let mut n = db.write().unwrap();
        *n = Broker {
            id: Uuid::new_v4(),
            client_id: "mqtt-client-1".to_string(),
            protocol: "mqtt://".to_string(),
            host: "localhost".to_string(),
            port: 1883,
            ..Default::default()
        };
    }

    Router::new()
        .route("/settings/broker", get(settings_index))
        .route("/settings/broker/:id", patch(settings_update))
        .with_state(db)
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
            (status = 200, description = "List broker settings successfully", body = Broker)
        )
    )]
pub async fn settings_index(State(db): State<Db>) -> impl IntoResponse {
    let broker = db.read().unwrap().clone();
    (StatusCode::OK, Json(broker))
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
    path = "/settings/broker/{id}",
    tag = "settings",
    request_body = BrokerUpdateParams,
    responses(
        (status = 200, description = "Broker settings updated successfully")
    )
)]
async fn settings_update(
    State(db): State<Db>,
    Json(input): Json<BrokerUpdateParams>,
) -> impl IntoResponse {
    let mut broker = db.read().unwrap().clone();

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

    let mut n = db.write().unwrap();
    *n = broker.clone();
    drop(n); // release lock so db can be read again

    // store changes to local file.
    store_to_system_file(db.clone());

    (StatusCode::OK, Json(broker))
}

/// Broker schema.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
pub struct Broker {
    id: Uuid,
    client_id: String,
    device_group: String,
    device_id: String,
    host: String,
    password: String,
    port: u16,
    protocol: String,
    username: String,
}

/// Local state for file routes
type Db = Arc<RwLock<Broker>>;

/// Local function to store changes to file system.
fn store_to_system_file(db: Db) {
    if let Err(err) = store::save_broker_data(db.read().unwrap().clone()) {
        println!("Got an error while storing data to file..");
        println!("{:?}", err);
    }
}
