use rumqttc::{AsyncClient, MqttOptions, Event, Packet};
use serde_json::Value;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::{AppState, ContactStatus};

pub struct MqttManager {
    client: Option<AsyncClient>,
}

impl MqttManager {
    pub fn new() -> Self {
        MqttManager { client: None }
    }

    pub async fn connect(
        &mut self,
        broker: String,
        port: u16,
        client_name: String,
        state: Arc<RwLock<AppState>>,
    ) -> Result<AsyncClient, Box<dyn std::error::Error>> {
        let mut options = MqttOptions::new(client_name, broker, port);
        options.set_keep_alive(Duration::from_secs(30));
        options.set_max_packet_size(10 * 1024, 10 * 1024);

        let (client, connection) = AsyncClient::new(options, 10);
        self.client = Some(client.clone());

        // Spawn connection event loop in background
        tokio::spawn(async move {
            let mut connection = connection;
            loop {
                match connection.poll().await {
                    Ok(notification) => {
                        match notification {
                            Event::Incoming(Packet::Publish(pub_pkt)) => {
                                tracing::debug!("MQTT message: {}", pub_pkt.topic);
                                match parse_contact_message(&pub_pkt.payload) {
                                    Ok(mut status) => {
                                        let mut app_state = state.write().await;
                                        let friendly_name = app_state
                                            .contacts
                                            .get(&pub_pkt.topic)
                                            .map(|c| c.friendly_name.clone())
                                            .unwrap_or_else(|| pub_pkt.topic.clone());
                                        status.topic = pub_pkt.topic.clone();
                                        status.friendly_name = friendly_name;
                                        app_state.contacts.insert(pub_pkt.topic.clone(), status);
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to parse MQTT payload on {}: {}",
                                            pub_pkt.topic,
                                            e
                                        );
                                    }
                                }
                            }
                            Event::Incoming(Packet::ConnAck(_)) => {
                                tracing::info!("MQTT connected");
                            }
                            Event::Incoming(Packet::Disconnect) => {
                                tracing::warn!("MQTT disconnected");
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        tracing::error!("MQTT error: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(client.clone())
    }

    pub async fn subscribe(&mut self, topic: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.subscribe(&topic, rumqttc::QoS::AtMostOnce).await?;
            tracing::info!("Subscribed to: {}", topic);
        }
        Ok(())
    }

    pub async fn unsubscribe(&mut self, topic: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.unsubscribe(&topic).await?;
            tracing::info!("Unsubscribed from: {}", topic);
        }
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.disconnect().await?;
            tracing::info!("MQTT disconnected");
        }
        self.client = None;
        Ok(())
    }
}

pub fn parse_contact_message(
    payload: &[u8],
) -> Result<ContactStatus, Box<dyn std::error::Error + Send + Sync>> {
    let text = String::from_utf8(payload.to_vec())?;
    let json: Value = serde_json::from_str(&text)?;

    let contact = json.get("contact")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let battery = json.get("battery")
        .and_then(|v| v.as_u64())
        .map(|v| v as u8);

    let last_seen = json.get("last_seen")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(ContactStatus {
        topic: String::new(),
        friendly_name: String::new(),
        contact,
        last_seen,
        battery,
        payload: text,
    })
}
