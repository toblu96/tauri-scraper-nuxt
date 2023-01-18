use super::store::AppState;
use crate::server::router::settings::Broker;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

mod debouncer;
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
            // Check broker settings
            let broker = app_state
                .db
                .read()
                .unwrap()
                .get_unwrap::<Broker>("broker")
                .unwrap();
            println!("Got an event from {msg:?}: {:?}", broker)
        }
    });

    // listen to file changes (get files from db and add event listener) -> this should emit a tx event
    // includes also a listener to db file change -> if files are changed then reset listeners

    // rx listener {
    // - read file version
    // - send mqtt with new version
    // }
}
