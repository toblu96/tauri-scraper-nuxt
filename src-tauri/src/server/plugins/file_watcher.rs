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
                tx.send(res).await.unwrap();
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

    // get all files from db and loop through it to add the watchers
    let files = store.read().unwrap().get_unwrap::<Files>("files");
    match files {
        Ok(files) => {
            // if files found, watch them for changes
            for (_uuid, file) in files {
                let path = &file.path;
                if let Err(err) = watcher.watch(Path::new(path), RecursiveMode::Recursive) {
                    println!("Could not add file watcher '{path}' due to: {err:?}");
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

    // check for changes on one of the watched paths
    let mut debouncer = debouncer::Bouncer::new(Duration::from_secs(1));
    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                for path in event.paths.iter() {
                    // debounce change events from listener (separate for each file path)
                    let path_string = String::from(path.to_string_lossy());
                    if let Some(_) = debouncer.debounce(path_string.clone(), || return true) {
                        if let Err(err) =
                            sender.read().unwrap().send(path_string.replace("\\", "/"))
                        {
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
