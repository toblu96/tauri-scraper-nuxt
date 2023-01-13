use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast::Sender;

pub fn main(sender: Arc<RwLock<Sender<String>>>) {
    static FILE_NAME: &str = "C:/Users/i40010702/Desktop/test/jsonDB.json";
    let path = Path::new(FILE_NAME);
    println!("watching {:?}", path);

    // new task required for watcher
    tokio::spawn(async move {
        if let Err(e) = async_watch(path, sender).await {
            println!("error: {:?}", e)
        }
    });
}

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

async fn async_watch<P: AsRef<Path>>(
    path: P,
    sender: Arc<RwLock<Sender<String>>>,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    // check for changes on one of the watched paths
    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                println!("changed: {:?}", event);
                for path in event.paths.iter() {
                    if let Err(err) = sender
                        .write()
                        .unwrap()
                        .send(String::from(path.to_string_lossy()))
                    {
                        println!("Could not send file change event due to: {err:?}")
                    };
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
