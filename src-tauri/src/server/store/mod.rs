use microkv::MicroKV;
use std::{
    path::Path,
    sync::{Arc, RwLock},
};

static FILE_DB_PATH: &str = "C:/ProgramData/Tauri/EH Version Scraper";

/// Shared application state
pub struct AppState {
    pub db: Arc<RwLock<MicroKV>>,
}

pub fn init_state() -> Arc<AppState> {
    // connect to db
    let some_path = Path::new(FILE_DB_PATH);
    let database: MicroKV = MicroKV::open_with_base_path("application_db", some_path.to_path_buf())
        .expect("Failed to create MicroKV from a stored file or create MicroKV for this file")
        .set_auto_commit(true);

    // create app state
    return Arc::new(AppState {
        db: Arc::new(RwLock::new(database)),
    });
}
