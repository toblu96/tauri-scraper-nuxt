use crate::server::router::files::File;
use serde::{Deserialize, Serialize};
use serde_json::json;

// use async_trait::async_trait;
// use axum::http::request::Parts;
// use axum_core::extract::{FromRef, FromRequestParts};
// // use http::request::Parts;

// use std::convert::Infallible;

// // the extractor your library provides
// struct MyLibraryExtractor;

// #[async_trait]
// impl<S> FromRequestParts<S> for MyLibraryExtractor
// where
//     // keep `S` generic but require that it can produce a `MyLibraryState`
//     // this means users will have to implement `FromRef<UserState> for MyLibraryState`
//     MyLibraryState: FromRef<S>,
//     S: Send + Sync,
// {
//     type Rejection = Infallible;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         // get a `MyLibraryState` from a reference to the state
//         let state = MyLibraryState::from_ref(state);

//         // ...
//         // # todo!()
//         Ok(Self)
//     }
// }

// // the state your library needs
// struct MyLibraryState {
//     files: u32,
// }

static FILE_NAME: &str = "C:/Users/i40010702/Desktop/test/jsonDB.json";

#[derive(Serialize, Deserialize)]
struct FileDB {
    files: Vec<File>,
}

pub fn save_files_data(files: Vec<File>) -> Result<(), &'static str> {
    println!("Saved data to file..");
    let db_json_structure = json!({ "files": &files });
    // db_json_structure["files"] = Value::as_array(&files);

    // test - write file to disk
    let path = std::path::Path::new(FILE_NAME);
    if let Err(_err) = std::fs::create_dir_all(path.parent().expect("Invalid store path")) {
        return Err("Could not create folder on local disk. -> {_err:?}");
    }
    if let Err(_err) = std::fs::write(
        path,
        serde_json::to_string_pretty(&db_json_structure).unwrap(),
    ) {
        return Err("Could not write to file. -> {_err:?}");
    }

    Ok(())
}

pub fn load_files_data() -> Result<Vec<File>, &'static str> {
    println!("Load data from file..");
    let path = std::path::Path::new(FILE_NAME);
    let data = std::fs::read_to_string(path).expect("Unable to read file");

    let parsed: FileDB =
        serde_json::from_str(&data.to_string()).expect("JSON does not have correct format.");

    Ok(parsed.files)
}

// pub struct StoreBuilder {
//     pub file_db: FileDb,
// }

// impl StoreBuilder {
//     pub fn init() -> Self {
//         println!("Store initialized..");
//         Self {
//             file_db: FileDb::default(),
//         }
//     }
//     pub fn get_file_store(self) -> FileDb {
//         self.file_db
//     }
// }
// type FileDb = Arc<RwLock<HashMap<Uuid, crate::server::router::files::File>>>;
