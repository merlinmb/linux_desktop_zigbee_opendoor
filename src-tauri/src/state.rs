use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::mqtt::MqttManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactStatus {
    pub topic: String,
    pub friendly_name: String,
    pub contact: bool,
    pub last_seen: Option<String>,
    pub battery: Option<u8>,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttStatus {
    pub connected: bool,
    pub broker: String,
    pub client_name: String,
}

pub struct AppState {
    pub contacts: HashMap<String, ContactStatus>,
    pub mqtt_status: MqttStatus,
    pub mqtt_manager: Option<MqttManager>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            contacts: HashMap::new(),
            mqtt_status: MqttStatus {
                connected: false,
                broker: String::new(),
                client_name: String::new(),
            },
            mqtt_manager: None,
        }
    }
}

impl AppState {
    pub fn count_open(&self) -> usize {
        self.contacts.values().filter(|c| !c.contact).count()
    }
}
