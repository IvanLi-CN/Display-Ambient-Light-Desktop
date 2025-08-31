use anyhow::Result;
use log::{debug, warn};

use crate::{
    ambient_light::{Border, ColorCalibration, LedStripConfig, LedStripConfigV2, LedType},
    display::DisplayRegistry,
    led_color::LedColor,
    led_data_sender::DataSendMode,
    websocket_events::WebSocketEventPublisher,
};

/// LED数据处理器
///
/// 负责统一处理所有模式的LED数据：
/// 1. 发布预览数据（不受颜色校准影响）
/// 2. 硬件编码（应用颜色校准）
/// 3. 返回硬件数据
pub struct LedDataProcessor;

impl LedDataProcessor {
    /// 标准流程：处理二维RGB颜色数据，发布预览，硬件编码
    ///
    /// # 参数
    /// * `led_colors` - 二维颜色数组，外层按strips排序，内层为每个LED的颜色
    /// * `strips` - LED配置数组（必填）
    /// * `color_calibration` - 颜色校准配置（None时使用当前配置）
    /// * `mode` - 当前数据发送模式
    /// * `start_led_offset` - LED偏移量（必填）
    ///
    /// # 返回值
    /// 返回硬件编码后的数据，可直接发送给LED硬件
    pub async fn process_and_publish(
        led_colors: Vec<Vec<LedColor>>,
        strips: &[LedStripConfig],
        color_calibration: Option<&ColorCalibration>,
        _mode: DataSendMode,
        start_led_offset: usize,
    ) -> Result<Vec<u8>> {
        // 1. 获取颜色校准配置
        let calibration = match color_calibration {
            Some(cal) => *cal,
            None => Self::get_current_color_calibration().await?,
        };

        // 2. 转换为预览数据（一维RGB字节数组，无校准）
        let preview_rgb_bytes = Self::colors_2d_to_rgb_bytes(&led_colors);

        // 3. 发布预览数据（避免不必要的clone）
        let websocket_publisher = WebSocketEventPublisher::global().await;
        // 移除旧的 LedColorsChanged 事件，使用按物理顺序排列的颜色事件和按灯带分组的事件替代
        websocket_publisher
            .publish_led_sorted_colors_changed(&preview_rgb_bytes, start_led_offset)
            .await;

        // 记录数据发送事件到频率计算器
        let status_manager = crate::led_status_manager::LedStatusManager::global().await;
        if let Err(e) = status_manager.record_data_send_event().await {
            log::warn!("Failed to record data send event: {e}");
        }

        // 3.1. 按灯带分组发布（替代旧的 LedColorsChanged 事件）
        Self::publish_led_strip_colors(&led_colors, strips, websocket_publisher).await;

        // 4. 硬件编码（应用颜色校准）
        let hardware_data =
            Self::encode_for_hardware(led_colors, strips, &calibration, start_led_offset)?;

        Ok(hardware_data)
    }

    /// V2配置版本：处理二维RGB颜色数据，发布预览，硬件编码
    ///
    /// # 参数
    /// * `led_colors` - 二维颜色数组，外层按strips排序，内层为每个LED的颜色
    /// * `strips` - V2 LED配置数组（必填）
    /// * `display_registry` - 显示器注册表，用于ID转换
    /// * `color_calibration` - 颜色校准配置（None时使用当前配置）
    /// * `mode` - 当前数据发送模式
    /// * `start_led_offset` - LED偏移量（必填）
    ///
    /// # 返回值
    /// 返回硬件编码后的数据，可直接发送给LED硬件
    pub async fn process_and_publish_v2(
        led_colors: Vec<Vec<LedColor>>,
        strips: &[LedStripConfigV2],
        display_registry: &DisplayRegistry,
        color_calibration: Option<&ColorCalibration>,
        _mode: DataSendMode,
        start_led_offset: usize,
    ) -> Result<Vec<u8>> {
        // 1. 获取颜色校准配置
        let calibration = match color_calibration {
            Some(cal) => *cal,
            None => Self::get_current_color_calibration().await?,
        };

        // 2. 转换为预览数据（一维RGB字节数组，无校准）
        let preview_rgb_bytes = Self::colors_2d_to_rgb_bytes(&led_colors);

        // 3. 发布预览数据（避免不必要的clone）
        let websocket_publisher = WebSocketEventPublisher::global().await;
        // 移除旧的 LedColorsChanged 事件，使用按物理顺序排列的颜色事件和按灯带分组的事件替代
        websocket_publisher
            .publish_led_sorted_colors_changed(&preview_rgb_bytes, start_led_offset)
            .await;

        // 记录数据发送事件到频率计算器
        let status_manager = crate::led_status_manager::LedStatusManager::global().await;
        if let Err(e) = status_manager.record_data_send_event().await {
            log::warn!("Failed to record data send event: {e}");
        }

        // 3.1. 按灯带分组发布（替代旧的 LedColorsChanged 事件）- V2版本
        Self::publish_led_strip_colors_v2(
            &led_colors,
            strips,
            display_registry,
            websocket_publisher,
        )
        .await;

        // 4. 硬件编码（应用颜色校准）- V2版本
        let hardware_data =
            Self::encode_for_hardware_v2(led_colors, strips, &calibration, start_led_offset)?;

        Ok(hardware_data)
    }

