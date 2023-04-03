use std::{collections::HashMap, sync::Arc, time::Duration};

use paris::{info, warn};
use tauri::async_runtime::{Mutex, RwLock};
use tokio::{sync::watch, time::sleep};

use crate::{
    ambient_light::{config, ConfigManager},
    led_color::LedColor,
    rpc::MqttRpc,
    screenshot::{self, Screenshot},
    screenshot_manager::ScreenshotManager,
};

use itertools::Itertools;

use super::{LedStripConfigGroup, SamplePointConfig};

pub struct LedColorsPublisher {
    sorted_colors_rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    sorted_colors_tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
    colors_rx: Arc<RwLock<watch::Receiver<Vec<LedColor>>>>,
    colors_tx: Arc<RwLock<watch::Sender<Vec<LedColor>>>>,
}

impl LedColorsPublisher {
    pub async fn global() -> &'static Self {
        static LED_COLORS_PUBLISHER_GLOBAL: tokio::sync::OnceCell<LedColorsPublisher> =
            tokio::sync::OnceCell::const_new();

        let (sorted_tx, sorted_rx) = watch::channel(Vec::new());
        let (tx, rx) = watch::channel(Vec::new());

        LED_COLORS_PUBLISHER_GLOBAL
            .get_or_init(|| async {
                LedColorsPublisher {
                    sorted_colors_rx: Arc::new(RwLock::new(sorted_rx)),
                    sorted_colors_tx: Arc::new(RwLock::new(sorted_tx)),
                    colors_rx: Arc::new(RwLock::new(rx)),
                    colors_tx: Arc::new(RwLock::new(tx)),
                }
            })
            .await
    }

    pub fn start(&self) {
        let sorted_colors_tx = self.sorted_colors_tx.clone();
        let colors_tx = self.colors_tx.clone();

        tokio::spawn(async move {
            loop {
                let sorted_colors_tx = sorted_colors_tx.write().await;
                let colors_tx = colors_tx.write().await;
                let screenshot_manager = ScreenshotManager::global().await;

                let config_manager = ConfigManager::global().await;
                let config_receiver = config_manager.clone_config_update_receiver();
                let configs = config_receiver.borrow().clone();
                let configs = Self::get_colors_configs(&configs).await;

                if let Err(err) = configs {
                    warn!("Failed to get configs: {}", err);
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }

                let configs = configs.unwrap();

                let mut merged_screenshot_receiver =
                    screenshot_manager.clone_merged_screenshot_rx().await;

                let mut screenshots = HashMap::new();

                loop {
                    let screenshot = merged_screenshot_receiver.recv().await;

                    if let Err(err) = screenshot {
                        match err {
                            tokio::sync::broadcast::error::RecvError::Closed => {
                                warn!("closed");
                                continue;
                            },
                            tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                                warn!("lagged");
                                continue;
                            },
                        }
                    }

                    let screenshot = screenshot.unwrap();
                    // log::info!("got screenshot: {:?}", screenshot.display_id);

                    screenshots.insert(screenshot.display_id, screenshot);

                    if screenshots.len() == configs.sample_point_groups.len() {
                        {
                            let screenshots = configs
                                .sample_point_groups
                                .iter()
                                .map(|strip| screenshots.get(&strip.display_id).unwrap())
                                .collect::<Vec<_>>();

                            let colors = screenshot_manager
                                .get_all_colors(&configs.sample_point_groups, &screenshots)
                                .await;

                            let sorted_colors =
                                ScreenshotManager::get_sorted_colors(&colors, &configs.mappers)
                                    .await;

                            match colors_tx.send(colors) {
                                Ok(_) => {
                                    // log::info!("colors updated");
                                }
                                Err(_) => {
                                    warn!("colors update failed");
                                }
                            }

                            match sorted_colors_tx.send(sorted_colors) {
                                Ok(_) => {
                                    // log::info!("colors updated");
                                }
                                Err(_) => {
                                    warn!("colors update failed");
                                }
                            }
                        }

                        screenshots.clear();
                    }
                }
            }
        });

        let rx = self.sorted_colors_rx.clone();
        tokio::spawn(async move {
            let mut rx = rx.read().await.clone();
            loop {
                if let Err(err) = rx.changed().await {
                    warn!("rx changed error: {}", err);
                    sleep(Duration::from_millis(1000)).await;
                    continue;
                }

                let colors = rx.borrow().clone();

                let len = colors.len();

                match Self::send_colors(colors).await {
                    Ok(_) => {
                        log::info!("colors sent. len: {}", len);
                    }
                    Err(err) => {
                        warn!("colors send failed: {}", err);
                    }
                }
            }
        });
    }

    pub async fn send_colors(payload: Vec<u8>) -> anyhow::Result<()> {
        let mqtt = MqttRpc::global().await;

        mqtt.publish_led_sub_pixels(payload).await
    }

    pub async fn clone_sorted_colors_receiver(&self) -> watch::Receiver<Vec<u8>> {
        self.sorted_colors_rx.read().await.clone()
    }
    pub async fn get_colors_configs(
        configs: &LedStripConfigGroup,
    ) -> anyhow::Result<AllColorConfig> {
        let screenshot_manager = ScreenshotManager::global().await;

        let channels = screenshot_manager.channels.read().await;

        let display_ids = configs
            .strips
            .iter()
            .map(|c| c.display_id)
            .unique()
            .collect::<Vec<_>>();

        let mappers = configs.mappers.clone();

        let mut local_rx_list = Vec::new();
        let mut colors_configs = Vec::new();

        for display_id in display_ids.clone().iter() {
            let display_id = *display_id;

            let channel = channels.get(&display_id);
            if channel.is_none() {
                anyhow::bail!("no channel for display_id: {}", display_id);
            }

            let channel_rx = channel.unwrap().clone();

            local_rx_list.push(channel.unwrap().clone());

            let led_strip_configs: Vec<_> = configs
                .strips
                .iter()
                .filter(|c| c.display_id == display_id)
                .collect();

            if led_strip_configs.len() == 0 {
                warn!("no led strip config for display_id: {}", display_id);
                continue;
            }
            let rx = channel_rx.to_owned();

            let screenshot = rx.borrow().clone();
            log::debug!("screenshot updated: {:?}", display_id);

            let points: Vec<_> = led_strip_configs
                .iter()
                .map(|config| screenshot.get_sample_points(&config))
                .flatten()
                .collect();

            let colors_config = config::SamplePointConfig { display_id, points };

            colors_configs.push(colors_config);
        }

        return Ok(AllColorConfig {
            sample_point_groups: colors_configs,
            mappers,
            screenshot_receivers: local_rx_list,
        });
    }

    pub async fn clone_colors_receiver(&self) -> watch::Receiver<Vec<LedColor>> {
        self.colors_rx.read().await.clone()
    }
}

#[derive(Debug)]
pub struct AllColorConfig {
    pub sample_point_groups: Vec<SamplePointConfig>,
    pub mappers: Vec<config::SamplePointMapper>,
    pub screenshot_receivers: Vec<watch::Receiver<Screenshot>>,
}
