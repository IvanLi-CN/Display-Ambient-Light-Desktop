use std::{collections::HashMap, sync::Arc, time::Duration};

use paris::warn;
use tauri::async_runtime::RwLock;
use tokio::{
    sync::{broadcast, watch},
    time::sleep,
};

use crate::{
    ambient_light::{config, ConfigManager},
    rpc::MqttRpc,
    screenshot::LedSamplePoints,
    screenshot_manager::{self, ScreenshotManager},
};

use itertools::Itertools;

use super::{LedStripConfigGroup, SamplePointConfig, SamplePointMapper};

pub struct LedColorsPublisher {
    sorted_colors_rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    sorted_colors_tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
    colors_rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    colors_tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
    display_colors_rx: Arc<RwLock<broadcast::Receiver<(u32, Vec<u8>)>>>,
    display_colors_tx: Arc<RwLock<broadcast::Sender<(u32, Vec<u8>)>>>,
    inner_tasks_version: Arc<RwLock<usize>>,
}

impl LedColorsPublisher {
    pub async fn global() -> &'static Self {
        static LED_COLORS_PUBLISHER_GLOBAL: tokio::sync::OnceCell<LedColorsPublisher> =
            tokio::sync::OnceCell::const_new();

        let (sorted_tx, sorted_rx) = watch::channel(Vec::new());
        let (tx, rx) = watch::channel(Vec::new());
        let (display_colors_tx, display_colors_rx) = broadcast::channel(8);

