use super::{router::settings::Broker, store::AppState};
use crate::server::router::files::Files;
use chrono::{self, SecondsFormat};
use log::{error, info};
use microkv::MicroKV;
use serde_json::json;
use std::{
    path::Path,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast;

mod debouncer;
mod file_version_reader;
mod file_watcher;
mod mqtt_client;

static DB_KEY: &str = "broker";

pub fn init(app_state: Arc<AppState>) {
    // Instantiate shared channel
    let (tx, mut rx) = broadcast::channel::<String>(40);
    let tx_file_watcher = Arc::new(RwLock::new(tx.clone()));

    // init mqtt client
    let mut client = mqtt_client::MqttClient::init(&app_state);

    // init file listener
    let mut file_watcher = file_watcher::FileWatcher::init(tx_file_watcher, &app_state, &client);

    // check all enabled file versions on application start
    let files = app_state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>("files")
        .unwrap();
    for (_uuid, file) in files {
        // skip disabled file watchers
        if !&file.enabled {
            continue;
        }
        handle_file_change(&file.path, &app_state.db, &mut client);
    }

    // Create global listener - execute version and mqtt logic here
    tokio::spawn(async move {
        while let Ok(path) = rx.recv().await {
            // handle local db changes
            let db_path = crate::server::store::FILE_DB_PATH;
            let db_name = crate::server::store::FILE_DB_NAME;
            let db_string = format!("{db_path}/{db_name}.kv");
            if path == db_string {
                // refresh watcher if file is new/deleted or path is changed
                file_watcher.refresh();
                // update mqtt client on settings change (client only)
                client.refresh();

                continue;
            }

            // TODO: handle other file changes
            handle_file_change(&path, &app_state.db, &mut client);
        }
    });
}

/// Gets the new file version on file change and stores it to the local DB
fn handle_file_change(
    path: &String,
    db: &Arc<RwLock<MicroKV>>,
    mqtt_client: &mut mqtt_client::MqttClient,
) {
    if let Some(os_str) = Path::new(&path).extension() {
        if let Some(extension) = os_str.to_str() {
            // handle different file types
            match extension {
                "exe" | "dll" => {
                    // get file version from file properties
                    let file_version =
                        file_version_reader::get_file_version_from_file_properties(&path);
                    match file_version {
                        Ok(version) => {
                            update_file_version(db, mqtt_client, path.clone(), version.clone());
                        }
                        Err(err) => {
                            error!(
                                "Could not get file version from path '{}' due to: {err:?}",
                                &path
                            );
                            update_file_error(db, path.clone(), err);
                        }
                    }
                }
                _ => {
                    // get current time stamp and file hash - no file version available
                    let hash = file_version_reader::get_file_meta_hash(&path);
                    match hash {
                        Ok(hash) => {
                            update_file_version(db, mqtt_client, path.clone(), hash.clone());
                        }
                        Err(err) => {
                            error!(
                                "Could not get file version from path '{}' due to: {err:?}",
                                &path
                            );
                            update_file_error(db, path.clone(), err);
                        }
                    }
                }
            }
        }
    }
}

/// Write the new file version to the local DB
fn update_file_version(
    db: &Arc<RwLock<MicroKV>>,
    mqtt_client: &mut mqtt_client::MqttClient,
    path: String,
    version: String,
) {
    // Update file state with version
    let lock = db.write().unwrap();
    let mut files = lock.get_unwrap::<Files>("files").unwrap();
    // update file version for all files with matching path
    let files_iterator = &mut files;
    for (_uuid, file) in files_iterator {
        // skip changes if path does not match
        if file.path.replace("\\", "/") != path.replace("\\", "/") {
            continue;
        }

        file.last_version = version.clone();
        file.last_update_utc = chrono::offset::Utc::now().to_string();
        file.update_state = "Success".to_string();

        // only send mqtt message if broker is connected
        let broker = lock.get_unwrap::<Broker>(DB_KEY);
        match broker {
            Ok(broker) => {
                // log info about new file version
                info!(
                    "[{}] Got version change from file '{}' to version '{}'",
                    &broker.device_id, &file.name, &version
                );

                if broker.connected {
                    // send mqtt message
                    let device_id = mqtt_client
                        .current_client_config
                        .read()
                        .unwrap()
                        .device_id
                        .clone();
                    let device_group = mqtt_client
                        .current_client_config
                        .read()
                        .unwrap()
                        .device_group
                        .clone();

                    mqtt_client.publish(
                        &file.mqtt_topic,
                        json!({
                          "deviceId": device_id,
                          "timestamp": format!("{}", chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)),
                          "group": device_group,
                          "measures": {
                            format!("{}", &file.name): &file.last_version,
                            format!("{}DataType", &file.name): "String",
                          },
                        }),
                    );
                } else {
                    file.update_state = "MQTT broker connection failed".to_string()
                }
            }
            Err(err) => error!("Could not get broker data due to: {err:?}"),
        }
    }

    // store data to local db
    if let Err(err) = lock.put("files", &files) {
        error!("Could not write new file version to local DB: {err:?}")
    }
}

/// Writes a new file error to the local DB
fn update_file_error(db: &Arc<RwLock<MicroKV>>, path: String, error: String) {
    // Update file state with version
    let lock = db.write().unwrap();
    let mut files = lock.get_unwrap::<Files>("files").unwrap();
    // update file version for all files with matching path
    let files_iterator = &mut files;
    for (_uuid, file) in files_iterator {
        // skip changes if path does not match
        if file.path.replace("\\", "/") != path.replace("\\", "/") {
            continue;
        }

        file.last_update_utc = chrono::offset::Utc::now().to_string();
        file.update_state = error.clone();
    }

    // store data to local db
    if let Err(err) = lock.put("files", &files) {
        error!("Could not write new file version to local DB: {err:?}")
    }
}
