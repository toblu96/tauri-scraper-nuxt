#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use log::info;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    WindowEvent,
};

use clap::Parser;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::PluginBuilder;

mod logger;
mod server;

#[derive(Parser)]
#[command(name = "E+H File Version Monitor")]
#[command(author = "Tobias Blum <tobias.blum@endress.com>")]
#[command(version = "0.1")]
#[command(about = "Scrapes file versions from different file types.", long_about = None)]
struct Cli {
    /// Starts application in server only mode (http backend server)
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    // #[arg(short, long, default_value_t = true)]
    server: bool,

    /// Change default server port
    #[arg(short, long, default_value_t = 8000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    // get cli arguments
    let cli = Cli::parse();

    match cli.server {
        true => {
            if let Err(err) = logger::init() {
                println!("{err}");
            }
            info!("Starting Http Server..");
            server::start(cli.port).await;
        }
        false => {
            println!("Starting Tauri GUI..");
            start_gui().await
        }
    }
}

async fn start_gui() {
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

                    app.exit(0);
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
