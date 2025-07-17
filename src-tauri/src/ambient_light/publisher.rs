use std::{sync::Arc, time::Duration};

use paris::warn;
use tauri::async_runtime::RwLock;

use crate::ambient_light::config::Border;
use tokio::{
    sync::{broadcast, watch},
    time::sleep,
};

use crate::{
    ambient_light::{config, ConfigManager},
    led_color::LedColor,
    led_data_sender::{DataSendMode, LedDataSender},
    screenshot::{LedSamplePoints, Screenshot},
    screenshot_manager::ScreenshotManager,
};

use super::{ColorCalibration, LedStripConfig, LedStripConfigGroup, LedType, SamplePointMapper};

#[derive(Clone)]
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
        start_led_offset: usize,
    ) {
        let internal_tasks_version = self.inner_tasks_version.clone();
        let screenshot_manager = ScreenshotManager::global().await;

        let screenshot_rx = screenshot_manager.subscribe_by_display_id(display_id).await;

        if let Err(err) = screenshot_rx {
            log::error!("{}", err);
            return;
        }
        let mut screenshot_rx = screenshot_rx.unwrap();

        log::info!("Starting fetcher for display #{}", display_id);

        tokio::spawn(async move {
            let init_version = internal_tasks_version.read().await.clone();

            loop {
                if let Err(err) = screenshot_rx.changed().await {
                    log::error!(
                        "Screenshot channel closed for display #{}: {:?}",
                        display_id,
                        err
                    );
                    break;
                }

                let screenshot = screenshot_rx.borrow().clone();
                log::info!(
                    "Received screenshot for display #{}: {}x{}",
                    display_id,
                    screenshot.width,
                    screenshot.height
                );
                let colors = screenshot.get_colors_by_sample_points(&sample_points).await;

                log::info!(
                    "🖼️ Got screenshot for display #{}, extracted {} colors",
                    display_id,
                    colors.len()
                );

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

                log::info!(
                    "Display #{}: test_mode_active={}, ambient_light_enabled={}, colors_count={}",
                    display_id,
                    test_mode_active,
                    ambient_light_enabled,
                    colors.len()
                );

                if !test_mode_active && ambient_light_enabled {
                    match Self::send_colors_by_display(
                        colors,
                        mappers,
                        &strips,
                        &color_calibration,
                        start_led_offset,
                    )
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
                    log::info!(
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
            // Set data send mode to AmbientLight when starting ambient light worker
            let sender = LedDataSender::global().await;
            sender.set_mode(DataSendMode::AmbientLight).await;

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
        log::info!("🚀 LED color publisher starting...");

        let config_manager = ConfigManager::global().await;

        let mut config_receiver = config_manager.clone_config_update_receiver();

        // Process initial configuration first
        let initial_configs = config_receiver.borrow().clone();
        if !initial_configs.strips.is_empty() {
            log::info!("📋 Processing initial LED configuration...");
            self.handle_config_change(initial_configs).await;
        } else {
            log::warn!("⚠️ Initial LED configuration is empty, waiting for updates...");
        }

        // Then, listen for subsequent configuration changes in a separate task
        let self_clone = self.clone();
        tokio::spawn(async move {
            log::info!("👂 Listening for subsequent LED configuration changes...");
            loop {
                if config_receiver.changed().await.is_ok() {
                    let configs = config_receiver.borrow().clone();
                    if !configs.strips.is_empty() {
                        log::info!("🔄 Subsequent LED configuration changed, reprocessing...");
                        self_clone.handle_config_change(configs).await;
                    } else {
                        log::warn!("⚠️ Received empty LED configuration, skipping...");
                    }
                } else {
                    log::error!("❌ Config receiver channel closed, stopping listener.");
                    break;
                }
            }
        });
    }

    async fn create_test_config(&self) -> anyhow::Result<LedStripConfigGroup> {
        log::info!("🔧 Creating test LED configuration...");

        // Get display information
        let displays = display_info::DisplayInfo::all().map_err(|e| {
            log::error!("Failed to get display info for test config: {}", e);
            anyhow::anyhow!("Failed to get display info: {}", e)
        })?;

        log::info!("✅ Found {} displays for test config", displays.len());

        // Create a simple test configuration
        let mut strips = Vec::new();
        let mut mappers = Vec::new();

        for (i, display) in displays.iter().enumerate().take(2) {
            // Limit to 2 displays
            for j in 0..4 {
                let strip = LedStripConfig {
                    index: j + i * 4,
                    display_id: display.id,
                    border: match j {
                        0 => Border::Top,
                        1 => Border::Bottom,
                        2 => Border::Left,
                        3 => Border::Right,
                        _ => unreachable!(),
                    },
                    start_pos: j + i * 4 * 30,
                    len: 30,
                    led_type: LedType::WS2812B,
                };
                strips.push(strip);
                mappers.push(SamplePointMapper {
                    start: (j + i * 4) * 30,
                    end: (j + i * 4 + 1) * 30,
                    pos: (j + i * 4) * 30,
                });
            }
        }

        let config = LedStripConfigGroup {
            strips,
            mappers,
            color_calibration: ColorCalibration::new(),
        };

        log::info!(
            "✅ Test configuration created with {} strips",
            config.strips.len()
        );
        Ok(config)
    }

    async fn handle_config_change(&self, mut original_configs: LedStripConfigGroup) {
        // Sort strips by index to ensure correct order
        original_configs.strips.sort_by_key(|s| s.index);

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

        log::info!(
            "Processed {} sample point groups.",
            configs.sample_point_groups.len()
        );

        let (display_colors_tx, display_colors_rx) = broadcast::channel::<(u32, Vec<u8>)>(8);

        // Calculate start offsets for each display
        let mut cumulative_led_offset = 0;
        let mut display_start_offsets = std::collections::HashMap::new();
        for strip in &original_configs.strips {
            display_start_offsets
                .entry(strip.display_id)
                .or_insert(cumulative_led_offset);
            cumulative_led_offset += strip.len;
        }

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

            let start_led_offset = *display_start_offsets.get(&display_id).unwrap_or(&0);

            self.start_one_display_colors_fetcher(
                display_id,
                sample_points,
                bound_scale_factor,
                sample_point_group.mappers,
                display_colors_tx.clone(),
                display_strips,
                original_configs.color_calibration,
                start_led_offset,
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

    pub async fn send_colors(offset: u16, payload: Vec<u8>) -> anyhow::Result<()> {
        let sender = LedDataSender::global().await;
        sender.send_ambient_light_data(offset, payload).await
    }

    pub async fn send_colors_by_display(
        colors: Vec<LedColor>,
        _mappers: Vec<SamplePointMapper>, // 保留参数但不使用，避免破坏API
        strips: &[LedStripConfig],
        color_calibration: &ColorCalibration,
        start_led_offset: usize,
    ) -> anyhow::Result<()> {
        let sender = LedDataSender::global().await;

        log::info!(
            "Starting LED data send for display: colors_count={}, strips_count={}, start_offset={}",
            colors.len(),
            strips.len(),
            start_led_offset
        );

        // 直接基于strips配置发送数据，不再使用mappers
        let mut color_offset = 0;
        let mut led_offset = start_led_offset; // 硬件中的LED偏移量

        for (strip_index, strip) in strips.iter().enumerate() {
            let strip_len = strip.len;

            log::info!(
                "Processing LED strip {}: border={:?}, len={}, color_offset={}, led_offset={}, led_type={:?}",
                strip_index,
                strip.border,
                strip_len,
                color_offset,
                led_offset,
                strip.led_type
            );

            // 检查颜色数据是否足够
            if color_offset + strip_len > colors.len() {
                log::warn!(
                    "Skipping strip {}: color range {}..{} exceeds available colors ({})",
                    strip_index,
                    color_offset,
                    color_offset + strip_len,
                    colors.len()
                );
                // 仍然需要更新偏移量，即使跳过这个灯条
                color_offset += strip_len;
                led_offset += strip_len;
                continue;
            }

            let led_type = strip.led_type;

            let bytes_per_led = match led_type {
                LedType::WS2812B => 3,
                LedType::SK6812 => 4,
            };

            let mut buffer = Vec::<u8>::with_capacity(strip_len * bytes_per_led);

            // 处理这个灯条的颜色数据
            for i in 0..strip_len {
                let color_index = color_offset + i;
                if color_index < colors.len() {
                    let bytes = match led_type {
                        LedType::WS2812B => {
                            let calibration_bytes = color_calibration.to_bytes();
                            let color_bytes = colors[color_index].as_bytes();
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
                            let color_bytes = colors[color_index].as_bytes();
                            // Apply calibration and convert RGB to GRBW for SK6812-RGBW
                            vec![
                                ((color_bytes[1] as f32 * calibration_bytes[1] as f32 / 255.0)
                                    as u8), // G (Green)
                                ((color_bytes[0] as f32 * calibration_bytes[0] as f32 / 255.0)
                                    as u8), // R (Red)
                                ((color_bytes[2] as f32 * calibration_bytes[2] as f32 / 255.0)
                                    as u8), // B (Blue)
                                calibration_bytes[3], // W channel
                            ]
                        }
                    };
                    buffer.extend_from_slice(&bytes);
                } else {
                    log::warn!(
                        "Color index {} out of bounds for colors array of length {}",
                        color_index,
                        colors.len()
                    );
                    // Add black color as fallback
                    match led_type {
                        LedType::WS2812B => buffer.extend_from_slice(&[0, 0, 0]),
                        LedType::SK6812 => buffer.extend_from_slice(&[0, 0, 0, 0]),
                    }
                }
            }

            // 计算字节偏移量（基于LED偏移量和LED类型）
            let byte_offset = led_offset * bytes_per_led;

            log::info!(
                "Sending LED data: strip={}, led_offset={}, byte_offset={}, buffer_size={}",
                strip_index,
                led_offset,
                byte_offset,
                buffer.len()
            );

            if !buffer.is_empty() {
                log::info!(
                    "📤 Attempting to send LED data for strip {}: {} bytes",
                    strip_index,
                    buffer.len()
                );

                match sender
                    .send_ambient_light_data(byte_offset as u16, buffer)
                    .await
                {
                    Ok(_) => {
                        log::info!("✅ Successfully sent LED data for strip {}", strip_index);
                    }
                    Err(e) => {
                        log::error!(
                            "❌ Failed to send LED data for strip {}: {}",
                            strip_index,
                            e
                        );
                        // Continue with next strip instead of returning error
                    }
                }

                // Add a small delay between packets to avoid overwhelming the network
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            } else {
                log::warn!("Empty buffer for strip {}, skipping", strip_index);
            }

            // 更新偏移量，为下一个灯条做准备
            color_offset += strip_len;
            led_offset += strip_len;
        }

        Ok(())
    }

    pub async fn clone_sorted_colors_receiver(&self) -> watch::Receiver<Vec<u8>> {
        self.sorted_colors_rx.read().await.clone()
    }
    pub async fn get_colors_configs(
        configs: &LedStripConfigGroup,
    ) -> anyhow::Result<AllColorConfig> {
        // Get actual display information and assign IDs if needed
        let displays = display_info::DisplayInfo::all().map_err(|e| {
            log::error!("Failed to get display info in get_colors_configs: {}", e);
            anyhow::anyhow!("Failed to get display info: {}", e)
        })?;

        // Create a mutable copy of configs with proper display IDs
        let mut updated_configs = configs.clone();
        for strip in updated_configs.strips.iter_mut() {
            if strip.display_id == 0 {
                // Assign display ID based on strip index
                let display_index = strip.index / 4;
                if display_index < displays.len() {
                    strip.display_id = displays[display_index].id;
                    log::info!(
                        "Assigned display ID {} to strip {}",
                        strip.display_id,
                        strip.index
                    );
                }
            }
        }

        let mappers = updated_configs.mappers.clone();

        let mut colors_configs = Vec::new();

        for display_info in displays {
            let display_id = display_info.id;

            let mut led_strip_configs: Vec<_> = updated_configs
                .strips
                .iter()
                .filter(|c| c.display_id == display_id)
                .cloned()
                .collect();

            if led_strip_configs.is_empty() {
                warn!(
                    "No LED strip config for display_id: {}, using default.",
                    display_id
                );
                led_strip_configs.push(LedStripConfig::default_for_display(
                    display_id,
                    updated_configs.strips.len(),
                ));
            }

            // Create a dummy screenshot object to calculate sample points
            let dummy_screenshot = Screenshot::new(
                display_id,
                display_info.height,
                display_info.width,
                0, // bytes_per_row is not used for sample point calculation
                Arc::new(vec![]),
                display_info.scale_factor as f32,
                display_info.scale_factor as f32,
            );

            let points: Vec<_> = led_strip_configs
                .iter()
                .map(|config| dummy_screenshot.get_sample_points(config))
                .flatten()
                .collect();

            if points.is_empty() {
                warn!("No sample points generated for display_id: {}", display_id);
                continue;
            }

            let display_mappers = updated_configs
                .mappers
                .iter()
                .zip(&updated_configs.strips)
                .filter(|(_, strip)| strip.display_id == display_id)
                .map(|(mapper, _)| mapper.clone())
                .collect();

            let colors_config = DisplaySamplePointGroup {
                display_id,
                points,
                bound_scale_factor: display_info.scale_factor as f32,
                mappers: display_mappers,
            };

            colors_configs.push(colors_config);
        }

        Ok(AllColorConfig {
            sample_point_groups: colors_configs,
            mappers,
        })
    }

    pub async fn clone_colors_receiver(&self) -> watch::Receiver<Vec<u8>> {
        self.colors_rx.read().await.clone()
    }

    /// Enable test mode - this will pause normal LED data publishing
    pub async fn enable_test_mode(&self) {
        let mut test_mode = self.test_mode_active.write().await;
        *test_mode = true;

        // Set data send mode to None to pause ambient light data sending
        let sender = LedDataSender::global().await;
        sender.set_mode(DataSendMode::None).await;

        log::info!("Test mode enabled - normal LED publishing paused");
    }

    /// Disable test mode - this will resume normal LED data publishing
    pub async fn disable_test_mode(&self) {
        let mut test_mode = self.test_mode_active.write().await;
        *test_mode = false;

        // Set data send mode back to AmbientLight to resume normal publishing
        let sender = LedDataSender::global().await;
        sender.set_mode(DataSendMode::AmbientLight).await;

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

#[derive(Debug, Clone)]
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
