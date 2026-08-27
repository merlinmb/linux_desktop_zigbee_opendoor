use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub client_name: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

impl Default for MqttConfig {
    fn default() -> Self {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        let client_name = if hostname.len() <= 8 {
            format!("door_{}", hostname)
        } else {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            hostname.hash(&mut hasher);
            let hash = hasher.finish();
            format!("door_{:x}", hash & 0xffffff)
        };

        MqttConfig {
            broker: "192.168.1.1".to_string(),
            port: 1883,
            client_name,
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub brightness: u8,
    pub flip_screen: bool,
    pub scroll_interval_ms: u32,
    #[serde(default = "default_clock_font_size")]
    pub clock_font_size: u16,
    #[serde(default = "default_contact_name_font_size")]
    pub contact_name_font_size: u16,
    #[serde(default = "default_highlight_duration_ms")]
    pub highlight_duration_ms: u32,
}

fn default_clock_font_size() -> u16 {
    156
}

fn default_contact_name_font_size() -> u16 {
    24
}

fn default_highlight_duration_ms() -> u32 {
    3000
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            brightness: 100,
            flip_screen: false,
            scroll_interval_ms: 1750,
            clock_font_size: 156,
            contact_name_font_size: 24,
            highlight_duration_ms: 3000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub x: i32,
    pub y: i32,
    pub always_on_top: bool,
    pub transparency: u8,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            x: 0,
            y: 0,
            always_on_top: true,
            transparency: 255,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mqtt: MqttConfig,
    pub display: DisplayConfig,
    pub window: WindowConfig,
    pub subscriptions: std::collections::HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            mqtt: MqttConfig::default(),
            display: DisplayConfig::default(),
            window: WindowConfig::default(),
            subscriptions: std::collections::HashMap::new(),
        }
    }
}

pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "merlin", "opendoor-monitor") {
        Ok(proj_dirs.config_dir().to_path_buf())
    } else {
        Err("Could not determine config directory".into())
    }
}

pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = get_config_dir()?;
    path.push("config.toml");
    Ok(path)
}

pub fn init_config_dir() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    Ok(())
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        let default_config = AppConfig::default();
        save_config(&default_config)?;
        Ok(default_config)
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(&config)?;
    fs::write(&config_path, content)?;
    Ok(())
}
