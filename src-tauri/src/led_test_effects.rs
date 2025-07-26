use crate::led_data_sender::LedDataSender;
use crate::led_color::LedColor;
use crate::ambient_light::LedType; // 使用统一的LedType
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestEffectType {
    FlowingRainbow,
    GroupCounting,
    SingleScan,
    Breathing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEffectConfig {
    pub effect_type: TestEffectType,
    pub led_count: u32,
    pub led_type: LedType,
    pub speed: f64,  // Speed multiplier
    pub offset: u32, // LED offset
}



/// LED测试效果任务信息
#[derive(Debug, Clone)]
pub struct TestEffectTask {
    pub board_address: String,
    pub config: TestEffectConfig,
    pub update_interval_ms: u32,
    pub start_time: Instant,
}

/// LED测试效果管理器
pub struct LedTestEffectManager {
    /// 活跃的测试效果任务
    active_tasks: Arc<RwLock<HashMap<String, TestEffectTask>>>,
}

impl LedTestEffectManager {
    /// 创建新的测试效果管理器
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取全局单例实例
    pub async fn global() -> &'static LedTestEffectManager {
        static INSTANCE: OnceCell<LedTestEffectManager> = OnceCell::const_new();
        INSTANCE
            .get_or_init(|| async { LedTestEffectManager::new() })
            .await
    }

    /// 启动LED测试效果
    pub async fn start_test_effect(
        &self,
        board_address: String,
        config: TestEffectConfig,
        update_interval_ms: u32,
    ) -> anyhow::Result<()> {
        log::info!(
            "🚀 Starting LED test effect for board: {}, effect: {:?}",
            board_address,
            config.effect_type
        );

        // 如果已有相同设备的任务在运行，先停止它
        self.stop_test_effect(&board_address).await?;

        // 创建新任务
        let task = TestEffectTask {
            board_address: board_address.clone(),
            config: config.clone(),
            update_interval_ms,
            start_time: Instant::now(),
        };

        // 添加到活跃任务列表
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(board_address.clone(), task);
        }

        // 启动后台任务
        let manager = self.clone();
        let task_board_address = board_address.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.run_test_effect_loop(task_board_address).await {
                log::error!("❌ Test effect loop failed: {e}");
            }
        });

        log::info!("✅ LED test effect started for board: {board_address}");
        Ok(())
    }

    /// 停止LED测试效果
    pub async fn stop_test_effect(&self, board_address: &str) -> anyhow::Result<()> {
        log::info!("🛑 Stopping LED test effect for board: {board_address}");

        let mut tasks = self.active_tasks.write().await;
        if let Some(_task) = tasks.remove(board_address) {
            log::info!("✅ LED test effect stopped for board: {board_address}");

            // 发送全黑数据来清除LED
            self.send_clear_data(board_address, &_task.config).await?;
        } else {
            log::warn!("⚠️ No active test effect found for board: {board_address}");
        }

        Ok(())
    }

    /// 停止所有测试效果
    pub async fn stop_all_test_effects(&self) -> anyhow::Result<()> {
        log::info!("🛑 Stopping all LED test effects");

        let board_addresses: Vec<String> = {
            let tasks = self.active_tasks.read().await;
            tasks.keys().cloned().collect()
        };

        for board_address in board_addresses {
            self.stop_test_effect(&board_address).await?;
        }

        log::info!("✅ All LED test effects stopped");
        Ok(())
    }

    /// 获取活跃任务列表
    pub async fn get_active_tasks(&self) -> Vec<String> {
        let tasks = self.active_tasks.read().await;
        tasks.keys().cloned().collect()
    }

    /// 运行测试效果循环
    async fn run_test_effect_loop(&self, board_address: String) -> anyhow::Result<()> {
        log::info!("🔄 Starting test effect loop for board: {board_address}");

        loop {
            // 检查任务是否还存在
            let task = {
                let tasks = self.active_tasks.read().await;
                tasks.get(&board_address).cloned()
            };

            let task = match task {
                Some(task) => task,
                None => {
                    log::info!("🏁 Test effect task removed for board: {board_address}");
                    break;
                }
            };

            // 计算当前时间
            let elapsed_ms = task.start_time.elapsed().as_millis() as u64;

            // 生成LED颜色数据
            let colors = LedTestEffects::generate_colors(&task.config, elapsed_ms);

            // 计算字节偏移量
            let byte_offset = LedTestEffects::calculate_byte_offset(&task.config);

            // 发送数据到硬件
            if let Err(e) = self
                .send_test_data(&board_address, byte_offset, colors)
                .await
            {
                log::error!("❌ Failed to send test data to {board_address}: {e}");
                // 继续运行，不因为单次发送失败而停止
            }

            // 等待下一次更新
            tokio::time::sleep(Duration::from_millis(task.update_interval_ms as u64)).await;
        }

        log::info!("✅ Test effect loop ended for board: {board_address}");
        Ok(())
    }

    /// 发送测试数据到硬件
    async fn send_test_data(
        &self,
        board_address: &str,
        offset: u16,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        // 获取任务配置以确定LED类型和数量
        let task_config = {
            let tasks = self.active_tasks.read().await;
            tasks.get(board_address).map(|task| task.config.clone())
        };

        let config = match task_config {
            Some(config) => config,
            None => {
                log::warn!("⚠️ No active task found for board: {board_address}");
                return Ok(());
            }
        };

        // 将硬件数据转换为RGB颜色数组用于预览
        let rgb_colors = LedTestEffects::hardware_data_to_rgb_colors(&data, &config.led_type);

        // 使用LED数据处理器来发布预览数据并编码硬件数据
        let hardware_data = crate::led_data_processor::LedDataProcessor::process_test_mode(
            rgb_colors,
            config.led_type,
            config.led_count as usize,
            crate::led_data_sender::DataSendMode::TestEffect,
        )
        .await?;

        // 获取LED数据发送器
        let sender = crate::led_data_sender::LedDataSender::global().await;

        // 设置为测试效果模式
        sender
            .set_mode(crate::led_data_sender::DataSendMode::TestEffect)
            .await;

        // 设置目标设备
        sender
            .set_test_target(Some(board_address.to_string()))
            .await;

        // 发送处理后的硬件数据
        sender
            .send_complete_led_data(offset, hardware_data, "TestEffect")
            .await?;

        Ok(())
    }

    /// 发送清除数据（全黑）
    async fn send_clear_data(
        &self,
        board_address: &str,
        config: &TestEffectConfig,
    ) -> anyhow::Result<()> {
        let bytes_per_led = if LedTestEffects::is_rgbw_type(&config.led_type) {
            4
        } else {
            3
        };
        let clear_data = vec![0u8; (config.led_count * bytes_per_led) as usize];
        let byte_offset = LedTestEffects::calculate_byte_offset(config);

        self.send_test_data(board_address, byte_offset, clear_data)
            .await
    }
}

