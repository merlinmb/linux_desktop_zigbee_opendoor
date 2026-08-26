use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::AppState;

#[tauri::command]
pub async fn mqtt_connect(
    broker: String,
    port: u16,
    client_name: String,
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<String, String> {
    let mut app_state = state.write().await;
    app_state.mqtt_status.broker = broker.clone();
    app_state.mqtt_status.client_name = client_name.clone();
    app_state.mqtt_status.connected = true;

    Ok(format!("Connected to {} as {}", broker, client_name))
}

#[tauri::command]
pub async fn mqtt_disconnect(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.write().await;
    app_state.mqtt_status.connected = false;
    Ok(())
}

#[tauri::command]
pub async fn mqtt_subscribe(
    topic: String,
    friendly_name: String,
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.write().await;
    app_state.contacts.insert(
        topic.clone(),
        crate::state::ContactStatus {
            topic,
            friendly_name,
            contact: true,
            last_seen: None,
            battery: None,
            payload: String::new(),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn mqtt_unsubscribe(
    topic: String,
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.write().await;
    app_state.contacts.remove(&topic);
    Ok(())
}
