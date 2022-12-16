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

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    ops::Add,
    sync::{Arc, RwLock},
};

use clap::Parser;

#[derive(Parser)]
#[command(name = "E+H Version Scraper")]
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
            println!("Starting Http Server..");
            start_server(cli.port).await;
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

async fn start_server(port: u16) {
    // build local state
    let db = Db::default();

    // init values
    db.write().unwrap().insert("counter".to_string(), 0.clone());

    // build our application with a route
    let app = Router::new().route("/", get(handler)).with_state(db);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(State(db): State<Db>) -> Result<impl IntoResponse, StatusCode> {
    // get counter state
    let mut counter = db
        .read()
        .unwrap()
        .get("counter")
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    counter += 1;

    // store new value to db
    db.write()
        .unwrap()
        .insert("counter".to_string(), counter.clone())
        .expect("Error while insterting in db..");

    let todo = Todo {
        id: "öjasdf790asfd".to_string(),
        text: "my first todo topic".to_string(),
        completed: false,
        count: counter,
    };

    Ok(Json(todo))
}

type Db = Arc<RwLock<HashMap<String, u16>>>;

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
    count: u16,
}

// async fn handler() -> Html<&'static str> {

//     Html("<h1>Hello, World!</h1><p>Whats up?</p>")
// }