    /// 测试模式专用：发布预览后按指定LED类型编码
    ///
    /// # 参数
    /// * `rgb_colors` - 一维测试效果RGB数据
    /// * `led_type` - 强制指定的LED类型
    /// * `led_count` - LED数量
    /// * `mode` - 当前数据发送模式
    ///
    /// # 返回值
    /// 返回硬件编码后的数据，可直接发送给LED硬件
    pub async fn process_test_mode(
        rgb_colors: Vec<LedColor>,
        led_type: LedType,
        led_count: usize,
        mode: DataSendMode,
    ) -> Result<Vec<u8>> {
        debug!(
            "🧪 LedDataProcessor::process_test_mode - led_type: {led_type:?}, count: {led_count}, mode: {mode:?}"
        );

        // 1. 转换为预览数据（一维RGB字节数组）
        let preview_rgb_bytes = Self::colors_1d_to_rgb_bytes(&rgb_colors);
        debug!(
            "📊 Generated test preview data: {} bytes",
            preview_rgb_bytes.len()
        );

        // 2. 发布预览数据
        let websocket_publisher = WebSocketEventPublisher::global().await;
        // 移除旧的 LedColorsChanged 事件，测试模式使用按物理顺序排列的颜色事件
        websocket_publisher
            .publish_led_sorted_colors_changed(&preview_rgb_bytes, 0) // 测试模式偏移量为0
            .await;

        // 记录数据发送事件到频率计算器
        let status_manager = crate::led_status_manager::LedStatusManager::global().await;
        if let Err(e) = status_manager.record_data_send_event().await {
            log::warn!("Failed to record data send event: {e}");
        }

        debug!("✅ Test LED preview data published successfully");

        // 3. 测试模式编码（无校准）
        let hardware_data = Self::encode_for_test_mode(rgb_colors, led_type, led_count)?;

        debug!(
            "🧪 Test mode encoding completed: {} bytes",
            hardware_data.len()
        );
        Ok(hardware_data)
    }

    /// 辅助方法：二维颜色数组转一维RGB字节数组（用于预览）
    ///
    /// 将二维颜色数组按顺序展开为RGB字节序列，不应用颜色校准
    fn colors_2d_to_rgb_bytes(led_colors: &[Vec<LedColor>]) -> Vec<u8> {
        led_colors
            .iter()
            .flat_map(|strip_colors| {
                strip_colors.iter().flat_map(|color| {
                    let rgb = color.get_rgb();
                    [rgb[0], rgb[1], rgb[2]] // 原始RGB，无校准
                })
            })
            .collect()
    }

    /// 辅助方法：一维颜色数组转RGB字节数组（用于测试模式预览）
    ///
    /// 将一维颜色数组转换为RGB字节序列，不应用颜色校准
    fn colors_1d_to_rgb_bytes(colors: &[LedColor]) -> Vec<u8> {
        colors
            .iter()
            .flat_map(|color| {
                let rgb = color.get_rgb();
                [rgb[0], rgb[1], rgb[2]] // 原始RGB，无校准
            })
            .collect()
    }

