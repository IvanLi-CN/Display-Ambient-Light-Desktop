use std::sync::Arc;

use paris::warn;
use tauri::async_runtime::RwLock;
use tokio::sync::watch;

use crate::{
    ambient_light::{config, ConfigManager},
    rpc::MqttRpc,
    screenshot_manager::ScreenshotManager,
};

use itertools::Itertools;

pub struct LedColorsPublisher {
    rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
}

impl LedColorsPublisher {
    pub async fn global() -> &'static Self {
        static LED_COLORS_PUBLISHER_GLOBAL: tokio::sync::OnceCell<LedColorsPublisher> =
            tokio::sync::OnceCell::const_new();

        let (tx, rx) = watch::channel(Vec::new());

        LED_COLORS_PUBLISHER_GLOBAL
            .get_or_init(|| async {
                LedColorsPublisher {
                    rx: Arc::new(RwLock::new(rx)),
                    tx: Arc::new(RwLock::new(tx)),
                }
            })
            .await
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let tx = tx.write().await;

            let screenshot_manager = ScreenshotManager::global().await;
            let config_manager = ConfigManager::global().await;

            loop {
                let configs = config_manager.configs().await;
                let channels = screenshot_manager.channels.read().await;

                let display_ids = configs
                    .strips
                    .iter()
                    .map(|c| c.display_id)
                    .unique().collect::<Vec<_>>();

                let mut colors_configs = Vec::new();

                for display_id in display_ids {
                    let led_strip_configs: Vec<_> = configs
                        .strips
                        .iter()
                        .filter(|c| c.display_id == display_id)
                        .collect();

                    let rx = channels.get(&display_id);

                    if rx.is_none() {
                        warn!("no channel for display_id: {}", display_id);
                        continue;
                    }

                    let rx = rx.unwrap();

                    if led_strip_configs.len() == 0 {
                        warn!("no led strip config for display_id: {}", display_id);
                        continue;
                    }

                    let mut rx = rx.clone();

                    if rx.changed().await.is_ok() {
                        let screenshot = rx.borrow().clone();
                        // log::info!("screenshot updated: {:?}", display_id);

                        let points: Vec<_> = led_strip_configs
                            .iter()
                            .map(|config| screenshot.get_sample_points(&config))
                            .flatten()
                            .collect();

                        let colors_config = config::SamplePointConfig {
                            display_id,
                            points,
                        };

                        colors_configs.push(colors_config);
                    }
                }
                let colors = screenshot_manager
                    .get_all_colors(&colors_configs, &configs.mappers, &channels)
                    .await;
                match tx.send(colors) {
                    Ok(_) => {
                        // log::info!("colors updated");
                    }
                    Err(_) => {
                        warn!("colors update failed");
                    }
                }
            }
        });
        Ok(())
    }

    pub async fn send_colors(payload: Vec<u8>) -> anyhow::Result<()> {
        let mqtt = MqttRpc::global().await;

        mqtt.publish_led_sub_pixels(payload).await
    }

    pub async fn clone_receiver(&self) -> watch::Receiver<Vec<u8>> {
        self.rx.read().await.clone()
    }
}
