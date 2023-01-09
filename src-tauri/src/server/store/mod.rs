use super::router::{files::File, settings::Broker};
use microkv::MicroKV;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, RwLock},
};

static FILE_NAME: &str = "C:/ProgramData/Tauri/EH Version Scraper/backendDB.json";

static FILE_DB_PATH: &str = "C:/ProgramData/Tauri/EH Version Scraper";

#[derive(Serialize, Deserialize, Debug, Default)]
struct FileDB {
    #[serde(default)]
    files: Vec<File>,
    #[serde(default)]
    broker: Broker,
}

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

pub fn save_files_data(files: Vec<File>) -> Result<(), &'static str> {
    let path = std::path::Path::new(FILE_NAME);
    let mut file_db_content: FileDB = FileDB::default();

    if path.exists() {
        // just open the existing file and load its content
        let data = std::fs::read_to_string(path).expect("Could not read content from file.");
        file_db_content = serde_json::from_str(&data).expect("Could not extract json from file");
    } else {
        // create folder to store file
        if let Err(_err) = std::fs::create_dir_all(path.parent().expect("Invalid store path")) {
            return Err("Could not create folder on local disk. -> {_err:?}");
        }
    }

    file_db_content.files = files;

    // write broker data to disk
    if let Err(_err) = std::fs::write(
        path,
        serde_json::to_string_pretty(&file_db_content).unwrap(),
    ) {
        return Err("Could not write to file. -> {_err:?}");
    }

    println!("Saved file data to file..");
    Ok(())
}

pub fn load_files_data() -> Result<Vec<File>, std::io::Error> {
    println!("Load file data from file..");
    let path = std::path::Path::new(FILE_NAME);
    let data = std::fs::read_to_string(path)?; //.expect("Unable to read file");

    let parsed: FileDB = serde_json::from_str(&data)?; //.expect("JSON does not have correct format.");

    Ok(parsed.files)
}

pub fn save_broker_data(broker: Broker) -> Result<(), &'static str> {
    let path = std::path::Path::new(FILE_NAME);
    let mut file_db_content: FileDB = FileDB::default();

    if path.exists() {
        // just open the existing file and load its content
        let data = std::fs::read_to_string(path).expect("Could not read content from file.");
        file_db_content = serde_json::from_str(&data).expect("Could not extract json from file");
    } else {
        // create folder to store file
        if let Err(_err) = std::fs::create_dir_all(path.parent().expect("Invalid store path")) {
            return Err("Could not create folder on local disk. -> {_err:?}");
        }
    }

    file_db_content.broker = broker;

    // write broker data to disk
    if let Err(_err) = std::fs::write(
        path,
        serde_json::to_string_pretty(&file_db_content).unwrap(),
    ) {
        return Err("Could not write to file. -> {_err:?}");
    }

    println!("Saved broker data to file..");
    Ok(())
}

pub fn load_broker_data() -> Result<Broker, std::io::Error> {
    println!("Load broker data from file..");
    let path = std::path::Path::new(FILE_NAME);
    let data = std::fs::read_to_string(path)?;

    let parsed: FileDB = serde_json::from_str(&data)?;

    Ok(parsed.broker)
}