    /// 核心方法：硬件编码（从 send_colors_by_display 移动过来）
    ///
    /// 将二维颜色数组按strips配置编码为硬件数据，应用颜色校准
    ///
    /// # 参数
    /// * `led_colors` - 二维颜色数组，外层按strips排序
    /// * `strips` - LED配置数组
    /// * `color_calibration` - 颜色校准配置
    /// * `start_led_offset` - LED偏移量
    ///
    /// # 返回值
    /// 返回硬件编码后的数据（GRB/GRBW格式）
    fn encode_for_hardware(
        led_colors: Vec<Vec<LedColor>>,
        strips: &[LedStripConfig],
        color_calibration: &ColorCalibration,
        start_led_offset: usize,
    ) -> Result<Vec<u8>> {
        debug!(
            "🔧 Encoding for hardware: {} strips, offset: {}",
            strips.len(),
            start_led_offset
        );

        // 按序列号排序灯带，确保正确的串联顺序
        let mut sorted_strips: Vec<_> = strips.iter().enumerate().collect();
        sorted_strips.sort_by_key(|(_, strip)| strip.index);

        debug!(
            "排序后的灯带顺序: {:?}",
            sorted_strips
                .iter()
                .map(|(_, s)| (s.index, s.border, s.display_id))
                .collect::<Vec<_>>()
        );

        // 预计算总字节数以预分配缓冲区，减少内存重分配
        let total_bytes: usize = sorted_strips
            .iter()
            .map(|(_, strip)| {
                let bytes_per_led = match strip.led_type {
                    LedType::WS2812B => 3,
                    LedType::SK6812 => 4,
                };
                strip.len * bytes_per_led
            })
            .sum();

        let mut complete_led_data = Vec::<u8>::with_capacity(total_bytes);
        let mut total_leds = 0;

        for (strip_index, strip) in sorted_strips {
            let strip_len = strip.len;

            debug!(
                "编码LED灯带 {}: border={:?}, len={}, led_type={:?}",
                strip_index, strip.border, strip_len, strip.led_type
            );

            // 检查二维数组索引是否有效
            if strip_index >= led_colors.len() {
                warn!(
                    "跳过灯带 {}: 索引超出颜色数组范围 ({})",
                    strip_index,
                    led_colors.len()
                );
                // 添加黑色作为后备
                for _ in 0..strip_len {
                    match strip.led_type {
                        LedType::WS2812B => complete_led_data.extend_from_slice(&[0, 0, 0]),
                        LedType::SK6812 => complete_led_data.extend_from_slice(&[0, 0, 0, 0]),
                    }
                }
                total_leds += strip_len;
                continue;
            }

            let strip_colors = &led_colors[strip_index];

            // 将这个灯带的数据添加到完整数据流中
            for i in 0..strip_len {
                if i < strip_colors.len() {
                    let color = strip_colors[i];
                    let rgb = color.get_rgb();

                    // 应用颜色校准
                    let calibrated_r = (rgb[0] as f32 * color_calibration.r) as u8;
                    let calibrated_g = (rgb[1] as f32 * color_calibration.g) as u8;
                    let calibrated_b = (rgb[2] as f32 * color_calibration.b) as u8;

                    match strip.led_type {
                        LedType::WS2812B => {
                            // GRB格式
                            complete_led_data.extend_from_slice(&[
                                calibrated_g, // G (Green)
                                calibrated_r, // R (Red)
                                calibrated_b, // B (Blue)
                            ]);
                        }
                        LedType::SK6812 => {
                            // GRBW格式，W通道单独校准
                            let w_channel = Self::calculate_white_channel(
                                calibrated_r,
                                calibrated_g,
                                calibrated_b,
                            );
                            let calibrated_w = (w_channel as f32 * color_calibration.w) as u8;
                            complete_led_data.extend_from_slice(&[
                                calibrated_g, // G (Green)
                                calibrated_r, // R (Red)
                                calibrated_b, // B (Blue)
                                calibrated_w, // W (White)
                            ]);
                        }
                    }
                } else {
                    warn!(
                        "LED索引 {} 超出灯带颜色数组范围 ({})",
                        i,
                        strip_colors.len()
                    );
                    // 添加黑色作为后备
                    match strip.led_type {
                        LedType::WS2812B => complete_led_data.extend_from_slice(&[0, 0, 0]),
                        LedType::SK6812 => complete_led_data.extend_from_slice(&[0, 0, 0, 0]),
                    }
                }
            }

            total_leds += strip_len;
        }

        debug!(
            "✅ 硬件编码完成: {} LEDs -> {} bytes",
            total_leds,
            complete_led_data.len()
        );

        Ok(complete_led_data)
    }

