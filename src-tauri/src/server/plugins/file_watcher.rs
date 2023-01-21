use super::debouncer;
use crate::server::{router::files::Files, store::AppState};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use microkv::MicroKV;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;

pub struct FileWatcher {
    sender: Arc<RwLock<Sender<String>>>,
    store: Arc<RwLock<MicroKV>>,
    watcher_thread: JoinHandle<()>,
}

impl FileWatcher {
    /// Init file watcher plugin
    pub fn init(sender: Arc<RwLock<Sender<String>>>, app_state: &Arc<AppState>) -> Self {
        // start file watcher
        let watch_store = app_state.db.clone();
        let watch_sender = sender.clone();
        let watcher_thread = tokio::spawn(async move {
            if let Err(e) = async_watch(watch_store, watch_sender).await {
                println!("error: {:?}", e)
            }
        });

        FileWatcher {
            sender,
            store: app_state.db.clone(),
            watcher_thread,
        }
    }

    /// Refresh currently watched files
    pub fn refresh(&mut self) {
        println!("refresh watchers");
        // first drop all active file watchers and end task
        self.watcher_thread.abort();
        // add new file watchers by starting them in new task
        let watch_store = self.store.clone();
        let watch_sender = self.sender.clone();
        self.watcher_thread = tokio::spawn(async move {
            if let Err(e) = async_watch(watch_store, watch_sender).await {
                println!("error: {:?}", e)
            }
        });
    }
}

/// Create new file watcher instance
fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // create watcher instance
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                if let Err(err) = tx.send(res).await {
                    println!("Could not send value. {err:?}")
                };
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

/// Start file watchers for all configured files
async fn async_watch(
    store: Arc<RwLock<MicroKV>>,
    sender: Arc<RwLock<Sender<String>>>,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    let mut new_watch_paths: Vec<String> = Vec::new();
    let mut active_watch_files: Vec<String> = Vec::new();

    // get all files from db and loop through it to add the watchers
    let files = store.read().unwrap().get_unwrap::<Files>("files");
    match files {
        Ok(files) => {
            // if files found, watch the parent folders for changes
            for (_uuid, file) in files {
                // skip disabled file watchers
                if !&file.enabled {
                    continue;
                }

                active_watch_files.push(file.path.clone());

                if let Some(folder_path) = Path::new(&file.path).parent() {
                    // only add folder watcher if not added yet
                    if !new_watch_paths.contains(&String::from(folder_path.to_string_lossy())) {
                        if let Err(err) = watcher.watch(folder_path, RecursiveMode::NonRecursive) {
                            println!(
                                "Could not add file watcher '{folder_path:?}' due to: {err:?}"
                            );
                            continue;
                        };
                        new_watch_paths.push(String::from(folder_path.to_string_lossy()));
                    }
                };
            }
        }
        Err(err) => {
            // if no file found. skip this part
            println!("Could not add file watcher because there are no configured ones. {err:?}")
        }
    }
    // always watch for local db file changes
    let db_path = crate::server::store::FILE_DB_PATH;
    let db_name = crate::server::store::FILE_DB_NAME;
    let db_string = format!("{db_path}/{db_name}.kv");
    if let Err(err) = watcher.watch(Path::new(&db_string), RecursiveMode::Recursive) {
        println!("Could not add local db watcher '{db_string}' due to: {err:?}");
    };
    active_watch_files.push(db_string.clone());

    // check for changes on one of the watched paths
    let mut debouncer = debouncer::Bouncer::new(Duration::from_secs(1));
    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                for path in event.paths.iter() {
                    let path_string = String::from(path.to_string_lossy()).replace("\\", "/");

                    // only pass event for enabled file paths
                    if !active_watch_files.contains(&path_string) {
                        continue;
                    }
                    // debounce change events from listener (separate for each file path)
                    if let Some(_) = debouncer.debounce(path_string.clone(), || return true) {
                        // println!("Sent from {path:?}");
                        if let Err(err) = sender.read().unwrap().send(path_string) {
                            println!("Could not send file change event due to: {err:?}")
                        };
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
