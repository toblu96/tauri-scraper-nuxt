use tokio::sync::broadcast;

mod file_watcher;

pub fn init() {
    // Instantiate shared channel
    let (tx, mut rx) = broadcast::channel::<String>(100);

    // Create global listener
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            println!("Got a message: {:?}", msg)
        }
    });

    let _ = tx.send("hell no".to_string());
    // start file watcher
    file_watcher::main(tx.clone());

    // listen to file changes (get files from db and add event listener) -> this should emit a tx event
    // includes also a listener to db file change -> if files are changed then reset listeners

    // rx listener {
    // - read file version
    // - send mqtt with new version
    // }
}
