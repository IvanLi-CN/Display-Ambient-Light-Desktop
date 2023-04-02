use std::{sync::Arc, time::Duration};

use paris::warn;
use tauri::async_runtime::{Mutex, RwLock};
use tokio::{sync::watch, time::sleep};

use crate::{
    ambient_light::{config, ConfigManager},
    rpc::MqttRpc,
    screenshot::Screenshot,
    screenshot_manager::ScreenshotManager,
};

use itertools::Itertools;

use super::{LedStripConfigGroup, SamplePointConfig};

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

    pub fn start(&self) {
        let tx = self.tx.clone();

        tokio::spawn(async move {
            loop {
                log::info!("colors update loop AAA");

                let tx = tx.write().await;
                let screenshot_manager = ScreenshotManager::global().await;

                let config_manager = ConfigManager::global().await;
                let config_receiver = config_manager.clone_config_update_receiver();
                let configs = config_receiver.borrow().clone();
                let configs = Self::get_colors_configs(&configs).await;

                let mut some_screenshot_receiver_is_none = false;

                loop {
                    let mut screenshots = Vec::new();

                    for rx in configs.screenshot_receivers.to_owned() {
                        let mut rx = rx.lock_owned().await;
                        if rx.is_none() {
                            some_screenshot_receiver_is_none = true;
                            warn!("screenshot receiver is none");
                            continue;
                        }

                        let rx = rx.as_mut().unwrap();

                        if let Err(err) = rx.changed().await {
                            warn!("rx changed error: {}", err);
                            continue;
                        }
                        // log::info!("screenshot updated");

                        let screenshot = rx.borrow().clone();

                        screenshots.push(screenshot);
                    }

                    let colors = screenshot_manager
                        .get_all_colors(
                            &configs.sample_point_groups,
                            &configs.mappers,
                            &screenshots,
                        )
                        .await;
                    match tx.send(colors) {
                        Ok(_) => {
                            // log::info!("colors updated");
                        }
                        Err(_) => {
                            warn!("colors update failed");
                        }
                    }

                    if some_screenshot_receiver_is_none
                        || config_receiver.has_changed().unwrap_or(true)
                    {
                        sleep(Duration::from_millis(1000)).await;
                        break;
                    }
                }
            }
        });

        let rx = self.rx.clone();
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

    pub async fn clone_receiver(&self) -> watch::Receiver<Vec<u8>> {
        self.rx.read().await.clone()
    }

    pub async fn get_colors_configs(configs: &LedStripConfigGroup) -> AllColorConfig {
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
            let channel = match channel {
                Some(channel) => Some(channel.clone()),
                None => None,
            };
            local_rx_list.push(Arc::new(Mutex::new(channel.clone())));

            let led_strip_configs: Vec<_> = configs
                .strips
                .iter()
                .filter(|c| c.display_id == display_id)
                .collect();

            let rx = channel;
            if rx.is_none() {
                warn!("no channel for display_id: {}", display_id);
                continue;
            }

            if led_strip_configs.len() == 0 {
                warn!("no led strip config for display_id: {}", display_id);
                continue;
            }
            let mut rx = rx.unwrap().to_owned();

            if rx.changed().await.is_ok() {
                let screenshot = rx.borrow().clone();
                log::info!("screenshot updated: {:?}", display_id);

                let points: Vec<_> = led_strip_configs
                    .iter()
                    .map(|config| screenshot.get_sample_points(&config))
                    .flatten()
                    .collect();

                let colors_config = config::SamplePointConfig { display_id, points };

                colors_configs.push(colors_config);
            }
        }

        return AllColorConfig {
            sample_point_groups: colors_configs,
            mappers,
            screenshot_receivers: local_rx_list,
        };
    }
}

pub struct AllColorConfig {
    pub sample_point_groups: Vec<SamplePointConfig>,
    pub mappers: Vec<config::SamplePointMapper>,
    pub screenshot_receivers: Vec<Arc<Mutex<Option<watch::Receiver<Screenshot>>>>>,
}
