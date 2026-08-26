use crate::config::{self, AppConfig};

#[tauri::command]
pub async fn config_load() -> Result<AppConfig, String> {
    config::load_config()
        .map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
pub async fn config_save(new_config: AppConfig) -> Result<(), String> {
    config::save_config(&new_config)
        .map_err(|e| format!("Failed to save config: {}", e))
}
