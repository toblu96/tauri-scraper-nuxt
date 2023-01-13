use super::store::AppState;
use crate::server::router::settings::Broker;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

mod file_watcher;

pub fn init(app_state: Arc<AppState>) {
    // tokio::spawn(async move {
    // Instantiate shared channel
    let (tx, mut rx) = broadcast::channel::<String>(40);
    let tx_file_watcher = Arc::new(RwLock::new(tx.clone()));

    // Create global listener - execute version and mqtt logic here
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            println!("Got a message: {:?}", msg);

            // Check broker settings
            let broker = app_state
                .db
                .read()
                .unwrap()
                .get_unwrap::<Broker>("broker")
                .unwrap();
            println!("Broker fdata: {:?}", broker)
        }
    });

    // start file watcher
    file_watcher::main(tx_file_watcher);

    // listen to file changes (get files from db and add event listener) -> this should emit a tx event
    // includes also a listener to db file change -> if files are changed then reset listeners

    // rx listener {
    // - read file version
    // - send mqtt with new version
    // }
    // });
}
