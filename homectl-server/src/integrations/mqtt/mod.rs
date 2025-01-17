mod utils;

use anyhow::{Context, Result};
use async_trait::async_trait;
use homectl_types::{
    custom_integration::CustomIntegration,
    device::Device,
    event::{Message, TxEventChannel},
    integration::{IntegrationActionPayload, IntegrationId},
};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

use crate::integrations::mqtt::utils::mqtt_to_homectl;

use self::utils::homectl_to_mqtt;

#[derive(Debug, Deserialize, Clone)]
pub struct MqttConfig {
    host: String,
    port: u16,
    topic_set: String,
    topic: String,
    id_field: Option<String>,
    name_field: Option<String>,
    color_field: Option<String>,
    cct_field: Option<String>,
    power_field: Option<String>,
    brightness_field: Option<String>,
    sensor_value_field: Option<String>,
    transition_ms_field: Option<String>,
}

pub struct Mqtt {
    id: IntegrationId,
    event_tx: TxEventChannel,
    config: MqttConfig,
    client: Option<AsyncClient>,
}

#[async_trait]
impl CustomIntegration for Mqtt {
    fn new(id: &IntegrationId, config: &config::Value, event_tx: TxEventChannel) -> Result<Self> {
        let config = config
            .clone()
            .try_into()
            .context("Failed to deserialize config of Mqtt integration")?;

        Ok(Mqtt {
            id: id.clone(),
            config,
            event_tx,
            client: None,
        })
    }

    async fn register(&mut self) -> Result<()> {
        println!("registered mqtt integration {}", self.id);

        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        let mut options =
            MqttOptions::new(self.id.clone(), self.config.host.clone(), self.config.port);
        options.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        client
            .subscribe(self.config.topic.replace("{id}", "+"), QoS::AtMostOnce)
            .await?;

        self.client = Some(client);

        let id = self.id.clone();
        let event_tx = self.event_tx.clone();
        let config_clone = Arc::new(self.config.clone());

        task::spawn(async move {
            while let Ok(notification) = eventloop.poll().await {
                let id = id.clone();
                let event_tx = event_tx.clone();
                let config_clone = Arc::clone(&config_clone);

                let res = (|| async move {
                    if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                        let device = mqtt_to_homectl(&msg.payload, id, &config_clone)?;
                        event_tx.send(Message::IntegrationDeviceRefresh { device })
                    }

                    Ok::<(), Box<dyn std::error::Error>>(())
                })()
                .await;

                if let Err(e) = res {
                    eprintln!("MQTT error: {:?}", e);
                }
            }
        });

        println!("started mqtt integration {}", self.id);

        Ok(())
    }

    async fn set_integration_device_state(&mut self, device: &Device) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .expect("Expected self.client to be set in start phase");
        let topic = self
            .config
            .topic_set
            .replace("{id}", &device.id.to_string());
        let mqtt_device = homectl_to_mqtt(device.clone(), &self.config)?;
        let json = serde_json::to_string(&mqtt_device)?;
        client.publish(topic, QoS::AtLeastOnce, true, json).await?;
        Ok(())
    }

    async fn run_integration_action(&mut self, _: &IntegrationActionPayload) -> Result<()> {
        // do nothing
        Ok(())
    }
}
