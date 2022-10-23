use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[tauri::command]
/**
Get the current file version from a .exe or .dll file.
*/
async fn get_file_version(path: String) -> String {
    // version_info::get_file_version("C:\\Program Files\\Microsoft VS Code\\libEGL.dll")
    // version_info::get_file_version("C:\\Program Files\\Microsoft VS Code\\Code.exe")

    // make sure path has double backslashes on windows
    let version = version_info::get_file_version(&path);

    match version {
        Some((a, b, c, d)) => return format!("{}.{}.{}.{}", a, b, c, d),
        None => return "Could not read version.".to_string(),
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("file-version")
        .invoke_handler(tauri::generate_handler![get_file_version])
        .build()
}
