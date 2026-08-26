#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod mqtt;
mod state;
mod commands;
mod window;

use tauri::{generate_handler, App, Manager};
use std::sync::Arc;
use tokio::sync::RwLock;
use state::AppState;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    tauri::Builder::default()
        .manage(Arc::new(RwLock::new(AppState::default())))
        .setup(|app| {
            let _app_handle = app.app_handle();

            #[cfg(target_os = "linux")]
            {
                if let Ok(main_window) = app.get_window("dock") {
                    let _ = window::set_x11_hints(&main_window);
                }
            }

            // Initialize config on startup
            if let Err(e) = config::init_config_dir() {
                eprintln!("Failed to initialize config directory: {}", e);
            }

            tracing::info!("Application started");

            Ok(())
        })
        .invoke_handler(generate_handler![
            commands::config::config_load,
            commands::config::config_save,
            commands::mqtt::mqtt_connect,
            commands::mqtt::mqtt_disconnect,
            commands::mqtt::mqtt_subscribe,
            commands::mqtt::mqtt_unsubscribe,
            commands::contacts::contacts_get_all,
            commands::contacts::contacts_count_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
