use rumqttc::{AsyncClient, MqttOptions};
use std::time::Duration;

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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut options = MqttOptions::new(client_name, broker, port);
        options.set_keep_alive(Duration::from_secs(30));

        let (client, _connection) = AsyncClient::new(options, 10);
        self.client = Some(client);

        Ok(())
    }

    pub async fn subscribe(&mut self, topic: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.subscribe(topic, rumqttc::QoS::AtMostOnce).await?;
        }
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.client {
            client.disconnect().await?;
        }
        self.client = None;
        Ok(())
    }
}
