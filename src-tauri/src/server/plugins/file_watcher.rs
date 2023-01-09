use notify::{RecursiveMode, Result};
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn main() {
    static FILE_NAME: &str = "C:/Users/i40010702/Desktop/test/jsonDB.json";
    let path = Path::new(FILE_NAME);
    println!("watching {:?}", path);
    if let Err(e) = watch(path) {
        println!("error: {:?}", e)
    }
}

fn watch<P: AsRef<Path>>(path: P) -> Result<()> {
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    // let mut watcher = RecommendedWatcher::new(tx, Config::default().with_compare_contents(true))?;
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    // watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // TODO: pack in task because this blocks the whole execution
    for res in rx {
        match res {
            // If there is a match execute the logevent function with the event::notify::Event as input
            // TODO: Only check for event.kind == Any (not AnyContinous because these are fired multiple times)
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
