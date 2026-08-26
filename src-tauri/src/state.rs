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
    pub config_loaded: bool,
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
            config_loaded: false,
        }
    }
}

impl AppState {
    pub fn count_open(&self) -> usize {
        self.contacts.values().filter(|c| !c.contact).count()
    }

    pub fn get_open_contacts(&self) -> Vec<ContactStatus> {
        self.contacts
            .values()
            .filter(|c| !c.contact)
            .cloned()
            .collect()
    }
}
