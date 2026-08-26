use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::{AppState, ContactStatus};

#[tauri::command]
pub async fn contacts_get_all(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<ContactStatus>, String> {
    let app_state = state.read().await;
    Ok(app_state.contacts.values().cloned().collect())
}

#[tauri::command]
pub async fn contacts_count_open(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<usize, String> {
    let app_state = state.read().await;
    Ok(app_state.count_open())
}
