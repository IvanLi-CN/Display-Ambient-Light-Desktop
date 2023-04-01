use paho_mqtt as mqtt;
use paris::{error, info, warn};
use serde_json::json;
use std::time::Duration;
use time::{format_description, OffsetDateTime};
use tokio::{sync::OnceCell, task};

const DISPLAY_TOPIC: &'static str = "display-ambient-light/display";
const DESKTOP_TOPIC: &'static str = "display-ambient-light/desktop";
const DISPLAY_BRIGHTNESS_TOPIC: &'static str = "display-ambient-light/board/brightness";
const BOARD_SEND_CMD: &'static str = "display-ambient-light/board/cmd";

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
        client.set_connection_lost_callback(|client| {
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

    pub async fn listen(&self) {
        // let change_display_brightness_tx2 = self.change_display_brightness_tx.clone();
        // let message_tx_cloned = self.message_tx.clone();

        // let mut stream = self.client.to_owned().get_stream(100);

        // while let Some(notification) = stream.next().await {
        //     match notification {
        //         Some(notification) => match notification.topic() {
        //             DISPLAY_BRIGHTNESS_TOPIC => {
        //                 let payload_text = String::from_utf8(notification.payload().to_vec());
        //                 match payload_text {
        //                     Ok(payload_text) => {
        //                         let display_brightness: Result<display::DisplayBrightness, _> =
        //                             serde_json::from_str(payload_text.as_str());
        //                         match display_brightness {
        //                             Ok(display_brightness) => {
        //                                 match change_display_brightness_tx2.send(display_brightness)
        //                                 {
        //                                     Ok(_) => {}
        //                                     Err(err) => {
        //                                         warn!(
        //                                                 "can not send display brightness to channel. {:?}",
        //                                                 err
        //                                             );
        //                                     }
        //                                 }
        //                             }
        //                             Err(err) => {
        //                                 warn!(
        //                                     "can not parse display brightness from payload. {:?}",
        //                                     err
        //                                 );
        //                             }
        //                         }
        //                     }
        //                     Err(err) => {
        //                         warn!("can not parse display brightness from payload. {:?}", err);
        //                     }
        //                 }
        //             }
        //             BOARD_SEND_CMD => {
        //                 let payload_text = String::from_utf8(notification.payload().to_vec());
        //                 match payload_text {
        //                     Ok(payload_text) => {
        //                         let message: Result<models::CmdMqMessage, _> =
        //                             serde_json::from_str(payload_text.as_str());
        //                         match message {
        //                             Ok(message) => match message_tx_cloned.send(message) {
        //                                 Ok(_) => {}
        //                                 Err(err) => {
        //                                     warn!("can not send message to channel. {:?}", err);
        //                                 }
        //                             },
        //                             Err(err) => {
        //                                 warn!("can not parse message from payload. {:?}", err);
        //                             }
        //                         }
        //                     }
        //                     Err(err) => {
        //                         warn!("can not parse message from payload. {:?}", err);
        //                     }
        //                 }
        //             }
        //             _ => {}
        //         },
        //         _ => {
        //             warn!("can not get notification from MQTT server.");
        //         }
        //     }
        // }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        // self.subscribe_board()?;
        // self.subscribe_display()?;
        self.broadcast_desktop_online();
        anyhow::Ok(())
    }

    fn subscribe_board(&self) -> anyhow::Result<()> {
        self.client
            .subscribe("display-ambient-light/board/#", mqtt::QOS_1)
            .wait()
            .map_err(|err| anyhow::anyhow!("subscribe board failed. {:?}", err))
            .map(|_| ())
    }
    fn subscribe_display(&self) -> anyhow::Result<()> {
        self.client
            .subscribe(format!("{}/#", DISPLAY_TOPIC), mqtt::QOS_1)
            .wait()
            .map_err(|err| anyhow::anyhow!("subscribe board failed. {:?}", err))
            .map(|_| ())
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

    pub async fn publish_led_sub_pixels(&self, payload: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .publish(mqtt::Message::new(
                "display-ambient-light/desktop/colors",
                payload,
                mqtt::QOS_1,
            ))
            .await
            .map_err(|error| anyhow::anyhow!("mqtt publish failed. {}", error))
    }

    // pub fn subscribe_change_display_brightness_rx(
    //     &self,
    // ) -> broadcast::Receiver<display::DisplayBrightness> {
    //     self.change_display_brightness_tx.subscribe()
    // }
    pub async fn publish_desktop_cmd(&self, field: &str, payload: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .publish(mqtt::Message::new(
                format!("{}/{}", DESKTOP_TOPIC, field),
                payload,
                mqtt::QOS_1,
            ))
            .await
            .map_err(|error| anyhow::anyhow!("mqtt publish failed. {}", error))
    }
}