impl Clone for LedTestEffectManager {
    fn clone(&self) -> Self {
        Self {
            active_tasks: self.active_tasks.clone(),
        }
    }
}

pub struct LedTestEffects;

impl LedTestEffects {
    /// Check if LED type supports white channel (RGBW)
    fn is_rgbw_type(led_type: &LedType) -> bool {
        matches!(led_type, LedType::SK6812)
    }

    /// Convert RGB buffer to GRB for WS2812B
    fn convert_rgb_to_grb(buffer: &mut [u8]) {
        let bytes_per_led = 3; // RGB only
        for i in (0..buffer.len()).step_by(bytes_per_led) {
            if i + 2 < buffer.len() {
                // Swap R and G: [R, G, B] -> [G, R, B]
                buffer.swap(i, i + 1);
            }
        }
    }

    /// Convert RGBW buffer to GRBW for SK6812-RGBW
    fn convert_rgbw_to_grbw(buffer: &mut [u8]) {
        let bytes_per_led = 4; // RGBW
        for i in (0..buffer.len()).step_by(bytes_per_led) {
            if i + 3 < buffer.len() {
                // Swap R and G: [R, G, B, W] -> [G, R, B, W]
                buffer.swap(i, i + 1);
            }
        }
    }

    /// 将硬件数据转换为RGB颜色数组（用于预览）
    fn hardware_data_to_rgb_colors(data: &[u8], led_type: &LedType) -> Vec<crate::led_color::LedColor> {
        let mut rgb_colors = Vec::new();

        let bytes_per_led = match led_type {
            LedType::WS2812B => 3, // RGB
            LedType::SK6812 => 4,  // RGBW
        };

        let mut i = 0;
        while i + bytes_per_led <= data.len() {
            match led_type {
                LedType::WS2812B => {
                    // GRB格式 -> RGB
                    let g = data[i];
                    let r = data[i + 1];
                    let b = data[i + 2];
                    rgb_colors.push(crate::led_color::LedColor::new(r, g, b));
                }
                LedType::SK6812 => {
                    // GRBW格式 -> RGB（忽略W通道）
                    let g = data[i];
                    let r = data[i + 1];
                    let b = data[i + 2];
                    // 忽略W通道 data[i + 3]
                    rgb_colors.push(crate::led_color::LedColor::new(r, g, b));
                }
            }
            i += bytes_per_led;
        }

        rgb_colors
    }
    /// Generate LED colors for a specific test effect at a given time
    pub fn generate_colors(config: &TestEffectConfig, time_ms: u64) -> Vec<u8> {
        let time_seconds = time_ms as f64 / 1000.0;

        let mut buffer = match config.effect_type {
            TestEffectType::FlowingRainbow => Self::flowing_rainbow(
                config.led_count,
                config.led_type.clone(),
                time_seconds,
                config.speed,
            ),
            TestEffectType::GroupCounting => {
                Self::group_counting(config.led_count, config.led_type.clone())
            }
            TestEffectType::SingleScan => Self::single_scan(
                config.led_count,
                config.led_type.clone(),
                time_seconds,
                config.speed,
            ),
            TestEffectType::Breathing => Self::breathing(
                config.led_count,
                config.led_type.clone(),
                time_seconds,
                config.speed,
            ),
        };

        // Convert RGB to correct color order based on LED type
        match config.led_type {
            LedType::WS2812B => {
                Self::convert_rgb_to_grb(&mut buffer);
            }
            LedType::SK6812 => {
                Self::convert_rgbw_to_grbw(&mut buffer);
            }
        }

        buffer
    }

