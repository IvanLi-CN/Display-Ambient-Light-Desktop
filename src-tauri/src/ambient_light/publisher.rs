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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BorderColors {
    pub top: [[u8; 3]; 2],    // 两种RGB颜色 [第一种, 第二种]
    pub bottom: [[u8; 3]; 2], // 两种RGB颜色 [第一种, 第二种]
    pub left: [[u8; 3]; 2],   // 两种RGB颜色 [第一种, 第二种]
    pub right: [[u8; 3]; 2],  // 两种RGB颜色 [第一种, 第二种]
}

#[derive(Clone)]
pub struct LedColorsPublisher {
    sorted_colors_rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    sorted_colors_tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
    colors_rx: Arc<RwLock<watch::Receiver<Vec<u8>>>>,
    colors_tx: Arc<RwLock<watch::Sender<Vec<u8>>>>,
    inner_tasks_version: Arc<RwLock<usize>>,
    test_mode_active: Arc<RwLock<bool>>,
    single_display_config_mode: Arc<RwLock<bool>>,
    single_display_config_data: Arc<RwLock<Option<(Vec<LedStripConfig>, BorderColors)>>>,
    active_strip_for_breathing: Arc<RwLock<Option<(u32, String)>>>, // (display_id, border)
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
                    single_display_config_mode: Arc::new(RwLock::new(false)),
                    single_display_config_data: Arc::new(RwLock::new(None)),
                    active_strip_for_breathing: Arc::new(RwLock::new(None)),
                }
            })
            .await
    }

    async fn start_one_display_colors_fetcher(
        &self,
        display_id: u32,
        _sample_points: Vec<LedSamplePoints>, // 不再使用旧的采样点，改用LED配置
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

                // 使用新的采样函数替换旧的采样逻辑
                // 只处理属于当前显示器的LED灯带配置
                let current_display_strips: Vec<LedStripConfig> = strips
                    .iter()
                    .filter(|strip| strip.display_id == display_id)
                    .cloned()
                    .collect();

                log::info!(
                    "Display #{}: Processing {} LED strips for this display",
                    display_id,
                    current_display_strips.len()
                );

                let colors_by_strips = screenshot
                    .get_colors_by_led_configs(&current_display_strips)
                    .await;

                // 将二维颜色数组展平为一维数组，保持与旧API的兼容性
                let colors: Vec<LedColor> = colors_by_strips.into_iter().flatten().collect();

                log::info!(
                    "🖼️ Got screenshot for display #{}, extracted {} colors using new sampling algorithm",
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
                    // In test mode or when ambient light is disabled, skip sending
                    // The test mode will handle its own data sending
                    if test_mode_active {
                        log::debug!(
                            "Skipping ambient light data for display #{}: test mode active",
                            display_id
                        );
                    } else {
                        log::info!(
                            "Skipping color send for display #{}: test_mode={}, enabled={}",
                            display_id,
                            test_mode_active,
                            ambient_light_enabled
                        );
                    }
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

                    match sorted_colors_tx.send(sorted_colors.clone()) {
                        Ok(_) => {}
                        Err(err) => {
                            warn!("Failed to send sorted colors: {}", err);
                        }
                    };

                    // 通过WebSocket广播颜色变化
                    crate::websocket_events::publish_led_colors_changed(flatten_colors).await;
                    crate::websocket_events::WebSocketEventPublisher::global()
                        .await
                        .publish_led_sorted_colors_changed(sorted_colors)
                        .await;

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

        // Get the updated configs with proper display IDs assigned
        let updated_configs = Self::get_updated_configs_with_display_ids(&original_configs).await;
        if let Err(err) = updated_configs {
            warn!("Failed to get updated configs: {}", err);
            return;
        }
        let updated_configs = updated_configs.unwrap();

        let (display_colors_tx, display_colors_rx) = broadcast::channel::<(u32, Vec<u8>)>(8);

        // Calculate start offsets for each display using updated configs
        // 按序列号排序灯带，确保正确的串联顺序
        let mut sorted_strips = updated_configs.strips.clone();
        sorted_strips.sort_by_key(|strip| strip.index);

        let mut display_start_offsets = std::collections::HashMap::new();
        let mut cumulative_led_offset = 0;

        for strip in &sorted_strips {
            // 为每个显示器记录其第一个灯带的起始偏移量
            display_start_offsets
                .entry(strip.display_id)
                .or_insert(cumulative_led_offset);
            cumulative_led_offset += strip.len;
        }

        log::info!("计算的显示器起始偏移量: {:?}", display_start_offsets);

        for sample_point_group in configs.sample_point_groups.clone() {
            let display_id = sample_point_group.display_id;
            let sample_points = sample_point_group.points;
            let bound_scale_factor = sample_point_group.bound_scale_factor;

            // Get strips for this display using updated configs
            let display_strips: Vec<LedStripConfig> = updated_configs
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
                updated_configs.color_calibration,
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
        sender
            .send_complete_led_data(offset, payload, "AmbientLight")
            .await
    }

    /// Get updated configs with proper display IDs assigned
    async fn get_updated_configs_with_display_ids(
        configs: &LedStripConfigGroup,
    ) -> anyhow::Result<LedStripConfigGroup> {
        let displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get displays: {}", e))?;

        // Log display detection order for debugging
        log::info!("🖥️ Detected displays in order:");
        for (i, display) in displays.iter().enumerate() {
            log::info!(
                "  Display {}: ID={}, X={}, Y={}, Primary={}",
                i,
                display.id,
                display.x,
                display.y,
                display.is_primary
            );
        }

        // Create a mutable copy of configs with proper display IDs
        let mut updated_configs = configs.clone();
        for strip in updated_configs.strips.iter_mut() {
            if strip.display_id == 0 {
                // Assign display ID based on strip index
                let display_index = strip.index / 4;
                if display_index < displays.len() {
                    // TEMPORARY FIX: Reverse display order to match UI layout
                    // This fixes the issue where display detection order doesn't match UI order
                    let corrected_display_index = if displays.len() == 2 {
                        1 - display_index // Swap 0->1, 1->0 for 2 displays
                    } else {
                        display_index // Keep original for other cases
                    };

                    if corrected_display_index < displays.len() {
                        strip.display_id = displays[corrected_display_index].id;
                        log::info!(
                            "Assigned display ID {} to strip {} (original_index={}, corrected_index={})",
                            strip.display_id,
                            strip.index,
                            display_index,
                            corrected_display_index
                        );
                    }
                }
            }
        }

        Ok(updated_configs)
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

        // 按序列号排序灯带，确保正确的串联顺序
        let mut sorted_strips: Vec<_> = strips.iter().collect();
        sorted_strips.sort_by_key(|strip| strip.index);

        log::info!(
            "排序后的灯带顺序: {:?}",
            sorted_strips
                .iter()
                .map(|s| (s.index, s.border, s.display_id))
                .collect::<Vec<_>>()
        );

        // 第一步：合并所有LED数据到一个完整的数据流
        let mut complete_led_data = Vec::<u8>::new();
        let mut color_offset = 0;
        let mut total_leds = 0;

        log::info!("🔄 Step 1: Merging all LED strip data into complete data stream");

        for (strip_index, strip) in sorted_strips.iter().enumerate() {
            let strip_len = strip.len;

            log::info!(
                "Merging LED strip {}: border={:?}, len={}, color_offset={}, led_type={:?}",
                strip_index,
                strip.border,
                strip_len,
                color_offset,
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
                continue;
            }

            let led_type = strip.led_type;

            // 将这个灯带的数据添加到完整数据流中
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
                    complete_led_data.extend_from_slice(&bytes);
                } else {
                    log::warn!(
                        "Color index {} out of bounds for colors array of length {}",
                        color_index,
                        colors.len()
                    );
                    // Add black color as fallback
                    match led_type {
                        LedType::WS2812B => complete_led_data.extend_from_slice(&[0, 0, 0]),
                        LedType::SK6812 => complete_led_data.extend_from_slice(&[0, 0, 0, 0]),
                    }
                }
            }

            total_leds += strip_len;
        }

        log::info!(
            "✅ Step 1 complete: Merged {} LEDs into {} bytes of data",
            total_leds,
            complete_led_data.len()
        );

        // 第二步：将完整数据流提交给统一的发布服务进行拆包
        log::info!("📦 Step 2: Submitting complete data to unified publisher for packet splitting");

        let byte_offset = start_led_offset * 3; // 计算字节偏移量

        sender
            .send_complete_led_data(byte_offset as u16, complete_led_data, "AmbientLight")
            .await?;

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

        // Log display detection order for debugging
        log::info!("🖥️ get_colors_configs - Detected displays in order:");
        for (i, display) in displays.iter().enumerate() {
            log::info!(
                "  Display {}: ID={}, X={}, Y={}, Primary={}",
                i,
                display.id,
                display.x,
                display.y,
                display.is_primary
            );
        }

        // Create a mutable copy of configs with proper display IDs
        let mut updated_configs = configs.clone();
        for strip in updated_configs.strips.iter_mut() {
            if strip.display_id == 0 {
                // Assign display ID based on strip index
                let display_index = strip.index / 4;
                if display_index < displays.len() {
                    // TEMPORARY FIX: Reverse display order to match UI layout
                    // This fixes the issue where display detection order doesn't match UI order
                    let corrected_display_index = if displays.len() == 2 {
                        1 - display_index // Swap 0->1, 1->0 for 2 displays
                    } else {
                        display_index // Keep original for other cases
                    };

                    if corrected_display_index < displays.len() {
                        strip.display_id = displays[corrected_display_index].id;
                        log::info!(
                            "get_colors_configs - Assigned display ID {} to strip {} (original_index={}, corrected_index={})",
                            strip.display_id,
                            strip.index,
                            display_index,
                            corrected_display_index
                        );
                    }
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

            // 按序列号排序，确保与send_colors_by_display中的顺序一致
            led_strip_configs.sort_by_key(|strip| strip.index);

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

    /// Enable test mode - this will pause normal LED data publishing but keep connection alive
    pub async fn enable_test_mode(&self) {
        let mut test_mode = self.test_mode_active.write().await;
        *test_mode = true;

        // Keep data send mode as AmbientLight to maintain connection with hardware
        // The test mode flag will prevent actual color data from being sent
        let sender = LedDataSender::global().await;
        sender.set_mode(DataSendMode::AmbientLight).await;

        log::info!("Test mode enabled - normal LED publishing paused but connection maintained");
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

    /// Check if test mode is currently active
    pub async fn is_test_mode_active(&self) -> bool {
        *self.test_mode_active.read().await
    }

    /// 启动单屏灯带配置定位色发布模式
    pub async fn start_single_display_config_mode(
        &self,
        strips: Vec<LedStripConfig>,
        border_colors: BorderColors,
    ) -> anyhow::Result<()> {
        log::info!("🎯 启动单屏灯带配置定位色发布模式");
        log::info!("🔄 收到 {} 个灯带配置", strips.len());

        // 首先停止所有当前的发布任务，避免冲突
        {
            let mut version = self.inner_tasks_version.write().await;
            *version += 1;
        }
        log::info!("✅ 已停止所有当前发布任务（增加任务版本号）");

        // 等待一段时间确保所有任务完全停止
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        log::info!("⏰ 等待任务完全停止");

        // 设置LED数据发送模式为StripConfig
        let sender = crate::led_data_sender::LedDataSender::global().await;
        sender
            .set_mode(crate::led_data_sender::DataSendMode::StripConfig)
            .await;
        log::info!("✅ 设置LED数据发送模式为StripConfig");

        // 验证模式设置是否成功
        let current_mode = sender.get_mode().await;
        log::info!("🔍 当前LED数据发送模式: {:?}", current_mode);

        // 设置目标硬件地址（如果有可用的硬件设备）
        let rpc = crate::rpc::UdpRpc::global().await;
        if let Ok(rpc) = rpc {
            let boards = rpc.get_boards().await;
            if !boards.is_empty() {
                let target_addr = format!("{}:{}", boards[0].address, boards[0].port);
                sender.set_test_target(Some(target_addr.clone())).await;
                log::info!("✅ 设置目标硬件地址为: {}", target_addr);
            } else {
                log::warn!("⚠️ 没有找到可用的硬件设备，将使用广播模式");
                sender.set_test_target(None).await;
            }
        } else {
            log::warn!("⚠️ UDP RPC不可用，将使用广播模式");
            sender.set_test_target(None).await;
        }

        // 设置单屏配置模式数据
        {
            let mut mode = self.single_display_config_mode.write().await;
            *mode = true;
        }

        {
            let mut data = self.single_display_config_data.write().await;
            *data = Some((strips.clone(), border_colors.clone()));
        }

        // 生成 mappers 信息
        let mut config_group = LedStripConfigGroup {
            strips: strips.clone(),
            mappers: Vec::new(),
            color_calibration: ColorCalibration::new(),
        };
        config_group.generate_mappers();
        log::info!("✅ 生成了 {} 个 mappers", config_group.mappers.len());

        // 启动30Hz发布任务
        log::info!("� 启动单屏配置模式30Hz发布任务");
        self.start_single_display_config_task(config_group, border_colors)
            .await;

        Ok(())
    }

    /// 停止单屏灯带配置定位色发布模式
    pub async fn stop_single_display_config_mode(&self) -> anyhow::Result<()> {
        log::info!("🛑 停止单屏灯带配置定位色发布模式");

        {
            let mut mode = self.single_display_config_mode.write().await;
            *mode = false;
        }

        {
            let mut data = self.single_display_config_data.write().await;
            *data = None;
        }

        // 清除活跃灯带状态
        {
            let mut active_strip = self.active_strip_for_breathing.write().await;
            *active_strip = None;
        }

        // 增加任务版本号以停止现有任务
        {
            let mut version = self.inner_tasks_version.write().await;
            *version += 1;
        }

        // 恢复LED数据发送模式为AmbientLight
        let sender = crate::led_data_sender::LedDataSender::global().await;
        sender
            .set_mode(crate::led_data_sender::DataSendMode::AmbientLight)
            .await;
        log::info!("✅ 恢复LED数据发送模式为AmbientLight");

        log::info!("✅ 单屏灯带配置定位色发布模式已停止");
        Ok(())
    }

    /// 设置活跃灯带用于呼吸效果
    pub async fn set_active_strip_for_breathing(
        &self,
        display_id: u32,
        border: Option<String>,
    ) -> anyhow::Result<()> {
        log::info!("🫁 设置活跃灯带用于呼吸效果");
        log::info!("   - 显示器ID: {}", display_id);
        log::info!("   - 边框: {:?}", border);

        {
            let mut active_strip = self.active_strip_for_breathing.write().await;
            *active_strip = border.map(|b| (display_id, b));
        }

        log::info!("✅ 活跃灯带状态已更新");
        Ok(())
    }

    /// 启动单屏配置模式的30Hz发布任务
    async fn start_single_display_config_task(
        &self,
        config_group: LedStripConfigGroup,
        border_colors: BorderColors,
    ) {
        log::info!("🔄 start_single_display_config_task 方法开始执行");
        log::info!("🔄 配置包含 {} 个灯带", config_group.strips.len());

        let current_version = {
            let mut version = self.inner_tasks_version.write().await;
            *version += 1;
            *version
        };

        let publisher = self.clone();
        let inner_tasks_version = self.inner_tasks_version.clone();

        tokio::spawn(async move {
            log::info!(
                "🚀 启动单屏配置模式30Hz发布任务 (版本: {})",
                current_version
            );

            let mut interval = tokio::time::interval(Duration::from_millis(33)); // 30Hz

            loop {
                interval.tick().await;

                // 检查任务版本是否已更改
                let version = inner_tasks_version.read().await.clone();
                if version != current_version {
                    log::info!(
                        "🛑 单屏配置模式任务版本已更改，停止任务 ({} != {})",
                        version,
                        current_version
                    );
                    break;
                }

                // 生成并发布定位色数据
                if let Err(e) = publisher
                    .generate_and_publish_config_colors(&config_group, &border_colors)
                    .await
                {
                    log::error!("❌ 生成和发布定位色数据失败: {}", e);
                }
            }

            log::info!("✅ 单屏配置模式30Hz发布任务结束");
        });
    }

    /// 生成并发布定位色数据
    async fn generate_and_publish_config_colors(
        &self,
        config_group: &LedStripConfigGroup,
        border_colors: &BorderColors,
    ) -> anyhow::Result<()> {
        // 1. 根据边框颜色常量生成四个边的颜色数据
        let edge_colors = self.generate_edge_colors_from_constants(border_colors);

        // 2. 读取完整的LED灯带配置以计算正确的全局偏移量
        let config_manager = crate::ambient_light::ConfigManager::global().await;
        let all_configs = config_manager.configs().await;

        // 3. 检查是否有活跃灯带需要呼吸效果
        let active_strip = {
            let active_strip_guard = self.active_strip_for_breathing.read().await;
            active_strip_guard.clone()
        };

        // 4. 使用采样映射函数将数据映射到完整灯带数据串缓冲区，并应用呼吸效果
        let (complete_buffer, global_start_offset) = self
            .map_edge_colors_to_led_buffer_with_breathing(
                config_group,
                &all_configs,
                &edge_colors,
                active_strip,
            )?;

        // 5. 委托发布服务将数据发给硬件，使用正确的全局偏移量
        let sender = LedDataSender::global().await;
        sender
            .send_complete_led_data(global_start_offset, complete_buffer, "StripConfig")
            .await?;

        Ok(())
    }

    /// 根据边框颜色常量生成四个边的颜色数据（支持双色分段）
    pub fn generate_edge_colors_from_constants(
        &self,
        border_colors: &BorderColors,
    ) -> std::collections::HashMap<Border, [LedColor; 2]> {
        let mut edge_colors = std::collections::HashMap::new();

        // Top边：蓝色 + 紫色
        edge_colors.insert(
            Border::Top,
            [
                LedColor::new(
                    border_colors.top[0][0],
                    border_colors.top[0][1],
                    border_colors.top[0][2],
                ), // 第一种颜色
                LedColor::new(
                    border_colors.top[1][0],
                    border_colors.top[1][1],
                    border_colors.top[1][2],
                ), // 第二种颜色
            ],
        );

        // Bottom边：深橙色 + 黄色
        edge_colors.insert(
            Border::Bottom,
            [
                LedColor::new(
                    border_colors.bottom[0][0],
                    border_colors.bottom[0][1],
                    border_colors.bottom[0][2],
                ),
                LedColor::new(
                    border_colors.bottom[1][0],
                    border_colors.bottom[1][1],
                    border_colors.bottom[1][2],
                ),
            ],
        );

        // Left边：玫红色 + 红色
        edge_colors.insert(
            Border::Left,
            [
                LedColor::new(
                    border_colors.left[0][0],
                    border_colors.left[0][1],
                    border_colors.left[0][2],
                ),
                LedColor::new(
                    border_colors.left[1][0],
                    border_colors.left[1][1],
                    border_colors.left[1][2],
                ),
            ],
        );

        // Right边：纯绿色 + 青色
        edge_colors.insert(
            Border::Right,
            [
                LedColor::new(
                    border_colors.right[0][0],
                    border_colors.right[0][1],
                    border_colors.right[0][2],
                ),
                LedColor::new(
                    border_colors.right[1][0],
                    border_colors.right[1][1],
                    border_colors.right[1][2],
                ),
            ],
        );

        edge_colors
    }

    /// 使用采样映射函数将边框颜色映射到LED数据缓冲区（兼容旧接口，用于测试）
    pub fn map_edge_colors_to_led_buffer(
        &self,
        config_group: &LedStripConfigGroup,
        edge_colors: &std::collections::HashMap<Border, [LedColor; 2]>,
    ) -> anyhow::Result<Vec<u8>> {
        // 简化实现，专门用于测试，不包含呼吸效果
        let mut sorted_strips = config_group.strips.clone();
        sorted_strips.sort_by_key(|s| s.index);

        let mut buffer = Vec::new();

        for strip in &sorted_strips {
            let default_colors = [LedColor::new(0, 0, 0), LedColor::new(0, 0, 0)];
            let colors = edge_colors.get(&strip.border).unwrap_or(&default_colors);

            for physical_index in 0..strip.len {
                let logical_index = if strip.reversed {
                    strip.len - 1 - physical_index
                } else {
                    physical_index
                };

                let half_count = strip.len / 2;
                let color = if logical_index < half_count {
                    &colors[0]
                } else {
                    &colors[1]
                };
                let rgb = color.get_rgb();

                match strip.led_type {
                    LedType::WS2812B => {
                        buffer.push(rgb[1]); // G
                        buffer.push(rgb[0]); // R
                        buffer.push(rgb[2]); // B
                    }
                    LedType::SK6812 => {
                        buffer.push(rgb[1]); // G
                        buffer.push(rgb[0]); // R
                        buffer.push(rgb[2]); // B
                        buffer.push(0); // W
                    }
                }
            }
        }

        Ok(buffer)
    }

    /// 使用采样映射函数将边框颜色映射到完整灯带数据串缓冲区，并为活跃灯带应用呼吸效果
    pub fn map_edge_colors_to_led_buffer_with_breathing(
        &self,
        config_group: &LedStripConfigGroup,
        all_configs: &LedStripConfigGroup,
        edge_colors: &std::collections::HashMap<Border, [LedColor; 2]>,
        active_strip: Option<(u32, String)>, // (display_id, border)
    ) -> anyhow::Result<(Vec<u8>, u16)> {
        // 按序列号排序所有灯带
        let mut all_sorted_strips = all_configs.strips.clone();
        all_sorted_strips.sort_by_key(|s| s.index);

        // 计算总LED数量和总字节数
        let total_leds: usize = all_sorted_strips.iter().map(|s| s.len).sum();
        let total_bytes: usize = all_sorted_strips
            .iter()
            .map(|s| {
                let bytes_per_led = match s.led_type {
                    LedType::WS2812B => 3,
                    LedType::SK6812 => 4,
                };
                s.len * bytes_per_led
            })
            .sum();

        log::info!(
            "🎨 生成完整LED数据流(带呼吸效果): 总LED数={}, 总字节数={}",
            total_leds,
            total_bytes
        );

        // 获取当前显示器的灯带ID集合
        let current_display_strips: std::collections::HashSet<usize> =
            config_group.strips.iter().map(|s| s.index).collect();

        // 简单的正弦函数呼吸效果 - 1Hz频率
        let time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let time_seconds = time_ms as f64 / 1000.0;

        // 1Hz正弦波，范围从0.3到1.0 (30%到100%亮度)
        let breathing_factor = (time_seconds * std::f64::consts::PI).sin() * 0.5 + 0.5; // 0到1
        let breathing_brightness = (0.3 + 0.7 * breathing_factor) as f32; // 30%到100%

        // 定义填充颜色：如果有活跃灯带则用白色填充，否则用黑色（保持原有行为）
        let (fill_rgb, fill_w) = if active_strip.is_some() {
            ([51, 51, 51], 51) // 白色填充（20%亮度）
        } else {
            ([0, 0, 0], 0) // 黑色填充（关闭）
        };

        let mut buffer = Vec::new();

        // 遍历所有灯带，按序列号顺序生成完整的LED数据流
        for strip in &all_sorted_strips {
            let is_current_display = current_display_strips.contains(&strip.index);

            if is_current_display {
                // 当前显示器的灯带：显示定位色
                let default_colors = [LedColor::new(0, 0, 0), LedColor::new(0, 0, 0)];
                let colors = edge_colors.get(&strip.border).unwrap_or(&default_colors);

                // 检查是否是活跃灯带
                let is_active_strip =
                    if let Some((active_display_id, ref active_border)) = active_strip {
                        strip.display_id == active_display_id
                            && format!("{:?}", strip.border).to_lowercase()
                                == active_border.to_lowercase()
                    } else {
                        false
                    };

                // 计算分段：前半部分用第一种颜色，后半部分用第二种颜色
                let half_count = strip.len / 2;

                if is_active_strip {
                    // 只在特定时间间隔输出日志，避免过多输出
                    if (time_ms / 200) % 5 == 0 {
                        // 每秒输出一次
                        log::info!("🫁 活跃灯带 {} ({}边): {} LEDs, 时间: {:.1}s, 呼吸因子: {:.3}, 亮度: {:.2}",
                            strip.index,
                            match strip.border {
                                Border::Top => "Top",
                                Border::Bottom => "Bottom",
                                Border::Left => "Left",
                                Border::Right => "Right",
                            },
                            strip.len,
                            time_seconds % 2.0, // 显示2秒周期内的位置
                            breathing_factor,
                            breathing_brightness
                        );
                    }
                } else {
                    log::debug!(
                        "🎨 当前显示器灯带 {} ({}边): {} LEDs, 非活跃",
                        strip.index,
                        match strip.border {
                            Border::Top => "Top",
                            Border::Bottom => "Bottom",
                            Border::Left => "Left",
                            Border::Right => "Right",
                        },
                        strip.len
                    );
                }

                // 为该灯带的所有LED生成定位色数据
                for physical_index in 0..strip.len {
                    // 根据reversed字段决定逻辑索引
                    let logical_index = if strip.reversed {
                        strip.len - 1 - physical_index // 反向：最后一个LED对应第一个逻辑位置
                    } else {
                        physical_index // 正向：物理索引等于逻辑索引
                    };

                    // 选择颜色：前半部分用第一种，后半部分用第二种（基于逻辑索引）
                    let color = if logical_index < half_count {
                        &colors[0] // 第一种颜色
                    } else {
                        &colors[1] // 第二种颜色
                    };
                    let mut rgb = color.get_rgb();

                    // 如果是活跃灯带，应用优雅的呼吸效果
                    if is_active_strip {
                        rgb[0] = (rgb[0] as f32 * breathing_brightness) as u8;
                        rgb[1] = (rgb[1] as f32 * breathing_brightness) as u8;
                        rgb[2] = (rgb[2] as f32 * breathing_brightness) as u8;
                    }

                    match strip.led_type {
                        LedType::WS2812B => {
                            // GRB格式
                            buffer.push(rgb[1]); // G
                            buffer.push(rgb[0]); // R
                            buffer.push(rgb[2]); // B
                        }
                        LedType::SK6812 => {
                            // GRBW格式
                            buffer.push(rgb[1]); // G
                            buffer.push(rgb[0]); // R
                            buffer.push(rgb[2]); // B
                            buffer.push(0); // W通道设为0
                        }
                    }
                }
            } else {
                // 其他显示器的灯带：根据是否有活跃灯带决定填充颜色
                let fill_description = if active_strip.is_some() {
                    "白色填充20%亮度"
                } else {
                    "黑色填充(关闭)"
                };
                log::debug!(
                    "🔲 其他显示器灯带 {} ({}边): {} LEDs, {}",
                    strip.index,
                    match strip.border {
                        Border::Top => "Top",
                        Border::Bottom => "Bottom",
                        Border::Left => "Left",
                        Border::Right => "Right",
                    },
                    strip.len,
                    fill_description
                );

                // 为该灯带的所有LED生成填充数据
                for _led_index in 0..strip.len {
                    match strip.led_type {
                        LedType::WS2812B => {
                            // GRB格式
                            buffer.push(fill_rgb[1]); // G
                            buffer.push(fill_rgb[0]); // R
                            buffer.push(fill_rgb[2]); // B
                        }
                        LedType::SK6812 => {
                            // GRBW格式
                            if active_strip.is_some() {
                                // 有活跃灯带时，只亮W通道
                                buffer.push(0); // G = 0
                                buffer.push(0); // R = 0
                                buffer.push(0); // B = 0
                                buffer.push(fill_w); // W
                            } else {
                                // 无活跃灯带时，全部关闭
                                buffer.push(fill_rgb[1]); // G
                                buffer.push(fill_rgb[0]); // R
                                buffer.push(fill_rgb[2]); // B
                                buffer.push(fill_w); // W
                            }
                        }
                    }
                }
            }
        }

        log::info!(
            "🎨 生成了完整的LED数据缓冲区(带呼吸效果): {} 字节 (总LED数: {}), 从偏移量0开始发送",
            buffer.len(),
            total_leds
        );

        // 验证生成的数据长度是否正确
        if buffer.len() != total_bytes {
            log::warn!(
                "⚠️ 数据长度不匹配: 期望{}字节, 实际{}字节",
                total_bytes,
                buffer.len()
            );
        }

        // 返回完整的LED数据流，从偏移量0开始
        Ok((buffer, 0))
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

#[cfg(test)]
mod tests {
    use crate::ambient_light::config::{Border, ColorCalibration, LedStripConfig, LedType};
    use crate::led_color::LedColor;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock LedDataSender to capture sent data instead of sending it over UDP
    struct MockLedDataSender {
        sent_data: Arc<Mutex<Vec<(u16, Vec<u8>)>>>,
    }

    impl MockLedDataSender {
        fn new() -> Self {
            Self {
                sent_data: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn send_ambient_light_data(
            &self,
            offset: u16,
            payload: Vec<u8>,
        ) -> anyhow::Result<()> {
            self.sent_data.lock().await.push((offset, payload));
            Ok(())
        }

        async fn get_sent_data(&self) -> Vec<(u16, Vec<u8>)> {
            self.sent_data.lock().await.clone()
        }
    }

    // We cannot directly test the original `send_colors_by_display` because it uses a global `LedDataSender`.
    // We create a testable version that accepts a mock sender.
    async fn testable_send_colors_by_display(
        sender: &MockLedDataSender,
        colors: Vec<LedColor>,
        strips: &[LedStripConfig],
        color_calibration: &ColorCalibration,
        start_led_offset: usize,
    ) -> anyhow::Result<()> {
        let mut color_offset = 0;
        let mut led_offset = start_led_offset;

        for strip in strips {
            let strip_len = strip.len;
            if color_offset + strip_len > colors.len() {
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

            for i in 0..strip_len {
                let color_index = color_offset + i;
                let bytes = match led_type {
                    LedType::WS2812B => {
                        let cal = color_calibration.to_bytes();
                        let col = colors[color_index].as_bytes();
                        vec![
                            ((col[1] as f32 * cal[1] as f32 / 255.0) as u8), // G
                            ((col[0] as f32 * cal[0] as f32 / 255.0) as u8), // R
                            ((col[2] as f32 * cal[2] as f32 / 255.0) as u8), // B
                        ]
                    }
                    LedType::SK6812 => {
                        let cal = color_calibration.to_bytes_rgbw();
                        let col = colors[color_index].as_bytes();
                        vec![
                            ((col[1] as f32 * cal[1] as f32 / 255.0) as u8), // G
                            ((col[0] as f32 * cal[0] as f32 / 255.0) as u8), // R
                            ((col[2] as f32 * cal[2] as f32 / 255.0) as u8), // B
                            cal[3],                                          // W
                        ]
                    }
                };
                buffer.extend_from_slice(&bytes);
            }

            let byte_offset = led_offset * bytes_per_led;
            if !buffer.is_empty() {
                sender
                    .send_ambient_light_data(byte_offset as u16, buffer)
                    .await?;
            }

            color_offset += strip_len;
            led_offset += strip_len;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_ws2812b_color_transformation_and_calibration() {
        let sender = MockLedDataSender::new();
        let colors = vec![LedColor::new(255, 128, 64)]; // R, G, B
        let strips = vec![LedStripConfig {
            index: 0,
            border: Border::Top,
            display_id: 1,
            len: 1,
            led_type: LedType::WS2812B,
            reversed: false,
        }];
        let mut calibration = ColorCalibration::new();
        calibration.r = 0.5; // Halve the red channel

        testable_send_colors_by_display(&sender, colors, &strips, &calibration, 0)
            .await
            .unwrap();

        let sent_data = sender.get_sent_data().await;
        assert_eq!(sent_data.len(), 1);
        let (offset, payload) = &sent_data[0];
        assert_eq!(*offset, 0);
        // Expected: G, R, B -> 128, 255*0.5, 64 -> [128, 127, 64]
        assert_eq!(*payload, vec![128, 127, 64]);
    }

    #[tokio::test]
    async fn test_sk6812_color_transformation_and_w_channel() {
        let sender = MockLedDataSender::new();
        let colors = vec![LedColor::new(255, 128, 64)]; // R, G, B
        let strips = vec![LedStripConfig {
            index: 0,
            border: Border::Top,
            display_id: 1,
            len: 1,
            led_type: LedType::SK6812,
            reversed: false,
        }];
        let mut calibration = ColorCalibration::new();
        calibration.w = 0.8; // Set white channel to 80%

        testable_send_colors_by_display(&sender, colors, &strips, &calibration, 0)
            .await
            .unwrap();

        let sent_data = sender.get_sent_data().await;
        assert_eq!(sent_data.len(), 1);
        let (offset, payload) = &sent_data[0];
        assert_eq!(*offset, 0);
        // Expected: G, R, B, W -> 128, 255, 64, 255*0.8 -> [128, 255, 64, 204]
        assert_eq!(*payload, vec![128, 255, 64, 204]);
    }

    #[tokio::test]
    async fn test_led_offset_calculation() {
        let sender = MockLedDataSender::new();
        let colors = vec![LedColor::new(10, 20, 30), LedColor::new(40, 50, 60)];
        let strips = vec![
            LedStripConfig {
                len: 1,
                led_type: LedType::WS2812B,
                ..Default::default()
            },
            LedStripConfig {
                len: 1,
                led_type: LedType::WS2812B,
                ..Default::default()
            },
        ];
        let calibration = ColorCalibration::new();

        // Start with a hardware LED offset of 10
        testable_send_colors_by_display(&sender, colors, &strips, &calibration, 10)
            .await
            .unwrap();

        let sent_data = sender.get_sent_data().await;
        assert_eq!(sent_data.len(), 2);

        // First strip (WS2812B): starts at LED 10. Byte offset = 10 * 3 = 30.
        assert_eq!(sent_data[0].0, 30);

        // Second strip (WS2812B): starts at LED 11. Byte offset = 11 * 3 = 33.
        assert_eq!(sent_data[1].0, 33);
    }

    // Helper function to provide a default LedStripConfig
    impl Default for LedStripConfig {
        fn default() -> Self {
            Self {
                index: 0,
                border: Border::Top,
                display_id: 0,
                len: 0,
                led_type: LedType::WS2812B,
                reversed: false,
            }
        }
    }
}
