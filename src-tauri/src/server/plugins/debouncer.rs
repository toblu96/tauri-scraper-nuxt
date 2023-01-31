use std::collections::HashMap;
use std::time::Duration;
use tokio::task::JoinHandle;

type Func = Box<dyn FnMut() + Send + Sync>;

pub struct Bouncer {
    pub delay: Duration,
    bouncer: HashMap<String, BouncerInstance>,
}

struct BouncerInstance {
    task: JoinHandle<()>,
}

impl Bouncer {
    pub fn new(delay: Duration) -> Self {
        return Bouncer {
            delay,
            bouncer: HashMap::new(),
        };
    }

    /// Debounce an inline functino execution with a specific duration. Calls only on last execution with specified delay.
    pub fn debounce(&mut self, path: String, func: Func) -> Result<(), &str> {
        let mut execute = func;
        let delay = self.delay;

        // start function in delayed task
        let task = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            execute()
        });

        // store task in local state and abort the old one (if there was one)
        if let Some(mut debouncer) = self.bouncer.get_mut(&path) {
            if !debouncer.task.is_finished() {
                debouncer.task.abort();
            }
            debouncer.task = task;
        } else {
            // create new debouncer
            self.bouncer.insert(path.clone(), BouncerInstance { task });
        }

        Ok(())
    }
}
