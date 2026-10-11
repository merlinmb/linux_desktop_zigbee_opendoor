use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::{AppState, ContactStatus, MqttStatus};
use crate::mqtt::MqttManager;

#[tauri::command]
pub async fn mqtt_connect(
    broker: String,
    port: u16,
    client_name: String,
    username: Option<String>,
    password: Option<String>,
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<String, String> {
    let state_handle = state.inner().clone();
    let mut app_state = state.write().await;

    let mut manager = MqttManager::new();
    match manager
        .connect(
            broker.clone(),
            port,
            client_name.clone(),
            username,
            password,
            state_handle,
        )
        .await
    {
        Ok(_client) => {
            app_state.mqtt_status.broker = broker.clone();
            app_state.mqtt_status.client_name = client_name.clone();
            app_state.mqtt_status.connected = true;
            app_state.mqtt_manager = Some(manager);

            tracing::info!("MQTT connected to {} as {}", broker, client_name);
            Ok(format!("Connected to {} as {}", broker, client_name))
        }
        Err(e) => {
            tracing::error!("Failed to connect to MQTT: {}", e);
            Err(format!("Failed to connect: {}", e))
        }
    }
}

#[tauri::command]
pub async fn mqtt_disconnect(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.write().await;

    if let Some(mut manager) = app_state.mqtt_manager.take() {
        if let Err(e) = manager.disconnect().await {
            tracing::error!("Error disconnecting from MQTT: {}", e);
            return Err(format!("Disconnect error: {}", e));
        }
    }

    app_state.mqtt_status.connected = false;
    tracing::info!("MQTT disconnected");
    Ok(())
}

#[tauri::command]
pub async fn mqtt_status(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<MqttStatus, String> {
    let app_state = state.read().await;
    Ok(app_state.mqtt_status.clone())
}

#[tauri::command]
pub async fn mqtt_subscribe(
    topic: String,
    friendly_name: String,
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.write().await;

    if let Some(manager) = &mut app_state.mqtt_manager {
        manager
            .subscribe(topic.clone())
            .await
            .map_err(|e| format!("Failed to subscribe: {}", e))?;

        tracing::info!("Subscribed to: {} ({})", topic, friendly_name);
        let entry = app_state
            .contacts
            .entry(topic.clone())
            .or_insert_with(|| ContactStatus {
                topic: topic.clone(),
                friendly_name: friendly_name.clone(),
                contact: true,
                last_seen: None,
                battery: None,
                payload: String::new(),
            });
        entry.friendly_name = friendly_name;
        Ok(())
    } else {
        Err("MQTT not connected".to_string())
    }
}

#[tauri::command]
pub async fn mqtt_unsubscribe(
    topic: String,
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.write().await;

    if let Some(manager) = &mut app_state.mqtt_manager {
        manager
            .unsubscribe(topic.clone())
            .await
            .map_err(|e| format!("Failed to unsubscribe: {}", e))?;
    }

    if app_state.contacts.remove(&topic).is_some() {
        tracing::info!("Removed subscription for: {}", topic);
        Ok(())
    } else {
        Err("Topic not found".to_string())
    }
}
