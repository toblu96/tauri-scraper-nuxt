
use tauri::{
    async_runtime::Mutex,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|initialize')`.
// where `awesome` is the plugin name.
async fn getFileVersion(path: String) -> String {
     // version_info::get_file_version("C:\\Program Files\\Microsoft VS Code\\libEGL.dll")
     let (b1, b2, b3, b4) =
     version_info::get_file_version("C:\\Program Files\\Microsoft VS Code\\Code.exe").unwrap();
    println!("File version is {}.{}.{}.{}", b1, b2, b3, b4);
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("file-version")
        .invoke_handler(tauri::generate_handler![getFileVersion])
        .build()
}
