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
        let resub_client = client.clone();

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
                                    Ok(parsed) => {
                                        let mut app_state = state.write().await;
                                        let existing = app_state.contacts.get(&pub_pkt.topic);
                                        let friendly_name = existing
                                            .map(|c| c.friendly_name.clone())
                                            .unwrap_or_else(|| pub_pkt.topic.clone());
                                        // Partial reports (e.g. battery-only heartbeats) omit
                                        // "contact"; fall back to the last known state instead
                                        // of assuming closed, so a real open door isn't cleared.
                                        let contact = parsed
                                            .contact
                                            .or_else(|| existing.map(|c| c.contact))
                                            .unwrap_or(true);
                                        let battery = parsed.battery.or_else(|| existing.and_then(|c| c.battery));
                                        let last_seen = parsed.last_seen.or_else(|| existing.and_then(|c| c.last_seen.clone()));
                                        let status = ContactStatus {
                                            topic: pub_pkt.topic.clone(),
                                            friendly_name,
                                            contact,
                                            last_seen,
                                            battery,
                                            payload: parsed.payload,
                                        };
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
                                state.write().await.mqtt_status.connected = true;

                                // clean_session=true means the broker forgets our
                                // subscriptions on every disconnect, including
                                // transient reconnects. Re-issue them all here so a
                                // network blip doesn't silently stop new contact
                                // updates from ever arriving again.
                                let topics: Vec<String> =
                                    subscribed_topics.read().await.iter().cloned().collect();
                                for topic in topics {
                                    if let Err(e) = resub_client
                                        .subscribe(&topic, rumqttc::QoS::AtLeastOnce)
                                        .await
                                    {
                                        tracing::error!(
                                            "Failed to resubscribe to {}: {}",
                                            topic,
                                            e
                                        );
                                    } else {
                                        tracing::info!("Resubscribed to topic: {}", topic);
                                    }
                                }
                            }
                            Event::Incoming(Packet::SubAck(_)) => {
                                tracing::debug!("MQTT subscription acknowledged");
                            }
                            Event::Incoming(Packet::Disconnect) => {
                                tracing::warn!("MQTT disconnected by broker");
                                was_connected = false;
                                state.write().await.mqtt_status.connected = false;
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
                            state.write().await.mqtt_status.connected = false;
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

pub struct ParsedContact {
    pub contact: Option<bool>,
    pub battery: Option<u8>,
    pub last_seen: Option<String>,
    pub payload: String,
}

pub fn parse_contact_message(
    payload: &[u8],
) -> Result<ParsedContact, Box<dyn std::error::Error + Send + Sync>> {
    let text = String::from_utf8(payload.to_vec())?;
    let json: Value = serde_json::from_str(&text)?;

    // Some devices publish partial reports (e.g. battery-only heartbeats)
    // that omit "contact". Leave it as None in that case so the caller can
    // keep the previously known open/closed state instead of assuming closed.
    let contact = json.get("contact").and_then(|v| v.as_bool());

    let battery = json
        .get("battery")
        .and_then(|v| v.as_u64())
        .map(|v| v as u8);

    let last_seen = json
        .get("last_seen")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(ParsedContact {
        contact,
        last_seen,
        battery,
        payload: text,
    })
}