    /// Calculate byte offset for 0x02 packet based on LED offset and LED type
    pub fn calculate_byte_offset(config: &TestEffectConfig) -> u16 {
        let bytes_per_led = if Self::is_rgbw_type(&config.led_type) {
            4
        } else {
            3
        };
        (config.offset * bytes_per_led) as u16
    }

    /// Flowing rainbow effect - smooth rainbow colors flowing along the strip
    fn flowing_rainbow(led_count: u32, led_type: LedType, time: f64, speed: f64) -> Vec<u8> {
        let mut buffer = Vec::new();
        let time_offset = (time * speed * 60.0) % 360.0; // 60 degrees per second at speed 1.0

        for i in 0..led_count {
            // Create longer wavelength for smoother color transitions
            let hue = ((i as f64 * 720.0 / led_count as f64) + time_offset) % 360.0;
            let rgb = Self::hsv_to_rgb(hue, 1.0, 1.0);

            buffer.push(rgb.0);
            buffer.push(rgb.1);
            buffer.push(rgb.2);

            if Self::is_rgbw_type(&led_type) {
                buffer.push(0); // White channel - 不点亮白色通道
            }
        }

        buffer
    }

    /// Group counting effect - every 10 LEDs have different colors
    fn group_counting(led_count: u32, led_type: LedType) -> Vec<u8> {
        let mut buffer = Vec::new();

        let group_colors = [
            (255, 0, 0),     // Red (1-10)
            (0, 255, 0),     // Green (11-20)
            (0, 0, 255),     // Blue (21-30)
            (255, 255, 0),   // Yellow (31-40)
            (255, 0, 255),   // Magenta (41-50)
            (0, 255, 255),   // Cyan (51-60)
            (255, 128, 0),   // Orange (61-70)
            (128, 255, 0),   // Lime (71-80)
            (255, 255, 255), // White (81-90)
            (128, 128, 128), // Gray (91-100)
        ];

        for i in 0..led_count {
            let group_index = (i / 10) % group_colors.len() as u32;
            let color = group_colors[group_index as usize];

            buffer.push(color.0);
            buffer.push(color.1);
            buffer.push(color.2);

            if Self::is_rgbw_type(&led_type) {
                buffer.push(0); // White channel - 不点亮白色通道
            }
        }

        buffer
    }

    /// Single LED scan effect - one LED moves along the strip
    fn single_scan(led_count: u32, led_type: LedType, time: f64, speed: f64) -> Vec<u8> {
        let mut buffer = Vec::new();
        let scan_period = 2.0 / speed; // 2 seconds per full scan at speed 1.0
        let active_index = ((time / scan_period * led_count as f64) as u32) % led_count;

        for i in 0..led_count {
            if i == active_index {
                // Bright white LED
                buffer.push(255);
                buffer.push(255);
                buffer.push(255);

                if Self::is_rgbw_type(&led_type) {
                    buffer.push(0); // White channel - 不点亮白色通道
                }
            } else {
                // Off
                buffer.push(0);
                buffer.push(0);
                buffer.push(0);

                if Self::is_rgbw_type(&led_type) {
                    buffer.push(0); // White channel - 不点亮白色通道
                }
            }
        }

        buffer
    }