    /// 测试模式编码：按指定LED类型编码（无校准）
    ///
    /// 将一维颜色数组按指定LED类型编码，不应用颜色校准
    ///
    /// # 参数
    /// * `rgb_colors` - 一维颜色数组
    /// * `led_type` - 强制指定的LED类型
    /// * `led_count` - LED数量
    ///
    /// # 返回值
    /// 返回硬件编码后的数据（GRB/GRBW格式，无校准）
    fn encode_for_test_mode(
        rgb_colors: Vec<LedColor>,
        led_type: LedType,
        led_count: usize,
    ) -> Result<Vec<u8>> {
        debug!("🧪 Encoding for test mode: type={led_type:?}, count={led_count}");

        // 预分配缓冲区大小，减少内存重分配
        let bytes_per_led = match led_type {
            LedType::WS2812B => 3,
            LedType::SK6812 => 4,
        };
        let mut buffer = Vec::with_capacity(led_count * bytes_per_led);

        let default_color = LedColor::new(0, 0, 0);
        for i in 0..led_count {
            let color = rgb_colors.get(i).unwrap_or(&default_color);
            let rgb = color.get_rgb();

            match led_type {
                LedType::WS2812B => {
                    // GRB格式，无校准
                    buffer.extend_from_slice(&[
                        rgb[1], // G (Green)
                        rgb[0], // R (Red)
                        rgb[2], // B (Blue)
                    ]);
                }
                LedType::SK6812 => {
                    // GRBW格式，无校准，W通道为0
                    buffer.extend_from_slice(&[
                        rgb[1], // G (Green)
                        rgb[0], // R (Red)
                        rgb[2], // B (Blue)
                        0,      // W (White) - 测试模式不使用白色通道
                    ]);
                }
            }
        }

        debug!(
            "✅ 测试模式编码完成: {} LEDs -> {} bytes",
            led_count,
            buffer.len()
        );

        Ok(buffer)
    }

    /// V2版本：硬件编码（支持V2配置格式）
    ///
    /// 将二维颜色数组按V2 strips配置编码为硬件数据，应用颜色校准
    ///
    /// # 参数
    /// * `led_colors` - 二维颜色数组，外层按strips排序
    /// * `strips` - V2 LED配置数组
    /// * `color_calibration` - 颜色校准配置
    /// * `start_led_offset` - LED偏移量
    ///
    /// # 返回值
    /// 返回硬件编码后的数据（GRB/GRBW格式）
    fn encode_for_hardware_v2(
        led_colors: Vec<Vec<LedColor>>,
        strips: &[LedStripConfigV2],
        color_calibration: &ColorCalibration,
        start_led_offset: usize,
    ) -> Result<Vec<u8>> {
        debug!(
            "🔧 Encoding for hardware (V2): {} strips, offset: {}",
            strips.len(),
            start_led_offset
        );

        // 计算总LED数量和每个LED的字节数
        let total_leds: usize = strips.iter().map(|s| s.len).sum();
        let mut complete_led_data = Vec::new();

        // 按strips顺序处理每个灯带
        for (strip_index, strip) in strips.iter().enumerate() {
            let strip_colors = &led_colors[strip_index];

            debug!(
                "🔧 Processing V2 strip {}: len={}, led_type={:?}, display_internal_id={}",
                strip.index, strip.len, strip.led_type, strip.display_internal_id
            );

            // 处理每个LED
            for i in 0..strip.len {
                if i < strip_colors.len() {
                    let color = &strip_colors[i];
                    let rgb = color.get_rgb();

                    // 应用颜色校准
                    let calibrated_r = (rgb[0] as f32 * color_calibration.r) as u8;
                    let calibrated_g = (rgb[1] as f32 * color_calibration.g) as u8;
                    let calibrated_b = (rgb[2] as f32 * color_calibration.b) as u8;

                    match strip.led_type {
                        LedType::WS2812B => {
                            // GRB格式
                            complete_led_data.extend_from_slice(&[
                                calibrated_g, // G (Green)
                                calibrated_r, // R (Red)
                                calibrated_b, // B (Blue)
                            ]);
                        }
                        LedType::SK6812 => {
                            // GRBW格式，W通道单独校准
                            let w_channel = Self::calculate_white_channel(
                                calibrated_r,
                                calibrated_g,
                                calibrated_b,
                            );
                            let calibrated_w = (w_channel as f32 * color_calibration.w) as u8;
                            complete_led_data.extend_from_slice(&[
                                calibrated_g, // G (Green)
                                calibrated_r, // R (Red)
                                calibrated_b, // B (Blue)
                                calibrated_w, // W (White)
                            ]);
                        }
                    }
                } else {
                    warn!(
                        "LED索引 {} 超出V2灯带颜色数组范围 ({})",
                        i,
                        strip_colors.len()
                    );
                    // 填充黑色
                    match strip.led_type {
                        LedType::WS2812B => {
                            complete_led_data.extend_from_slice(&[0, 0, 0]);
                        }
                        LedType::SK6812 => {
                            complete_led_data.extend_from_slice(&[0, 0, 0, 0]);
                        }
                    }
                }
            }
        }

        debug!(
            "✅ V2硬件编码完成: {} LEDs -> {} bytes",
            total_leds,
            complete_led_data.len()
        );

        Ok(complete_led_data)
    }