        LED_COLORS_PUBLISHER_GLOBAL
            .get_or_init(|| async {
                LedColorsPublisher {
                    sorted_colors_rx: Arc::new(RwLock::new(sorted_rx)),
                    sorted_colors_tx: Arc::new(RwLock::new(sorted_tx)),
                    colors_rx: Arc::new(RwLock::new(rx)),
                    colors_tx: Arc::new(RwLock::new(tx)),
                    display_colors_rx: Arc::new(RwLock::new(display_colors_rx)),
                    display_colors_tx: Arc::new(RwLock::new(display_colors_tx)),
                    inner_tasks_version: Arc::new(RwLock::new(0)),
                }
            })
            .await
    }

    fn start_one_display_colors_fetcher(
        &self,
        display_id: u32,
        sample_points: Vec<Vec<LedSamplePoints>>,
    ) {
        let display_colors_tx = self.display_colors_tx.clone();
        let internal_tasks_version = self.inner_tasks_version.clone();

        tokio::spawn(async move {
            let display_colors_tx = display_colors_tx.read().await.clone();

            let colors = screenshot_manager::get_display_colors(display_id, &sample_points);

            if let Err(err) = colors {
                warn!("Failed to get colors: {}", err);
                return;
            }

            let mut start: tokio::time::Instant = tokio::time::Instant::now();
            let mut interval = tokio::time::interval(Duration::from_millis(66));
            let init_version = internal_tasks_version.read().await.clone();

            loop {
                interval.tick().await;
                tokio::time::sleep(Duration::from_millis(1)).await;

                if internal_tasks_version.read().await.clone() != init_version {
                    log::info!(
                        "inner task version changed, stop.  {} != {}",
                        internal_tasks_version.read().await.clone(),
                        init_version
                    );

                    break;
                }

                // log::info!("tick: {}ms", start.elapsed().as_millis());
                start = tokio::time::Instant::now();
                let colors = screenshot_manager::get_display_colors(display_id, &sample_points);

                if let Err(err) = colors {
                    warn!("Failed to get colors: {}", err);
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }

                let colors = colors.unwrap();

                let color_len = colors.len();

                match display_colors_tx.send((
                    display_id,
                    colors
                        .into_iter()
                        .map(|color| color.get_rgb())
                        .flatten()
                        .collect::<Vec<_>>(),
                )) {
                    Ok(_) => {
                        // log::info!("sent colors: {:?}", color_len);
                    }
                    Err(err) => {
                        warn!("Failed to send display_colors: {}", err);
                    }
                };
            }
        });
    }

    fn start_all_colors_worker(&self, display_ids: Vec<u32>, mappers: Vec<SamplePointMapper>) {
        let sorted_colors_tx = self.sorted_colors_tx.clone();
        let colors_tx = self.colors_tx.clone();
        let display_colors_rx = self.display_colors_rx.clone();

        tokio::spawn(async move {
            for _ in 0..10 {
                let mut rx = display_colors_rx.read().await.resubscribe();

                let sorted_colors_tx = sorted_colors_tx.write().await;
                let colors_tx = colors_tx.write().await;

                let mut all_colors: Vec<Option<Vec<u8>>> = vec![None; display_ids.len()];
                let mut start: tokio::time::Instant = tokio::time::Instant::now();

                log::info!("start all_colors_worker");
                loop {
                    // log::info!("display_colors_rx changed");
                    let color_info = rx.recv().await;

                    if let Err(err) = color_info {
                        match err {
                            broadcast::error::RecvError::Closed => {
                                break;
                            }
                            broadcast::error::RecvError::Lagged(_) => {
                                warn!("display_colors_rx lagged");
                                continue;
                            }
                        }
                    }
                    let (display_id, colors) = color_info.unwrap();

                    let index = display_ids.iter().position(|id| *id == display_id);

                    if index.is_none() {
                        warn!("display id not found");
                        continue;
                    }

                    all_colors[index.unwrap()] = Some(colors);

                    if all_colors.iter().all(|color| color.is_some()) {
                        let flatten_colors = all_colors
                            .clone()
                            .into_iter()
                            .flat_map(|c| c.unwrap())
                            .collect::<Vec<_>>();

                        match colors_tx.send(flatten_colors.clone()) {
                            Ok(_) => {}
                            Err(err) => {
                                warn!("Failed to send colors: {}", err);
                            }
                        };

                        let sorted_colors =
                            ScreenshotManager::get_sorted_colors(&flatten_colors, &mappers);

                        match sorted_colors_tx.send(sorted_colors) {
                            Ok(_) => {}
                            Err(err) => {
                                warn!("Failed to send sorted colors: {}", err);
                            }
                        };
                        log::info!("tick: {}ms", start.elapsed().as_millis());
                        start = tokio::time::Instant::now();
                    }
                }
            }
        });
    }

    pub fn start(&self) {
        let inner_tasks_version = self.inner_tasks_version.clone();

        tokio::spawn(async move {
            let publisher = Self::global().await;

            let mut inner_tasks_version = inner_tasks_version.write().await;
            *inner_tasks_version = inner_tasks_version.overflowing_add(1).0;

            let config_manager = ConfigManager::global().await;
            let mut config_receiver = config_manager.clone_config_update_receiver();

            log::info!("waiting for config update...");

            while config_receiver.changed().await.is_ok() {
                log::info!("config updated, restart inner tasks...");
                let configs = config_receiver.borrow().clone();
                let configs = Self::get_colors_configs(&configs).await;

                if let Err(err) = configs {
                    warn!("Failed to get configs: {}", err);
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }

                let configs = configs.unwrap();

                for sample_point_group in configs.sample_point_groups.clone() {
                    let display_id = sample_point_group.display_id;
                    let sample_points = sample_point_group.points;

                    publisher.start_one_display_colors_fetcher(display_id, sample_points);
                }

                let display_ids = configs.sample_point_groups;
                publisher.start_all_colors_worker(
                    display_ids.iter().map(|c| c.display_id).collect(),
                    configs.mappers,
                );

                break;
            }
        });

        // tokio::spawn(async move {
        //     loop {
        //         let sorted_colors_tx = sorted_colors_tx.write().await;
        //         let colors_tx = colors_tx.write().await;
        //         let screenshot_manager = ScreenshotManager::global().await;

        //         let config_manager = ConfigManager::global().await;
        //         let config_receiver = config_manager.clone_config_update_receiver();
        //         let configs = config_receiver.borrow().clone();
        //         let configs = Self::get_colors_configs(&configs).await;

        //         if let Err(err) = configs {
        //             warn!("Failed to get configs: {}", err);
        //             sleep(Duration::from_millis(100)).await;
        //             continue;
        //         }

        //         let configs = configs.unwrap();

        //         let mut merged_screenshot_receiver =
        //             screenshot_manager.clone_merged_screenshot_rx().await;

        //         let mut screenshots = HashMap::new();

        //         // let mut start = tokio::time::Instant::now();

        //         loop {
        //             let screenshot = merged_screenshot_receiver.recv().await;

        //             if let Err(err) = screenshot {
        //                 match err {
        //                     tokio::sync::broadcast::error::RecvError::Closed => {
        //                         warn!("closed");
        //                         continue;
        //                     }
        //                     tokio::sync::broadcast::error::RecvError::Lagged(_) => {
        //                         warn!("lagged");
        //                         continue;
        //                     }
        //                 }
        //             }

        //             let screenshot = screenshot.unwrap();
        //             // log::info!("got screenshot: {:?}", screenshot.display_id);

        //             screenshots.insert(screenshot.display_id, screenshot);

        //             if screenshots.len() == configs.sample_point_groups.len() {
        //                 // log::info!("{}", start.elapsed().as_millis().to_string());
        //                 {
        //                     let screenshots = configs
        //                         .sample_point_groups
        //                         .iter()
        //                         .map(|strip| screenshots.get(&strip.display_id).unwrap())
        //                         .collect::<Vec<_>>();

        //                     let colors = screenshot_manager
        //                         .get_all_colors(&configs.sample_point_groups, &screenshots)
        //                         .await;

        //                     let sorted_colors =
        //                         ScreenshotManager::get_sorted_colors(&colors, &configs.mappers)
        //                             .await;

        //                     match colors_tx.send(colors) {
        //                         Ok(_) => {
        //                             // log::info!("colors updated");
        //                         }
        //                         Err(_) => {
        //                             warn!("colors update failed");
        //                         }
        //                     }

        //                     match sorted_colors_tx.send(sorted_colors) {
        //                         Ok(_) => {
        //                             // log::info!("colors updated");
        //                         }
        //                         Err(_) => {
        //                             warn!("colors update failed");
        //                         }
        //                     }
        //                 }

        //                 // screenshots.clear();
        //                 // start = tokio::time::Instant::now();
        //             }
        //         }
        //     }
        // });

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
                        // log::info!("colors sent. len: {}", len);
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

        let display_ids = configs
            .strips
            .iter()
            .map(|c| c.display_id)
            .unique()
            .collect::<Vec<_>>();

        let mappers = configs.mappers.clone();

        let mut colors_configs = Vec::new();

        let mut merged_screenshot_receiver = screenshot_manager.clone_merged_screenshot_rx().await;

        let mut screenshots = HashMap::new();

        loop {
            log::info!("waiting merged screenshot...");
            let screenshot = merged_screenshot_receiver.recv().await;

            if let Err(err) = screenshot {
                match err {
                    tokio::sync::broadcast::error::RecvError::Closed => {
                        warn!("closed");
                        continue;
                    }
                    tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                        warn!("lagged");
                        continue;
                    }
                }
            }

            let screenshot = screenshot.unwrap();
            // log::info!("got screenshot: {:?}", screenshot.display_id);

            screenshots.insert(screenshot.display_id, screenshot);

            if screenshots.len() == display_ids.len() {
                for display_id in display_ids {
                    let led_strip_configs: Vec<_> = configs
                        .strips
                        .iter()
                        .filter(|c| c.display_id == display_id)
                        .collect();

                    if led_strip_configs.len() == 0 {
                        warn!("no led strip config for display_id: {}", display_id);
                        continue;
                    }

                    let screenshot = screenshots.get(&display_id).unwrap();
                    log::debug!("screenshot updated: {:?}", display_id);

                    let points: Vec<_> = led_strip_configs
                        .iter()
                        .map(|config| screenshot.get_sample_points(&config))
                        .collect();

                    let colors_config = DisplaySamplePointGroup { display_id, points };

                    colors_configs.push(colors_config);
                }

                return Ok(AllColorConfig {
                    sample_point_groups: colors_configs,
                    mappers,
                    // screenshot_receivers: local_rx_list,
                });
            }
        }
    }

    pub async fn clone_colors_receiver(&self) -> watch::Receiver<Vec<u8>> {
        self.colors_rx.read().await.clone()
    }
}

#[derive(Debug)]
pub struct AllColorConfig {
    pub sample_point_groups: Vec<DisplaySamplePointGroup>,
    pub mappers: Vec<config::SamplePointMapper>,
    // pub screenshot_receivers: Vec<watch::Receiver<Screenshot>>,
}

#[derive(Debug, Clone)]
pub struct DisplaySamplePointGroup {
    pub display_id: u32,
    pub points: Vec<Vec<LedSamplePoints>>,
}
