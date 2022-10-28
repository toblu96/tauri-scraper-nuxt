#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    WindowEvent,
};

use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_fs_watch::Watcher;
use tauri_plugin_store::PluginBuilder;

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide Dashboard");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::RightClick {
                position: _,
                size: _,
                ..
            } => {
                // update dashboard menu text
                if app.get_window("main").unwrap().is_visible().unwrap() {
                    app.tray_handle()
                        .get_item("hide")
                        .set_title("Hide Dashboard")
                        .unwrap();
                } else {
                    app.tray_handle()
                        .get_item("hide")
                        .set_title("Show Dashboard")
                        .unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    // use app.exit(0) instead to perform cleanup before closing the application
                    // std::process::exit(0);

                    app.exit(1);
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .plugin(PluginBuilder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(Watcher::default())
        .plugin(tauri_plugin_mqtt_client::init())
        .plugin(tauri_plugin_file_version::init())
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    _app_handle.get_window("main").unwrap().hide().unwrap();
                }
                _ => {}
            },
            tauri::RunEvent::ExitRequested { api, .. } => {
                println!("exit requested");
                api.prevent_exit();
            }
            _ => {}
        })
}
