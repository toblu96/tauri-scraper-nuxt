use crate::server::router::{files::Files, settings::Broker};
use microkv::MicroKV;
use std::{
    path::Path,
    sync::{Arc, RwLock},
};

pub static FILE_DB_PATH: &str = "C:/ProgramData/Tauri/EH Version Scraper";
pub static FILE_DB_NAME: &str = "application_db";

static DB_KEY_FILES: &str = "files";
static DB_KEY_BROKER: &str = "broker";

/// Shared application state
pub struct AppState {
    pub db: Arc<RwLock<MicroKV>>,
}

pub fn init_state() -> Arc<AppState> {
    // connect to db
    let some_path = Path::new(FILE_DB_PATH);
    let database: MicroKV = MicroKV::open_with_base_path(&FILE_DB_NAME, some_path.to_path_buf())
        .expect("Failed to create MicroKV from a stored file or create MicroKV for this file")
        .set_auto_commit(true);

    // init content in file db if file or keys do not exist
    init_file_state_if_necessary(&database);
    init_broker_state_if_necessary(&database);

    // create app state
    return Arc::new(AppState {
        db: Arc::new(RwLock::new(database)),
    });
}

fn init_file_state_if_necessary(db: &MicroKV) {
    if db.get_unwrap::<Files>(DB_KEY_FILES).is_err() {
        println!("need to update inital files state");

        if let Err(err) = db.put(DB_KEY_FILES, &Files::new()) {
            println!("Could not initialize file state: {err:?}")
        }
    }
}

fn init_broker_state_if_necessary(db: &MicroKV) {
    if db.get_unwrap::<Files>(DB_KEY_BROKER).is_err() {
        println!("need to update inital broker state");
        let broker = Broker {
            client_id: "eh-mqtt-client-1".to_string(),
            host: "localhost".to_string(),
            port: 1883,
            protocol: "mqtt://".to_string(),
            ..Default::default()
        };
        if let Err(err) = db.put(DB_KEY_BROKER, &broker) {
            println!("Could not initialize broker state: {err:?}")
        }
    }
}