    /// Breathing effect - entire strip breathes with white light
    fn breathing(led_count: u32, led_type: LedType, time: f64, speed: f64) -> Vec<u8> {
        let mut buffer = Vec::new();
        let breathing_period = 4.0 / speed; // 4 seconds per breath at speed 1.0
        let brightness = ((time / breathing_period * 2.0 * PI).sin() * 0.5 + 0.5) * 255.0;
        let brightness = brightness as u8;

        for _i in 0..led_count {
            buffer.push(brightness);
            buffer.push(brightness);
            buffer.push(brightness);

            if Self::is_rgbw_type(&led_type) {
                buffer.push(brightness); // White channel
            }
        }

        buffer
    }

    /// Convert HSV to RGB
    /// H: 0-360, S: 0-1, V: 0-1
    /// Returns: (R, G, B) where each component is 0-255
    fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0).round() as u8;
        let g = ((g_prime + m) * 255.0).round() as u8;
        let b = ((b_prime + m) * 255.0).round() as u8;

        (r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsv_to_rgb() {
        // Test red
        let (r, g, b) = LedTestEffects::hsv_to_rgb(0.0, 1.0, 1.0);
        assert_eq!((r, g, b), (255, 0, 0));

        // Test green
        let (r, g, b) = LedTestEffects::hsv_to_rgb(120.0, 1.0, 1.0);
        assert_eq!((r, g, b), (0, 255, 0));

        // Test blue
        let (r, g, b) = LedTestEffects::hsv_to_rgb(240.0, 1.0, 1.0);
        assert_eq!((r, g, b), (0, 0, 255));
    }

    #[test]
    fn test_flowing_rainbow() {
        let config = TestEffectConfig {
            effect_type: TestEffectType::FlowingRainbow,
            led_count: 10,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 0,
        };

        let colors_data = LedTestEffects::generate_colors(&config, 0);
        assert_eq!(colors_data.len(), 30); // 10 LEDs * 3 bytes per LED = 30 bytes

        // Convert hardware data back to RGB colors for verification
        let rgb_colors = LedTestEffects::hardware_data_to_rgb_colors(&colors_data, &config.led_type);
        assert_eq!(rgb_colors.len(), 10); // 10 LEDs
    }

    #[test]
    fn test_calculate_byte_offset() {
        // Test WS2812B (3 bytes per LED)
        let config_ws2812b = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 60,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 10, // 10 LEDs offset
        };

        let byte_offset_ws2812b = LedTestEffects::calculate_byte_offset(&config_ws2812b);
        assert_eq!(byte_offset_ws2812b, 30); // 10 LEDs * 3 bytes = 30 bytes

        // Test SK6812 (4 bytes per LED)
        let config_sk6812 = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 60,
            led_type: LedType::SK6812,
            speed: 1.0,
            offset: 10, // 10 LEDs offset
        };

        let byte_offset_sk6812 = LedTestEffects::calculate_byte_offset(&config_sk6812);
        assert_eq!(byte_offset_sk6812, 40); // 10 LEDs * 4 bytes = 40 bytes

        // Test zero offset
        let config_zero_offset = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 60,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 0,
        };

        let byte_offset_zero = LedTestEffects::calculate_byte_offset(&config_zero_offset);
        assert_eq!(byte_offset_zero, 0); // 0 LEDs * 3 bytes = 0 bytes
    }

    #[test]
    fn test_group_counting() {
        let config = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 20,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 0,
        };

        let colors_data = LedTestEffects::generate_colors(&config, 0);
        assert_eq!(colors_data.len(), 60); // 20 LEDs * 3 bytes per LED = 60 bytes

        // Convert hardware data back to RGB colors for testing
        let rgb_colors = LedTestEffects::hardware_data_to_rgb_colors(&colors_data, &config.led_type);
        assert_eq!(rgb_colors.len(), 20); // 20 LEDs

        // First 10 should be red
        let first_color = rgb_colors[0].get_rgb();
        assert_eq!(first_color, [255, 0, 0]); // RGB: Red

        // Next 10 should be green
        let tenth_color = rgb_colors[10].get_rgb();
        assert_eq!(tenth_color, [0, 255, 0]); // RGB: Green
    }
}