    /// 计算SK6812的白色通道值
    ///
    /// 基于RGB值计算合适的白色通道值
    fn calculate_white_channel(r: u8, g: u8, b: u8) -> u8 {
        // 使用RGB的最小值作为白色通道的基础
        // 这样可以减少RGB通道的负担，提高亮度效率
        std::cmp::min(std::cmp::min(r, g), b)
    }

    /// 获取当前颜色校准配置
    ///
    /// 从配置管理器获取当前的颜色校准设置
    async fn get_current_color_calibration() -> Result<ColorCalibration> {
        let config_manager = crate::ambient_light::ConfigManager::global().await;
        let configs = config_manager.configs().await;
        Ok(configs.color_calibration)
    }

    /// 按灯带分组发布LED颜色数据
    ///
    /// 为每个灯带单独发布颜色数据，解决多显示器LED预览闪烁问题
    async fn publish_led_strip_colors(
        led_colors: &[Vec<LedColor>],
        strips: &[LedStripConfig],
        websocket_publisher: &WebSocketEventPublisher,
    ) {
        for (strip, colors) in strips.iter().zip(led_colors.iter()) {
            let rgb_bytes: Vec<u8> = colors.iter().flat_map(|color| color.get_rgb()).collect();

            let border_str = match strip.border {
                Border::Top => "Top",
                Border::Bottom => "Bottom",
                Border::Left => "Left",
                Border::Right => "Right",
            };

            websocket_publisher
                .publish_led_strip_colors_changed(
                    strip.display_id,
                    border_str,
                    strip.index,
                    &rgb_bytes,
                )
                .await;
        }
    }

    /// V2版本：按灯带分组发布LED颜色数据
    ///
    /// 为每个V2灯带单独发布颜色数据，解决多显示器LED预览闪烁问题
    async fn publish_led_strip_colors_v2(
        led_colors: &[Vec<LedColor>],
        strips: &[LedStripConfigV2],
        display_registry: &DisplayRegistry,
        websocket_publisher: &WebSocketEventPublisher,
    ) {
        for (strip, colors) in strips.iter().zip(led_colors.iter()) {
            let rgb_bytes: Vec<u8> = colors.iter().flat_map(|color| color.get_rgb()).collect();

            let border_str = match strip.border {
                Border::Top => "Top",
                Border::Bottom => "Bottom",
                Border::Left => "Left",
                Border::Right => "Right",
            };

            // 通过DisplayRegistry将internal_id转换为system_id
            let display_id = match display_registry
                .get_display_id_by_internal_id(&strip.display_internal_id)
                .await
            {
                Ok(id) => {
                    debug!(
                        "✅ V2发布：映射显示器内部ID {} -> 系统ID {}",
                        strip.display_internal_id, id
                    );
                    id
                }
                Err(e) => {
                    warn!(
                        "⚠️ V2发布：无法获取显示器 {} 的系统ID: {}，使用默认值0",
                        strip.display_internal_id, e
                    );
                    0
                }
            };

            websocket_publisher
                .publish_led_strip_colors_changed(display_id, border_str, strip.index, &rgb_bytes)
                .await;
        }
    }
}
