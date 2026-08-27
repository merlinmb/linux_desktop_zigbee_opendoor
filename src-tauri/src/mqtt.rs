use rumqttc::{AsyncClient, MqttOptions, Event, Packet};
use serde_json::Value;
use std::time::Duration;
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::RwLock;
use crate::state::{AppState, ContactStatus};

pub struct MqttManager {
    client: Option<AsyncClient>,
    subscribed_topics: Arc<RwLock<HashSet<String>>>,
    broker: String,
    port: u16,
    client_name: String,
}

impl MqttManager {
    pub fn new() -> Self {
        MqttManager {
            client: None,
            subscribed_topics: Arc::new(RwLock::new(HashSet::new())),
            broker: String::new(),
            port: 1883,
            client_name: String::new(),
        }
    }

    pub async fn connect(
        &mut self,
        broker: String,
        port: u16,
        client_name: String,
        username: Option<String>,
        password: Option<String>,
        state: Arc<RwLock<AppState>>,
    ) -> Result<AsyncClient, Box<dyn std::error::Error>> {
        self.broker = broker.clone();
        self.port = port;
        self.client_name = client_name.clone();

        let mut options = MqttOptions::new(client_name.clone(), broker, port);
        options.set_keep_alive(Duration::from_secs(30));
        options.set_max_packet_size(10 * 1024, 10 * 1024);
        options.set_clean_session(true);

        if let (Some(user), Some(pass)) = (username, password) {
            options.set_credentials(user, pass);
            tracing::info!("MQTT configured with authentication");
        }

        let (client, connection) = AsyncClient::new(options, 10);
        self.client = Some(client.clone());

        let subscribed_topics = self.subscribed_topics.clone();
        let broker_clone = self.broker.clone();
        let port_clone = self.port;
        let client_name_clone = self.client_name.clone();

        // Spawn connection event loop in background
        tokio::spawn(async move {
            let mut connection = connection;
            let mut reconnect_delay = Duration::from_millis(500);
            let mut was_connected = false;

            loop {
                match connection.poll().await {
                    Ok(notification) => {
                        match notification {
                            Event::Incoming(Packet::Publish(pub_pkt)) => {
                                tracing::debug!("MQTT message received: {}", pub_pkt.topic);
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
                            Event::Incoming(Packet::ConnAck(conn_ack)) => {
                                tracing::info!(
                                    "MQTT connected successfully (session_present: {})",
                                    conn_ack.session_present
                                );
                                was_connected = true;
                                reconnect_delay = Duration::from_millis(500);
                            }
                            Event::Incoming(Packet::SubAck(_)) => {
                                tracing::debug!("MQTT subscription acknowledged");
                            }
                            Event::Incoming(Packet::Disconnect) => {
                                tracing::warn!("MQTT disconnected by broker");
                                was_connected = false;
                            }
                            Event::Outgoing(_) => {
                                tracing::trace!("MQTT outgoing packet");
                            }
                            Event::Incoming(Packet::PingResp) => {
                                tracing::trace!("MQTT ping response received");
                            }
                            _ => {
                                tracing::trace!("MQTT event: {:?}", notification);
                            }
                        }
                    }
                    Err(e) => {
                        if was_connected {
                            tracing::error!(
                                "MQTT connection lost: {}. Reconnecting in {:?}...",
                                e,
                                reconnect_delay
                            );
                            was_connected = false;
                        } else {
                            tracing::error!(
                                "MQTT connection failed: {}. Reconnecting in {:?}...",
                                e,
                                reconnect_delay
                            );
                        }

                        tokio::time::sleep(reconnect_delay).await;

                        // Exponential backoff with cap at 30 seconds
                        if reconnect_delay.as_millis() < 30000 {
                            reconnect_delay =
                                Duration::from_millis(reconnect_delay.as_millis() as u64 * 2);
                        }
                    }
                }
            }
        });

        Ok(client.clone())
    }

    pub async fn subscribe(&mut self, topic: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.subscribe(&topic, rumqttc::QoS::AtLeastOnce).await?;
            self.subscribed_topics.write().await.insert(topic.clone());
            tracing::info!("Subscribed to topic: {}", topic);
        } else {
            tracing::warn!("Cannot subscribe to {}: MQTT client not connected", topic);
        }
        Ok(())
    }

    pub async fn unsubscribe(&mut self, topic: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.unsubscribe(&topic).await?;
            self.subscribed_topics.write().await.remove(&topic);
            tracing::info!("Unsubscribed from topic: {}", topic);
        }
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.disconnect().await?;
            tracing::info!("MQTT disconnected");
        }
        self.client = None;
        self.subscribed_topics.write().await.clear();
        Ok(())
    }
}

pub fn parse_contact_message(
    payload: &[u8],
) -> Result<ContactStatus, Box<dyn std::error::Error + Send + Sync>> {
    let text = String::from_utf8(payload.to_vec())?;
    let json: Value = serde_json::from_str(&text)?;

    let contact = json
        .get("contact")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let battery = json
        .get("battery")
        .and_then(|v| v.as_u64())
        .map(|v| v as u8);

    let last_seen = json
        .get("last_seen")
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
