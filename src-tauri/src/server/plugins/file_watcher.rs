use notify::{RecursiveMode, Result};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::Path;
use std::sync::mpsc::channel;
use tokio::sync::broadcast::Sender;

use std::time::Duration;

pub fn main(sender: Sender<String>) {
    // new task required for watcher
    tokio::spawn(async move {
        static FILE_NAME: &str = "C:/Users/i40010702/Desktop/test/jsonDB.json";
        let path = Path::new(FILE_NAME);
        println!("watching {:?}", path);
        if let Err(e) = watch(path, sender) {
            println!("error: {:?}", e)
        }
    });
}

fn watch<P: AsRef<Path>>(path: P, sender: Sender<String>) -> Result<()> {
    let (tx, rx) = channel();

    // Create new debounced file watcher instance
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx)?;

    // Add all path to be watched.
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    debouncer.watcher().watch(
        Path::new("C:/Users/i40010702/Desktop/test/jsonDB - Kopie.json"),
        RecursiveMode::NonRecursive,
    )?;

    // Check for file change events - loop needed to keep rx session alive!!!!
    loop {
        for res in rx.recv() {
            match res {
                Ok(event) => {
                    if event[0].kind == DebouncedEventKind::Any {
                        println!("changed: {:?}", event);
                        // Communicate to outside world
                        if let Err(e) = sender.send(String::from(format!(
                            "File change detected: {:?}",
                            event[0].path
                        ))) {
                            println!("Got a tx sending error: {}", e)
                        }
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }
}
