use std::{collections::HashMap, sync::Arc, time::Duration};

use paris::warn;
use tauri::async_runtime::RwLock;
use tokio::{
    sync::{broadcast, watch},
    time::sleep,
};

use crate::{
    ambient_light::{config, ConfigManager},
    led_color::LedColor,
    rpc::UdpRpc,
    screenshot::LedSamplePoints,
    screenshot_manager::ScreenshotManager,
};

use itertools::Itertools;

use super::{ColorCalibration, LedStripConfig, LedStripConfigGroup, LedType, SamplePointMapper};

pub struct LedColorsPublisher {
    sorted_colors_rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    sorted_colors_tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
    colors_rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    colors_tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
    inner_tasks_version: Arc<RwLock<usize>>,
    test_mode_active: Arc<RwLock<bool>>,
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
                    inner_tasks_version: Arc::new(RwLock::new(0)),
                    test_mode_active: Arc::new(RwLock::new(false)),
                }
            })
            .await
    }

    async fn start_one_display_colors_fetcher(
        &self,
        display_id: u32,
        sample_points: Vec<LedSamplePoints>,
        _bound_scale_factor: f32,
        mappers: Vec<SamplePointMapper>,
        display_colors_tx: broadcast::Sender<(u32, Vec<u8>)>,
        strips: Vec<LedStripConfig>,
        color_calibration: ColorCalibration,
    ) {
        let internal_tasks_version = self.inner_tasks_version.clone();
        let screenshot_manager = ScreenshotManager::global().await;

        let screenshot_rx = screenshot_manager.subscribe_by_display_id(display_id).await;

        if let Err(err) = screenshot_rx {
            log::error!("{}", err);
            return;
        }
        let mut screenshot_rx = screenshot_rx.unwrap();

        tokio::spawn(async move {
            let init_version = internal_tasks_version.read().await.clone();

            while screenshot_rx.changed().await.is_ok() {
                let screenshot = screenshot_rx.borrow().clone();
                let colors = screenshot.get_colors_by_sample_points(&sample_points).await;

                let colors_copy = colors.clone();

                let mappers = mappers.clone();

                // Check if test mode is active and ambient light is enabled before sending normal colors
                let test_mode_active = {
                    let publisher = LedColorsPublisher::global().await;
                    *publisher.test_mode_active.read().await
                };

                let ambient_light_enabled = {
                    let state_manager =
                        crate::ambient_light_state::AmbientLightStateManager::global().await;
                    state_manager.is_enabled().await
                };

                log::debug!(
                    "Display #{}: test_mode_active={}, ambient_light_enabled={}, colors_count={}",
                    display_id,
                    test_mode_active,
                    ambient_light_enabled,
                    colors.len()
                );

                if !test_mode_active && ambient_light_enabled {
                    match Self::send_colors_by_display(colors, mappers, &strips, &color_calibration)
                        .await
                    {
                        Ok(_) => {
                            log::info!("Successfully sent colors for display #{}", display_id);
                        }
                        Err(err) => {
                            warn!("Failed to send colors:  #{: >15}\t{}", display_id, err);
                        }
                    }
                } else {
                    log::debug!(
                        "Skipping color send for display #{}: test_mode={}, enabled={}",
                        display_id,
                        test_mode_active,
                        ambient_light_enabled
                    );
                }

                match display_colors_tx.send((
                    display_id,
                    colors_copy
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

                // Check if the inner task version changed
                let version = internal_tasks_version.read().await.clone();
                if version != init_version {
                    log::info!(
                        "inner task version changed, stop.  {} != {}",
                        internal_tasks_version.read().await.clone(),
                        init_version
                    );

                    break;
                }
            }
        });
    }

    fn start_all_colors_worker(
        &self,
        display_ids: Vec<u32>,
        mappers: Vec<SamplePointMapper>,
        mut display_colors_rx: broadcast::Receiver<(u32, Vec<u8>)>,
    ) {
        let sorted_colors_tx = self.sorted_colors_tx.clone();
        let colors_tx = self.colors_tx.clone();

        tokio::spawn(async move {
            let sorted_colors_tx = sorted_colors_tx.write().await;
            let colors_tx = colors_tx.write().await;

            let mut all_colors: Vec<Option<Vec<u8>>> = vec![None; display_ids.len()];
            let mut _start: tokio::time::Instant = tokio::time::Instant::now();

            loop {
                let color_info = display_colors_rx.recv().await;

                if let Err(err) = color_info {
                    match err {
                        broadcast::error::RecvError::Closed => {
                            return;
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

                    _start = tokio::time::Instant::now();
                }
            }
        });
    }

    pub async fn start(&self) {
        let config_manager = ConfigManager::global().await;
        let mut config_receiver = config_manager.clone_config_update_receiver();
        let configs = config_receiver.borrow().clone();

        self.handle_config_change(configs).await;

        while config_receiver.changed().await.is_ok() {
            let configs = config_receiver.borrow().clone();
            self.handle_config_change(configs).await;
        }
    }

    async fn handle_config_change(&self, original_configs: LedStripConfigGroup) {
        let inner_tasks_version = self.inner_tasks_version.clone();
        let configs = Self::get_colors_configs(&original_configs).await;

        if let Err(err) = configs {
            warn!("Failed to get configs: {}", err);
            sleep(Duration::from_millis(100)).await;
            return;
        }

        let configs = configs.unwrap();

        let mut inner_tasks_version = inner_tasks_version.write().await;
        *inner_tasks_version = inner_tasks_version.overflowing_add(1).0;
        drop(inner_tasks_version);

        let (display_colors_tx, display_colors_rx) = broadcast::channel::<(u32, Vec<u8>)>(8);

        for sample_point_group in configs.sample_point_groups.clone() {
            let display_id = sample_point_group.display_id;
            let sample_points = sample_point_group.points;
            let bound_scale_factor = sample_point_group.bound_scale_factor;

            // Get strips for this display
            let display_strips: Vec<LedStripConfig> = original_configs
                .strips
                .iter()
                .filter(|strip| strip.display_id == display_id)
                .cloned()
                .collect();

            self.start_one_display_colors_fetcher(
                display_id,
                sample_points,
                bound_scale_factor,
                sample_point_group.mappers,
                display_colors_tx.clone(),
                display_strips,
                original_configs.color_calibration,
            )
            .await;
        }

        let display_ids = configs.sample_point_groups;
        self.start_all_colors_worker(
            display_ids.iter().map(|c| c.display_id).collect(),
            configs.mappers,
            display_colors_rx,
        );
    }

    pub async fn send_colors(offset: u16, mut payload: Vec<u8>) -> anyhow::Result<()> {
        // Use UdpRpc to send to all discovered devices instead of hardcoded IP
        let udp_rpc = UdpRpc::global().await;
        if let Err(err) = udp_rpc {
            warn!("udp_rpc can not be initialized: {}", err);
            return Err(anyhow::anyhow!("UDP RPC not available: {}", err));
        }
        let udp_rpc = udp_rpc.as_ref().unwrap();

        let mut buffer = vec![2];
        buffer.push((offset >> 8) as u8);
        buffer.push((offset & 0xff) as u8);
        buffer.append(&mut payload);

        udp_rpc.send_to_all(&buffer).await?;
        Ok(())
    }

    pub async fn send_colors_by_display(
        colors: Vec<LedColor>,
        mappers: Vec<SamplePointMapper>,
        strips: &[LedStripConfig],
        color_calibration: &ColorCalibration,
    ) -> anyhow::Result<()> {
        // let color_len = colors.len();
        let display_led_offset = mappers
            .clone()
            .iter()
            .flat_map(|mapper| [mapper.start, mapper.end])
            .min()
            .unwrap();

        let udp_rpc = UdpRpc::global().await;
        if let Err(err) = udp_rpc {
            warn!("udp_rpc can not be initialized: {}", err);
            return Err(anyhow::anyhow!("UDP RPC not available: {}", err));
        }
        let udp_rpc = udp_rpc.as_ref().unwrap();

        // let socket = UdpSocket::bind("0.0.0.0:0").await?;
        for (group_index, group) in mappers.clone().iter().enumerate() {
            if (group.start.abs_diff(group.end)) > colors.len() {
                return Err(anyhow::anyhow!(
                    "get_sorted_colors: color_index out of range. color_index: {}, strip len: {}, colors.len(): {}",
                    group.pos,
                    group.start.abs_diff(group.end),
                    colors.len()
                ));
            }

            let group_size = group.start.abs_diff(group.end);

            // Find the corresponding LED strip config to get LED type
            let led_type = if group_index < strips.len() {
                strips[group_index].led_type
            } else {
                LedType::WS2812B // fallback to WS2812B
            };

            let bytes_per_led = match led_type {
                LedType::WS2812B => 3,
                LedType::SK6812 => 4,
            };

            let mut buffer = Vec::<u8>::with_capacity(group_size * bytes_per_led);

            if group.end > group.start {
                // Prevent integer underflow by using saturating subtraction
                let start_index = if group.pos >= display_led_offset {
                    group.pos - display_led_offset
                } else {
                    0
                };
                let end_index = if group.pos + group_size >= display_led_offset {
                    group_size + group.pos - display_led_offset
                } else {
                    0
                };

                for i in start_index..end_index {
                    if i < colors.len() {
                        let bytes = match led_type {
                            LedType::WS2812B => {
                                let calibration_bytes = color_calibration.to_bytes();
                                let color_bytes = colors[i].as_bytes();
                                // Apply calibration and convert RGB to GRB for WS2812B
                                vec![
                                    ((color_bytes[1] as f32 * calibration_bytes[1] as f32 / 255.0)
                                        as u8), // G (Green)
                                    ((color_bytes[0] as f32 * calibration_bytes[0] as f32 / 255.0)
                                        as u8), // R (Red)
                                    ((color_bytes[2] as f32 * calibration_bytes[2] as f32 / 255.0)
                                        as u8), // B (Blue)
                                ]
                            }
                            LedType::SK6812 => {
                                let calibration_bytes = color_calibration.to_bytes_rgbw();
                                let color_bytes = colors[i].as_bytes();
                                // Apply calibration to RGB values and use calibrated W
                                vec![
                                    ((color_bytes[0] as f32 * calibration_bytes[0] as f32 / 255.0)
                                        as u8),
                                    ((color_bytes[1] as f32 * calibration_bytes[1] as f32 / 255.0)
                                        as u8),
                                    ((color_bytes[2] as f32 * calibration_bytes[2] as f32 / 255.0)
                                        as u8),
                                    calibration_bytes[3], // W channel
                                ]
                            }
                        };
                        buffer.extend_from_slice(&bytes);
                    } else {
                        log::warn!(
                            "Index {} out of bounds for colors array of length {}",
                            i,
                            colors.len()
                        );
                        // Add black color as fallback
                        match led_type {
                            LedType::WS2812B => buffer.extend_from_slice(&[0, 0, 0]),
                            LedType::SK6812 => buffer.extend_from_slice(&[0, 0, 0, 0]),
                        }
                    }
                }
            } else {
                // Prevent integer underflow by using saturating subtraction
                let start_index = if group.pos >= display_led_offset {
                    group.pos - display_led_offset
                } else {
                    0
                };
                let end_index = if group.pos + group_size >= display_led_offset {
                    group_size + group.pos - display_led_offset
                } else {
                    0
                };

                for i in (start_index..end_index).rev() {
                    if i < colors.len() {
                        let bytes = match led_type {
                            LedType::WS2812B => {
                                let calibration_bytes = color_calibration.to_bytes();
                                let color_bytes = colors[i].as_bytes();
                                // Apply calibration and convert RGB to GRB for WS2812B
                                vec![
                                    ((color_bytes[1] as f32 * calibration_bytes[1] as f32 / 255.0)
                                        as u8), // G (Green)
                                    ((color_bytes[0] as f32 * calibration_bytes[0] as f32 / 255.0)
                                        as u8), // R (Red)
                                    ((color_bytes[2] as f32 * calibration_bytes[2] as f32 / 255.0)
                                        as u8), // B (Blue)
                                ]
                            }
                            LedType::SK6812 => {
                                let calibration_bytes = color_calibration.to_bytes_rgbw();
                                let color_bytes = colors[i].as_bytes();
                                // Apply calibration to RGB values and use calibrated W
                                vec![
                                    ((color_bytes[0] as f32 * calibration_bytes[0] as f32 / 255.0)
                                        as u8),
                                    ((color_bytes[1] as f32 * calibration_bytes[1] as f32 / 255.0)
                                        as u8),
                                    ((color_bytes[2] as f32 * calibration_bytes[2] as f32 / 255.0)
                                        as u8),
                                    calibration_bytes[3], // W channel
                                ]
                            }
                        };
                        buffer.extend_from_slice(&bytes);
                    } else {
                        log::warn!(
                            "Index {} out of bounds for colors array of length {}",
                            i,
                            colors.len()
                        );
                        // Add black color as fallback
                        match led_type {
                            LedType::WS2812B => buffer.extend_from_slice(&[0, 0, 0]),
                            LedType::SK6812 => buffer.extend_from_slice(&[0, 0, 0, 0]),
                        }
                    }
                }
            }

            // Calculate byte offset based on LED position and LED type
            let led_offset = group.start.min(group.end);
            let byte_offset = led_offset * bytes_per_led;
            let mut tx_buffer = vec![2];
            tx_buffer.push((byte_offset >> 8) as u8);
            tx_buffer.push((byte_offset & 0xff) as u8);
            tx_buffer.append(&mut buffer);

            udp_rpc.send_to_all(&tx_buffer).await?;
        }

        Ok(())
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
        merged_screenshot_receiver.resubscribe();

        let mut screenshots = HashMap::new();

        loop {
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
                let mut led_start = 0;

                for display_id in display_ids {
                    let led_strip_configs = configs
                        .strips
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.display_id == display_id);

                    let screenshot = screenshots.get(&display_id).unwrap();

                    let points: Vec<_> = led_strip_configs
                        .clone()
                        .map(|(_, config)| screenshot.get_sample_points(&config))
                        .flatten()
                        .collect();

                    if points.len() == 0 {
                        warn!("no led strip config for display_id: {}", display_id);
                        continue;
                    }

                    let bound_scale_factor = screenshot.bound_scale_factor;

                    let led_end = led_start + points.iter().map(|p| p.len()).sum::<usize>();

                    let mappers = led_strip_configs.map(|(i, _)| mappers[i].clone()).collect();

                    let colors_config = DisplaySamplePointGroup {
                        display_id,
                        points,
                        bound_scale_factor,
                        mappers,
                    };

                    colors_configs.push(colors_config);
                    led_start = led_end;
                }

                return Ok(AllColorConfig {
                    sample_point_groups: colors_configs,
                    mappers,
                });
            }
        }
    }

    pub async fn clone_colors_receiver(&self) -> watch::Receiver<Vec<u8>> {
        self.colors_rx.read().await.clone()
    }

    /// Enable test mode - this will pause normal LED data publishing
    pub async fn enable_test_mode(&self) {
        let mut test_mode = self.test_mode_active.write().await;
        *test_mode = true;
        log::info!("Test mode enabled - normal LED publishing paused");
    }

    /// Disable test mode - this will resume normal LED data publishing
    pub async fn disable_test_mode(&self) {
        let mut test_mode = self.test_mode_active.write().await;
        *test_mode = false;
        log::info!("Test mode disabled - normal LED publishing resumed");
    }

    /// Disable test mode with a delay to ensure clean transition
    pub async fn disable_test_mode_with_delay(&self, delay_ms: u64) {
        // Wait for the specified delay
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        let mut test_mode = self.test_mode_active.write().await;
        *test_mode = false;
        log::info!("Test mode disabled with delay - normal LED publishing resumed");
    }

    /// Check if test mode is currently active
    pub async fn is_test_mode_active(&self) -> bool {
        *self.test_mode_active.read().await
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
    pub points: Vec<LedSamplePoints>,
    pub bound_scale_factor: f32,
    pub mappers: Vec<config::SamplePointMapper>,
}
