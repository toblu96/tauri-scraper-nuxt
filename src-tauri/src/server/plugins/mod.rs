use super::store::AppState;
use crate::server::router::files::Files;
use chrono;
use std::{
    path::Path,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast;

mod debouncer;
mod file_version_reader;
mod file_watcher;

pub fn init(app_state: Arc<AppState>) {
    // Instantiate shared channel
    let (tx, mut rx) = broadcast::channel::<String>(40);
    let tx_file_watcher = Arc::new(RwLock::new(tx.clone()));

    // init file listener
    let mut file_watcher = file_watcher::FileWatcher::init(tx_file_watcher, &app_state);

    // Create global listener - execute version and mqtt logic here
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // handle local db changes
            let db_path = crate::server::store::FILE_DB_PATH;
            let db_name = crate::server::store::FILE_DB_NAME;
            let db_string = format!("{db_path}/{db_name}.kv");
            if msg == db_string {
                file_watcher.refresh();
                continue;
            }

            // TODO: handle other file changes
            // handle different file types
            if let Some(os_str) = Path::new(&msg).extension() {
                if let Some(extension) = os_str.to_str() {
                    match extension {
                        "exe" | "dll" => {
                            // get file version from file properties
                            let file_version =
                                file_version_reader::get_file_version_from_file_properties(&msg);
                            match file_version {
                                Ok(version) => {
                                    update_file_version(&app_state, msg.clone(), version.clone());
                                    // TODO: send mqtt message
                                }
                                Err(err) => {
                                    println!("Could not get file version due to: {err:?}")
                                    // TODO: Update file state with error message
                                }
                            }
                        }
                        _ => {
                            // get current time stamp and file hash - no file version available
                            let hash = file_version_reader::get_file_meta_hash(&msg);
                            match hash {
                                Ok(hash) => {
                                    update_file_version(&app_state, msg.clone(), hash.clone());
                                    // TODO: send mqtt message
                                }
                                Err(err) => println!("Could not get file version due to: {err:?}"),
                            }
                        }
                    }
                }
            }
        }
    });
}

fn update_file_version(app_state: &Arc<AppState>, path: String, version: String) {
    // Update file state with version
    let mut files = app_state
        .db
        .read()
        .unwrap()
        .get_unwrap::<Files>("files")
        .unwrap();
    // update file version for all files with matching path
    let files_iterator = &mut files;
    for (_uuid, file) in files_iterator {
        // skip changes if path does not match
        if file.path != path {
            continue;
        }

        file.last_version = version.clone();
        file.last_update_utc = chrono::offset::Utc::now().to_string();
        file.update_state = "Success".to_string();
    }

    // store data to local db
    if let Err(err) = app_state.db.write().unwrap().put("files", &files) {
        println!("Could not write new file version to local DB: {err:?}")
    }
}
