use paho_mqtt as mqtt;
use paris::{info, warn};
use serde_json::json;
use std::time::Duration;
use time::{format_description, OffsetDateTime};
use tokio::{sync::OnceCell, task};

use crate::ambient_light::{ColorCalibration, ConfigManager};

const DISPLAY_TOPIC: &'static str = "display-ambient-light/display";
const DESKTOP_TOPIC: &'static str = "display-ambient-light/desktop";
const COLOR_CALIBRATION: &'static str = "display-ambient-light/desktop/color-calibration";

pub struct MqttRpc {
    client: mqtt::AsyncClient,
    // change_display_brightness_tx: broadcast::Sender<display::DisplayBrightness>,
    // message_tx: broadcast::Sender<models::CmdMqMessage>,
}

impl MqttRpc {
    pub async fn global() -> &'static Self {
        static MQTT_RPC: OnceCell<MqttRpc> = OnceCell::const_new();

        MQTT_RPC
            .get_or_init(|| async {
                let mqtt_rpc = MqttRpc::new().await.unwrap();
                mqtt_rpc.initialize().await.unwrap();
                mqtt_rpc
            })
            .await
    }

    pub async fn new() -> anyhow::Result<Self> {
        let client = mqtt::AsyncClient::new("tcp://192.168.31.11:1883")
            .map_err(|err| anyhow::anyhow!("can not create MQTT client. {:?}", err))?;

        client.set_connected_callback(|client| {
            info!("MQTT server connected.");

            client.subscribe("display-ambient-light/board/#", mqtt::QOS_1);

            client.subscribe(format!("{}/#", DISPLAY_TOPIC), mqtt::QOS_1);
        });
        client.set_connection_lost_callback(|_| {
            info!("MQTT server connection lost.");
        });
        client.set_disconnected_callback(|_, a1, a2| {
            info!("MQTT server disconnected. {:?} {:?}", a1, a2);
        });

        let mut last_will_payload = serde_json::Map::new();
        last_will_payload.insert("message".to_string(), json!("offline"));
        last_will_payload.insert(
            "time".to_string(),
            serde_json::Value::String(
                OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::iso8601::Iso8601::DEFAULT)
                    .unwrap()
                    .to_string(),
            ),
        );

        let last_will = mqtt::Message::new(
            format!("{}/status", DESKTOP_TOPIC),
            serde_json::to_string(&last_will_payload)
                .unwrap()
                .as_bytes(),
            mqtt::QOS_1,
        );

        let connect_options = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(5))
            .will_message(last_will)
            .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(5))
            .finalize();

        let token = client.connect(connect_options);

        token.await.map_err(|err| {
            anyhow::anyhow!(
                "can not connect MQTT server. wait for connect token failed. {:?}",
                err
            )
        })?;

        // let (change_display_brightness_tx, _) =
        //     broadcast::channel::<display::DisplayBrightness>(16);
        // let (message_tx, _) = broadcast::channel::<models::CmdMqMessage>(32);
        Ok(Self { client })
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        self.broadcast_desktop_online();
        Self::publish_color_calibration_worker();
        anyhow::Ok(())
    }

    fn publish_color_calibration_worker() {
        tokio::spawn(async move {
            let mqtt = Self::global().await;
            let config_manager = ConfigManager::global().await;
            let mut config_receiver = config_manager.clone_config_update_receiver();

            let config = config_manager.configs().await;
            if let Err(err) = mqtt
                .publish_color_calibration(config.color_calibration)
                .await
            {
                warn!("can not publish color calibration. {}", err);
            }

            while config_receiver.changed().await.is_ok() {
                let config = config_receiver.borrow().clone();
                if let Err(err) = mqtt
                    .publish_color_calibration(config.color_calibration)
                    .await
                {
                    warn!("can not publish color calibration. {}", err);
                }
            }
        });
    }

    fn broadcast_desktop_online(&self) {
        let client = self.client.to_owned();
        task::spawn(async move {
            loop {
                match OffsetDateTime::now_utc()
                    .format(&format_description::well_known::Iso8601::DEFAULT)
                {
                    Ok(now_str) => {
                        let msg = mqtt::Message::new(
                            "display-ambient-light/desktop/online",
                            now_str.as_bytes(),
                            mqtt::QOS_0,
                        );
                        match client.publish(msg).await {
                            Ok(_) => {}
                            Err(error) => {
                                warn!("can not publish last online time. {}", error)
                            }
                        }
                    }
                    Err(error) => {
                        warn!("can not get time for now. {}", error);
                    }
                }
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
    }

     pub async fn publish_color_calibration(&self, payload: ColorCalibration) -> anyhow::Result<()> {
        self.client
            .publish(mqtt::Message::new(
                COLOR_CALIBRATION,
                payload.to_bytes(),
                mqtt::QOS_1,
            ))
            .await
            .map_err(|error| anyhow::anyhow!("mqtt publish color calibration failed. {}", error))
    }
}
