use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Bouncer {
    pub delay: Duration,
    bouncer: HashMap<String, BouncerInstance>,
}

struct BouncerInstance {
    last_run: Option<Instant>,
    // func_thread: Option<JoinHandle<()>>,
}

impl Bouncer {
    pub fn new(delay: Duration) -> Self {
        return Bouncer {
            delay,
            bouncer: HashMap::new(),
        };
    }

    /// debounces provided function only running
    /// if it has never been run, or, if the elasped
    /// time has past since the function was last run.
    pub fn debounce<T>(&mut self, path: String, func: fn() -> T) -> Option<T> {
        if let Some(mut debouncer) = self.bouncer.get_mut(&path) {
            // get debouncer by its key
            // let mut debouncer = self.bouncer.get_mut(&path).unwrap();
            if debouncer.last_run.is_some() {
                let then = debouncer.last_run.unwrap();
                let now = Instant::now();

                if now.duration_since(then) > self.delay {
                    debouncer.last_run = Some(Instant::now());

                    return Some(func());
                } else {
                    return None;
                }
            } else {
                debouncer.last_run = Some(Instant::now());
                return Some(func());
            }
        } else {
            // create new debouncer
            self.bouncer.insert(
                path.clone(),
                BouncerInstance {
                    last_run: Some(Instant::now()),
                },
            );

            return Some(func());
        }
    }

    // TODO: Change function execution not on first hit and then delay, instead call only on last execution with specified delay (as usual with debounce functions)
    // fn resetFuncExec(path: String, func: fn() -> T) {
    //     // get thread

    //     // reset thread
    //     let scheduler = tokio::spawn(async {
    //         println!("from inside async call..");
    //     });
    // }
}
