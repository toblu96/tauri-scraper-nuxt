use super::{debouncer, mqtt_client::MqttClient};
use crate::server::{plugins::handle_file_change, router::files::Files, store::AppState};
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

static DB_KEY: &str = "files";

pub struct FileWatcher {
    sender: Arc<RwLock<Sender<String>>>,
    store: Arc<RwLock<MicroKV>>,
    watcher_thread: Arc<RwLock<JoinHandle<()>>>,
    current_file_config: Arc<RwLock<Files>>,
    mqtt_client: MqttClient,
}

impl FileWatcher {
    /// Init file watcher plugin
    pub fn init(
        sender: Arc<RwLock<Sender<String>>>,
        app_state: &Arc<AppState>,
        mqtt_client: &MqttClient,
    ) -> Self {
        // start file watcher
        let watch_store = app_state.db.clone();
        let watch_sender = sender.clone();
        let watcher_thread = tokio::spawn(async move {
            if let Err(e) = async_watch(watch_store, watch_sender).await {
                println!("error: {:?}", e)
            }
        });

        // add current file state
        let current_file_config = app_state
            .db
            .read()
            .unwrap()
            .get_unwrap::<Files>(DB_KEY)
            .unwrap();

        FileWatcher {
            sender,
            store: app_state.db.clone(),
            watcher_thread: Arc::new(RwLock::new(watcher_thread)),
            current_file_config: Arc::new(RwLock::new(current_file_config)),
            mqtt_client: mqtt_client.clone(),
        }
    }

    /// Refresh currently watched files
    pub fn refresh(&mut self) {
        let current_files = self.current_file_config.read().unwrap().clone();
        let new_files = self
            .store
            .read()
            .unwrap()
            .get_unwrap::<Files>(DB_KEY)
            .unwrap();
        if current_files != new_files {
            println!("refresh watchers");
            // first drop all active file watchers and end task
            self.watcher_thread.write().unwrap().abort();
            // add new file watchers by starting them in new task
            let watch_store = self.store.clone();
            let watch_sender = self.sender.clone();
            let thread = tokio::spawn(async move {
                if let Err(e) = async_watch(watch_store, watch_sender).await {
                    println!("error: {:?}", e)
                }
            });

            // trigger reread of changed files (for example enable state)
            for (_uuid, new_file) in &new_files {
                // skip disabled file watchers
                if !&new_file.enabled {
                    continue;
                }

                // check if there are any changes
                if let Some(current_file) = current_files.get(_uuid) {
                    if new_file != current_file {
                        handle_file_change(&current_file.path, &self.store, &mut self.mqtt_client);
                    }
                }
            }

            // store updated data to local state
            *self.watcher_thread.write().unwrap() = thread;
            *self.current_file_config.write().unwrap() = new_files;
        }
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
    {
        let lock = store.write().unwrap();
        let files = lock.get_unwrap::<Files>("files");
        match files {
            Ok(mut files) => {
                // if files found, watch the parent folders for changes
                for (_uuid, mut file) in &mut files {
                    // skip disabled file watchers
                    if !&file.enabled {
                        continue;
                    }

                    active_watch_files.push(file.path.clone().replace("\\", "/"));

                    if let Some(folder_path) = Path::new(&file.path).parent() {
                        // only add folder watcher if not added yet
                        if !new_watch_paths.contains(&String::from(folder_path.to_string_lossy())) {
                            if let Err(err) =
                                watcher.watch(folder_path, RecursiveMode::NonRecursive)
                            {
                                println!(
                                    "Could not add file watcher '{folder_path:?}' due to: {err:?}"
                                );
                                // update file state in case of error and disable watcher
                                file.update_state = err.to_string();
                                file.enabled = false;
                                continue;
                            };
                            new_watch_paths.push(String::from(folder_path.to_string_lossy()));
                        }
                    };
                }
                if let Err(err) = lock.put("files", &files) {
                    println!("Could not update file state on local file db: {err:?}")
                }
            }
            Err(err) => {
                // if no file found. skip this part
                println!("Could not add file watcher because there are no configured ones. {err:?}")
            }
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
    let mut debouncer = debouncer::Bouncer::new(Duration::from_millis(500));
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
                    let tmp_sender = sender.clone();
                    let tmp_path_string = path_string.clone();
                    if let Err(err) = debouncer.debounce(
                        path_string.clone(),
                        Box::new(move || {
                            if let Err(err) = tmp_sender
                                .clone()
                                .read()
                                .unwrap()
                                .send(tmp_path_string.clone())
                            {
                                println!("Could not send file change event due to: {err:?}")
                            };
                        }),
                    ) {
                        println!("debounce error: {:?}", err)
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
