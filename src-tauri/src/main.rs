#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod mqtt;
mod state;
mod commands;
mod window;

use tauri::{generate_handler, Manager};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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
                if let Some(main_window) = app.get_window("dock") {
                    let _ = window::set_x11_hints(&main_window);
                }
            }

            // Initialize config on startup
            if let Err(e) = config::init_config_dir() {
                eprintln!("Failed to initialize config directory: {}", e);
            }

            // Restore saved window position/always-on-top, and persist new
            // positions as the user drags the (decoration-less) window.
            //
            // Window managers commonly force-relocate an always-on-top,
            // borderless window back onto the remaining display when its
            // monitor is unplugged. We must not persist that forced move
            // (it would clobber the position we want to return to), and
            // since Tauri 1.x/X11 has no display-added event, we poll
            // `available_monitors()` to notice when the monitor comes back
            // and actively move the window back ourselves.
            match config::load_config() {
                Ok(cfg) => {
                    if let Some(main_window) = app.get_window("dock") {
                        let _ = main_window.set_position(tauri::Position::Physical(
                            tauri::PhysicalPosition {
                                x: cfg.window.x,
                                y: cfg.window.y,
                            },
                        ));
                        let _ = main_window.set_always_on_top(cfg.window.always_on_top);

                        let suppress_save = Arc::new(AtomicBool::new(false));
                        let last_saved = Arc::new(Mutex::new(Instant::now()));

                        let suppress_save_for_move = suppress_save.clone();
                        main_window.clone().on_window_event(move |event| {
                            if let tauri::WindowEvent::Moved(position) = event {
                                if suppress_save_for_move.load(Ordering::Relaxed) {
                                    return;
                                }
                                let mut last = last_saved.lock().unwrap();
                                if last.elapsed() < Duration::from_millis(300) {
                                    return;
                                }
                                *last = Instant::now();

                                if let Ok(mut cfg) = config::load_config() {
                                    cfg.window.x = position.x;
                                    cfg.window.y = position.y;
                                    if let Err(e) = config::save_config(&cfg) {
                                        eprintln!("Failed to save window position: {}", e);
                                    }
                                }
                            }
                        });

                        let poll_window = main_window.clone();
                        std::thread::spawn(move || {
                            let mut full_count = poll_window
                                .available_monitors()
                                .map(|m| m.len())
                                .unwrap_or(1)
                                .max(1);
                            let mut degraded = false;

                            loop {
                                std::thread::sleep(Duration::from_secs(2));

                                let current = match poll_window.available_monitors() {
                                    Ok(monitors) => monitors.len(),
                                    Err(_) => continue,
                                };

                                if current > full_count {
                                    full_count = current;
                                }

                                if current < full_count {
                                    // A monitor disappeared. The WM may force-move
                                    // the window onto the remaining display; don't
                                    // let that clobber the saved position.
                                    degraded = true;
                                    suppress_save.store(true, Ordering::Relaxed);
                                } else if degraded {
                                    // The missing monitor is back. Restore the
                                    // saved position instead of leaving the window
                                    // wherever the WM parked it.
                                    suppress_save.store(true, Ordering::Relaxed);
                                    std::thread::sleep(Duration::from_millis(500));

                                    if let Ok(cfg) = config::load_config() {
                                        let _ = poll_window.set_position(
                                            tauri::Position::Physical(tauri::PhysicalPosition {
                                                x: cfg.window.x,
                                                y: cfg.window.y,
                                            }),
                                        );
                                    }

                                    std::thread::sleep(Duration::from_millis(300));
                                    degraded = false;
                                    suppress_save.store(false, Ordering::Relaxed);
                                }
                            }
                        });
                    }
                }
                Err(e) => eprintln!("Failed to load config for window setup: {}", e),
            }

            tracing::info!("Application started");

            Ok(())
        })
        .on_window_event(|global_window_event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = global_window_event.event() {
                let app = global_window_event.window().app_handle();
                let state = app.state::<Arc<RwLock<AppState>>>();

                let state_handle = state.inner().clone();
                tokio::spawn(async move {
                    let mut app_state = state_handle.write().await;
                    if let Some(mut manager) = app_state.mqtt_manager.take() {
                        let _ = manager.disconnect().await;
                        tracing::info!("MQTT disconnected on app close");
                    }
                });

                api.prevent_close();

                let window = global_window_event.window().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    window.close().unwrap();
                });
            }
        })
        .invoke_handler(generate_handler![
            commands::config::config_load,
            commands::config::config_save,
            commands::mqtt::mqtt_connect,
            commands::mqtt::mqtt_disconnect,
            commands::mqtt::mqtt_status,
            commands::mqtt::mqtt_subscribe,
            commands::mqtt::mqtt_unsubscribe,
            commands::contacts::contacts_get_all,
            commands::contacts::contacts_count_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
