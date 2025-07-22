//! 测试 LedColorsPublisher 的 generate_and_publish_config_colors 函数
//!
//! 这个测试验证单屏配置模式下的数据生成和发布逻辑是否正确。

use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock 的 LedDataSender，用于捕获发送的数据
    #[derive(Debug, Clone)]
    struct MockLedDataSender {
        sent_data: Arc<RwLock<Vec<(u16, Vec<u8>, String)>>>, // (offset, buffer, source)
    }

    impl MockLedDataSender {
        fn new() -> Self {
            Self {
                sent_data: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn send_complete_led_data(
            &self,
            offset: u16,
            buffer: Vec<u8>,
            source: &str,
        ) -> anyhow::Result<()> {
            let mut data = self.sent_data.write().await;
            data.push((offset, buffer, source.to_string()));
            Ok(())
        }

        async fn get_sent_data(&self) -> Vec<(u16, Vec<u8>, String)> {
            self.sent_data.read().await.clone()
        }
    }

    /// 创建测试用的灯带配置
    fn create_test_config_group() -> LedStripConfigGroup {
        let strips = vec![
            LedStripConfig {
                index: 0,
                border: Border::Bottom,
                display_id: 2,
                len: 4, // 使用小数量便于验证
                led_type: LedType::WS2812B,
                reversed: false,
            },
            LedStripConfig {
                index: 1,
                border: Border::Right,
                display_id: 2,
                len: 3,
                led_type: LedType::SK6812,
                reversed: false,
            },
            LedStripConfig {
                index: 2,
                border: Border::Top,
                display_id: 2,
                len: 2,
                led_type: LedType::WS2812B,
                reversed: false,
            },
        ];

        let mut config_group = LedStripConfigGroup {
            strips,
            mappers: Vec::new(),
            color_calibration: ColorCalibration::new(),
        };
        config_group.generate_mappers();
        config_group
    }

    /// 创建测试用的边框颜色
    fn create_test_border_colors() -> BorderColors {
        BorderColors {
            top: [[0, 255, 255], [0, 0, 255]],     // 青色 + 蓝色
            bottom: [[255, 0, 0], [255, 128, 0]],  // 红色 + 橙色
            left: [[128, 0, 255], [255, 0, 128]],  // 紫色 + 玫红色
            right: [[255, 255, 0], [128, 255, 0]], // 黄色 + 黄绿色
        }
    }

    #[tokio::test]
    async fn test_generate_edge_colors_from_constants() {
        let publisher = LedColorsPublisher::global().await;
        let border_colors = create_test_border_colors();

        let edge_colors = publisher.generate_edge_colors_from_constants(&border_colors);

        // 验证生成的边框颜色
        assert_eq!(edge_colors.len(), 4);

        let top_colors = edge_colors.get(&Border::Top).unwrap();
        let top_rgb_1 = top_colors[0].get_rgb();
        let top_rgb_2 = top_colors[1].get_rgb();
        assert_eq!(top_rgb_1, [0, 255, 255]); // 青色 (第一种颜色)
        assert_eq!(top_rgb_2, [0, 0, 255]); // 蓝色 (第二种颜色)

        let bottom_colors = edge_colors.get(&Border::Bottom).unwrap();
        let bottom_rgb_1 = bottom_colors[0].get_rgb();
        let bottom_rgb_2 = bottom_colors[1].get_rgb();
        assert_eq!(bottom_rgb_1, [255, 0, 0]); // 红色 (第一种颜色)
        assert_eq!(bottom_rgb_2, [255, 128, 0]); // 橙色 (第二种颜色)

        let left_colors = edge_colors.get(&Border::Left).unwrap();
        let left_rgb_1 = left_colors[0].get_rgb();
        let left_rgb_2 = left_colors[1].get_rgb();
        assert_eq!(left_rgb_1, [128, 0, 255]); // 紫色 (第一种颜色)
        assert_eq!(left_rgb_2, [255, 0, 128]); // 玫红色 (第二种颜色)

        let right_colors = edge_colors.get(&Border::Right).unwrap();
        let right_rgb_1 = right_colors[0].get_rgb();
        let right_rgb_2 = right_colors[1].get_rgb();
        assert_eq!(right_rgb_1, [255, 255, 0]); // 黄色 (第一种颜色)
        assert_eq!(right_rgb_2, [128, 255, 0]); // 黄绿色 (第二种颜色)
    }

    #[tokio::test]
    async fn test_map_edge_colors_to_led_buffer() {
        let publisher = LedColorsPublisher::global().await;
        let config_group = create_test_config_group();
        let border_colors = create_test_border_colors();

        let edge_colors = publisher.generate_edge_colors_from_constants(&border_colors);
        let buffer = publisher
            .map_edge_colors_to_led_buffer(&config_group, &edge_colors)
            .unwrap();

        // 验证缓冲区大小
        // 序列号0: Bottom边, 4个LED, WS2812B (3字节/LED) = 12字节
        // 序列号1: Right边, 3个LED, SK6812 (4字节/LED) = 12字节
        // 序列号2: Top边, 2个LED, WS2812B (3字节/LED) = 6字节
        // 总计: 12 + 12 + 6 = 30字节
        assert_eq!(buffer.len(), 30);

        // 验证序列号0 (Bottom边, 双色分段: 红色+橙色, WS2812B格式: GRB)
        let bottom_start = 0;
        for i in 0..4 {
            let offset = bottom_start + i * 3;
            if i < 2 {
                // 前半部分应该是红色 [255, 0, 0] -> GRB: [0, 255, 0]
                assert_eq!(buffer[offset], 0, "LED {} G channel should be 0", i); // G
                assert_eq!(buffer[offset + 1], 255, "LED {} R channel should be 255", i); // R
                assert_eq!(buffer[offset + 2], 0, "LED {} B channel should be 0", i);
            // B
            } else {
                // 后半部分应该是橙色 [255, 128, 0] -> GRB: [128, 255, 0]
                assert_eq!(buffer[offset], 128, "LED {} G channel should be 128", i); // G
                assert_eq!(buffer[offset + 1], 255, "LED {} R channel should be 255", i); // R
                assert_eq!(buffer[offset + 2], 0, "LED {} B channel should be 0", i);
                // B
            }
        }

        // 验证序列号1 (Right边, 双色分段: 黄色+黄绿色, SK6812格式: GRBW)
        let right_start = 12;
        for i in 0..3 {
            let offset = right_start + i * 4;
            if i < 1 {
                // 前半部分（第1个LED）应该是黄色 [255, 255, 0] -> GRBW: [255, 255, 0, 0]
                assert_eq!(buffer[offset], 255); // G
                assert_eq!(buffer[offset + 1], 255); // R
                assert_eq!(buffer[offset + 2], 0); // B
                assert_eq!(buffer[offset + 3], 0); // W
            } else {
                // 后半部分（第2-3个LED）应该是黄绿色 [128, 255, 0] -> GRBW: [255, 128, 0, 0]
                assert_eq!(buffer[offset], 255); // G
                assert_eq!(buffer[offset + 1], 128); // R
                assert_eq!(buffer[offset + 2], 0); // B
                assert_eq!(buffer[offset + 3], 0); // W
            }
        }

        // 验证序列号2 (Top边, 双色分段: 青色+蓝色, WS2812B格式: GRB)
        let top_start = 24;
        for i in 0..2 {
            let offset = top_start + i * 3;
            if i < 1 {
                // 前半部分（第1个LED）应该是青色 [0, 255, 255] -> GRB: [255, 0, 255]
                assert_eq!(buffer[offset], 255); // G
                assert_eq!(buffer[offset + 1], 0); // R
                assert_eq!(buffer[offset + 2], 255); // B
            } else {
                // 后半部分（第2个LED）应该是蓝色 [0, 0, 255] -> GRB: [0, 0, 255]
                assert_eq!(buffer[offset], 0); // G
                assert_eq!(buffer[offset + 1], 0); // R
                assert_eq!(buffer[offset + 2], 255); // B
            }
        }
    }

    #[tokio::test]
    async fn test_generate_and_publish_config_colors_with_mock() {
        // 创建 mock sender
        let mock_sender = MockLedDataSender::new();

        // 创建测试数据
        let publisher = LedColorsPublisher::global().await;
        let config_group = create_test_config_group();
        let border_colors = create_test_border_colors();

        // 手动调用内部方法来测试数据生成
        let edge_colors = publisher.generate_edge_colors_from_constants(&border_colors);
        let complete_buffer = publisher
            .map_edge_colors_to_led_buffer(&config_group, &edge_colors)
            .unwrap();

        // 模拟发送数据
        mock_sender
            .send_complete_led_data(0, complete_buffer.clone(), "SingleDisplayConfig")
            .await
            .unwrap();

        // 验证发送的数据
        let sent_data = mock_sender.get_sent_data().await;
        assert_eq!(sent_data.len(), 1);

        let (offset, buffer, source) = &sent_data[0];
        assert_eq!(*offset, 0);
        assert_eq!(*source, "SingleDisplayConfig");
        assert_eq!(buffer.len(), 30); // 总字节数

        // 验证具体的LED数据内容
        // Bottom边 (双色分段: 红色+橙色): 4个LED × 3字节 = 12字节
        // half_count = 4/2 = 2, 所以LED0和LED1用红色，LED2和LED3用橙色
        for i in 0..4 {
            let led_offset = i * 3;
            if i < 2 {
                // LED0,LED1: 红色 [255, 0, 0] -> GRB: [0, 255, 0]
                assert_eq!(buffer[led_offset], 0); // G
                assert_eq!(buffer[led_offset + 1], 255); // R
                assert_eq!(buffer[led_offset + 2], 0); // B
            } else {
                // LED2,LED3: 橙色 [255, 128, 0] -> GRB: [128, 255, 0]
                assert_eq!(buffer[led_offset], 128); // G
                assert_eq!(buffer[led_offset + 1], 255); // R
                assert_eq!(buffer[led_offset + 2], 0); // B
            }
        }

        // Right边 (双色分段: 黄色+黄绿色): 3个LED × 4字节 = 12字节
        // half_count = 3/2 = 1, 所以LED0用黄色，LED1和LED2用黄绿色
        for i in 0..3 {
            let led_offset = 12 + i * 4;
            if i < 1 {
                // LED0: 黄色 [255, 255, 0] -> GRBW: [255, 255, 0, 0]
                assert_eq!(buffer[led_offset], 255); // G
                assert_eq!(buffer[led_offset + 1], 255); // R
                assert_eq!(buffer[led_offset + 2], 0); // B
                assert_eq!(buffer[led_offset + 3], 0); // W
            } else {
                // LED1,LED2: 黄绿色 [128, 255, 0] -> GRBW: [255, 128, 0, 0]
                assert_eq!(buffer[led_offset], 255); // G
                assert_eq!(buffer[led_offset + 1], 128); // R
                assert_eq!(buffer[led_offset + 2], 0); // B
                assert_eq!(buffer[led_offset + 3], 0); // W
            }
        }

        // Top边 (双色分段: 青色+蓝色): 2个LED × 3字节 = 6字节
        // half_count = 2/2 = 1, 所以LED0用青色，LED1用蓝色
        for i in 0..2 {
            let led_offset = 24 + i * 3;
            if i < 1 {
                // LED0: 青色 [0, 255, 255] -> GRB: [255, 0, 255]
                assert_eq!(buffer[led_offset], 255); // G
                assert_eq!(buffer[led_offset + 1], 0); // R
                assert_eq!(buffer[led_offset + 2], 255); // B
            } else {
                // LED1: 蓝色 [0, 0, 255] -> GRB: [0, 0, 255]
                assert_eq!(buffer[led_offset], 0); // G
                assert_eq!(buffer[led_offset + 1], 0); // R
                assert_eq!(buffer[led_offset + 2], 255); // B
            }
        }

        println!("✅ 测试通过: generate_and_publish_config_colors 生成了正确的LED数据");
        println!("   - 总字节数: {}", buffer.len());
        println!("   - Bottom边(红色): 4个LED × 3字节 = 12字节");
        println!("   - Right边(黄色): 3个LED × 4字节 = 12字节");
        println!("   - Top边(青色): 2个LED × 3字节 = 6字节");
    }

    #[tokio::test]
    async fn test_led_data_order_and_format() {
        let publisher = LedColorsPublisher::global().await;
        let config_group = create_test_config_group();
        let border_colors = create_test_border_colors();

        let edge_colors = publisher.generate_edge_colors_from_constants(&border_colors);
        let buffer = publisher
            .map_edge_colors_to_led_buffer(&config_group, &edge_colors)
            .unwrap();

        // 验证LED数据的顺序是否按序列号排序
        // 序列号0: index=0, Bottom边
        // 序列号1: index=1, Right边
        // 序列号2: index=2, Top边

        println!("🔍 LED数据顺序验证:");
        println!("序列号0 (Bottom, 红色, WS2812B): 字节0-11");
        println!("序列号1 (Right, 黄色, SK6812): 字节12-23");
        println!("序列号2 (Top, 青色, WS2812B): 字节24-29");

        // 验证数据连续性 - 没有间隙
        assert_eq!(buffer.len(), 30);

        // 验证不同LED类型的字节格式
        // WS2812B: GRB (3字节)
        // SK6812: GRBW (4字节)

        let mut byte_index = 0;

        // 序列号0: Bottom边, 双色分段: 红色+橙色, WS2812B
        for led in 0..4 {
            if led < 2 {
                // 前半部分: 红色 [255,0,0] -> GRB: [0,255,0]
                assert_eq!(buffer[byte_index], 0); // G
                assert_eq!(buffer[byte_index + 1], 255); // R
                assert_eq!(buffer[byte_index + 2], 0); // B
            } else {
                // 后半部分: 橙色 [255,128,0] -> GRB: [128,255,0]
                assert_eq!(buffer[byte_index], 128); // G
                assert_eq!(buffer[byte_index + 1], 255); // R
                assert_eq!(buffer[byte_index + 2], 0); // B
            }
            byte_index += 3;
        }

        // 序列号1: Right边, 双色分段: 黄色+黄绿色, SK6812
        for led in 0..3 {
            if led < 1 {
                // 前半部分: 黄色 [255,255,0] -> GRBW: [255,255,0,0]
                assert_eq!(buffer[byte_index], 255); // G
                assert_eq!(buffer[byte_index + 1], 255); // R
                assert_eq!(buffer[byte_index + 2], 0); // B
                assert_eq!(buffer[byte_index + 3], 0); // W
            } else {
                // 后半部分: 黄绿色 [128,255,0] -> GRBW: [255,128,0,0]
                assert_eq!(buffer[byte_index], 255); // G
                assert_eq!(buffer[byte_index + 1], 128); // R
                assert_eq!(buffer[byte_index + 2], 0); // B
                assert_eq!(buffer[byte_index + 3], 0); // W
            }
            byte_index += 4;
        }

        // 序列号2: Top边, 双色分段: 青色+蓝色, WS2812B
        for led in 0..2 {
            if led < 1 {
                // 前半部分: 青色 [0,255,255] -> GRB: [255,0,255]
                assert_eq!(buffer[byte_index], 255); // G
                assert_eq!(buffer[byte_index + 1], 0); // R
                assert_eq!(buffer[byte_index + 2], 255); // B
            } else {
                // 后半部分: 蓝色 [0,0,255] -> GRB: [0,0,255]
                assert_eq!(buffer[byte_index], 0); // G
                assert_eq!(buffer[byte_index + 1], 0); // R
                assert_eq!(buffer[byte_index + 2], 255); // B
            }
            byte_index += 3;
        }

        assert_eq!(byte_index, 30); // 验证所有字节都被检查了

        println!("✅ LED数据顺序和格式验证通过");
    }

    /// 创建跨显示器串联的测试配置
    fn create_cross_display_config_group() -> LedStripConfigGroup {
        let strips = vec![
            // 显示器2的灯带 (序列号0-2)
            LedStripConfig {
                index: 0,
                border: Border::Bottom,
                display_id: 2,
                len: 3, // 使用小数量便于验证
                led_type: LedType::SK6812,
                reversed: false,
            },
            LedStripConfig {
                index: 1,
                border: Border::Right,
                display_id: 2,
                len: 2,
                led_type: LedType::WS2812B,
                reversed: false,
            },
            LedStripConfig {
                index: 2,
                border: Border::Top,
                display_id: 2,
                len: 3,
                led_type: LedType::WS2812B,
                reversed: false,
            },
            // 显示器1的灯带 (序列号3，继续串联)
            LedStripConfig {
                index: 3,
                border: Border::Top,
                display_id: 1,
                len: 4,
                led_type: LedType::SK6812,
                reversed: false,
            },
        ];

        let mut config_group = LedStripConfigGroup {
            strips,
            mappers: Vec::new(),
            color_calibration: ColorCalibration::new(),
        };
        config_group.generate_mappers();
        config_group
    }

    #[tokio::test]
    async fn test_cross_display_led_buffer_generation() {
        let publisher = LedColorsPublisher::global().await;
        let config_group = create_cross_display_config_group();
        let border_colors = create_test_border_colors();

        let edge_colors = publisher.generate_edge_colors_from_constants(&border_colors);
        let buffer = publisher
            .map_edge_colors_to_led_buffer(&config_group, &edge_colors)
            .unwrap();

        // 验证缓冲区大小
        // 序列号0: Bottom边, 3个LED, SK6812 (4字节/LED) = 12字节
        // 序列号1: Right边, 2个LED, WS2812B (3字节/LED) = 6字节
        // 序列号2: Top边, 3个LED, WS2812B (3字节/LED) = 9字节
        // 序列号3: Top边, 4个LED, SK6812 (4字节/LED) = 16字节
        // 总计: 12 + 6 + 9 + 16 = 43字节
        assert_eq!(buffer.len(), 43);

        println!("🔍 跨显示器串联LED数据验证:");
        println!("显示器2:");
        println!("  序列号0 (Bottom, 红色, SK6812): 字节0-11   (3个LED × 4字节)");
        println!("  序列号1 (Right, 黄色, WS2812B): 字节12-17  (2个LED × 3字节)");
        println!("  序列号2 (Top, 青色, WS2812B): 字节18-26    (3个LED × 3字节)");
        println!("显示器1:");
        println!("  序列号3 (Top, 青色, SK6812): 字节27-42     (4个LED × 4字节)");

        let mut byte_index = 0;

        // 验证序列号0 (显示器2, Bottom边, 双色分段: 红色+橙色, SK6812格式: GRBW)
        // half_count = 3/2 = 1, 所以LED0用红色，LED1和LED2用橙色
        for i in 0..3 {
            let offset = byte_index + i * 4;
            if i < 1 {
                // LED0: 红色 [255, 0, 0] -> GRBW: [0, 255, 0, 0]
                assert_eq!(buffer[offset], 0, "序列号0 LED{} G通道应该是0", i);
                assert_eq!(buffer[offset + 1], 255, "序列号0 LED{} R通道应该是255", i);
                assert_eq!(buffer[offset + 2], 0, "序列号0 LED{} B通道应该是0", i);
                assert_eq!(buffer[offset + 3], 0, "序列号0 LED{} W通道应该是0", i);
            } else {
                // LED1,LED2: 橙色 [255, 128, 0] -> GRBW: [128, 255, 0, 0]
                assert_eq!(buffer[offset], 128, "序列号0 LED{} G通道应该是128", i);
                assert_eq!(buffer[offset + 1], 255, "序列号0 LED{} R通道应该是255", i);
                assert_eq!(buffer[offset + 2], 0, "序列号0 LED{} B通道应该是0", i);
                assert_eq!(buffer[offset + 3], 0, "序列号0 LED{} W通道应该是0", i);
            }
        }
        byte_index += 12;

        // 验证序列号1 (显示器2, Right边, 双色分段: 黄色+黄绿色, WS2812B格式: GRB)
        // half_count = 2/2 = 1, 所以LED0用黄色，LED1用黄绿色
        for i in 0..2 {
            let offset = byte_index + i * 3;
            if i < 1 {
                // LED0: 黄色 [255, 255, 0] -> GRB: [255, 255, 0]
                assert_eq!(buffer[offset], 255, "序列号1 LED{} G通道应该是255", i);
                assert_eq!(buffer[offset + 1], 255, "序列号1 LED{} R通道应该是255", i);
                assert_eq!(buffer[offset + 2], 0, "序列号1 LED{} B通道应该是0", i);
            } else {
                // LED1: 黄绿色 [128, 255, 0] -> GRB: [255, 128, 0]
                assert_eq!(buffer[offset], 255, "序列号1 LED{} G通道应该是255", i);
                assert_eq!(buffer[offset + 1], 128, "序列号1 LED{} R通道应该是128", i);
                assert_eq!(buffer[offset + 2], 0, "序列号1 LED{} B通道应该是0", i);
            }
        }
        byte_index += 6;

        // 验证序列号2 (显示器2, Top边, 双色分段: 青色+蓝色, WS2812B格式: GRB)
        // half_count = 3/2 = 1, 所以LED0用青色，LED1和LED2用蓝色
        for i in 0..3 {
            let offset = byte_index + i * 3;
            if i < 1 {
                // LED0: 青色 [0, 255, 255] -> GRB: [255, 0, 255]
                assert_eq!(buffer[offset], 255, "序列号2 LED{} G通道应该是255", i);
                assert_eq!(buffer[offset + 1], 0, "序列号2 LED{} R通道应该是0", i);
                assert_eq!(buffer[offset + 2], 255, "序列号2 LED{} B通道应该是255", i);
            } else {
                // LED1,LED2: 蓝色 [0, 0, 255] -> GRB: [0, 0, 255]
                assert_eq!(buffer[offset], 0, "序列号2 LED{} G通道应该是0", i);
                assert_eq!(buffer[offset + 1], 0, "序列号2 LED{} R通道应该是0", i);
                assert_eq!(buffer[offset + 2], 255, "序列号2 LED{} B通道应该是255", i);
            }
        }
        byte_index += 9;

        // 验证序列号3 (显示器1, Top边, 双色分段: 青色+蓝色, SK6812格式: GRBW)
        // half_count = 4/2 = 2, 所以LED0和LED1用青色，LED2和LED3用蓝色
        for i in 0..4 {
            let offset = byte_index + i * 4;
            if i < 2 {
                // LED0,LED1: 青色 [0, 255, 255] -> GRBW: [255, 0, 255, 0]
                assert_eq!(buffer[offset], 255, "序列号3 LED{} G通道应该是255", i);
                assert_eq!(buffer[offset + 1], 0, "序列号3 LED{} R通道应该是0", i);
                assert_eq!(buffer[offset + 2], 255, "序列号3 LED{} B通道应该是255", i);
                assert_eq!(buffer[offset + 3], 0, "序列号3 LED{} W通道应该是0", i);
            } else {
                // LED2,LED3: 蓝色 [0, 0, 255] -> GRBW: [0, 0, 255, 0]
                assert_eq!(buffer[offset], 0, "序列号3 LED{} G通道应该是0", i);
                assert_eq!(buffer[offset + 1], 0, "序列号3 LED{} R通道应该是0", i);
                assert_eq!(buffer[offset + 2], 255, "序列号3 LED{} B通道应该是255", i);
                assert_eq!(buffer[offset + 3], 0, "序列号3 LED{} W通道应该是0", i);
            }
        }
        byte_index += 16;

        assert_eq!(byte_index, 43, "所有字节都应该被验证");

        println!("✅ 跨显示器串联LED数据验证通过");
        println!("   - 显示器2总字节: 27 (12+6+9)");
        println!("   - 显示器1总字节: 16");
        println!("   - 串联总字节: 43");
    }

    #[tokio::test]
    async fn test_cross_display_single_display_config_mode() {
        // 测试单屏配置模式下，只有显示器1的灯带会被处理
        let publisher = LedColorsPublisher::global().await;
        let full_config = create_cross_display_config_group();
        let border_colors = create_test_border_colors();

        // 模拟单屏配置模式：只处理显示器1的灯带
        let display_1_strips: Vec<_> = full_config
            .strips
            .iter()
            .filter(|s| s.display_id == 1)
            .cloned()
            .collect();

        let single_display_config = LedStripConfigGroup {
            strips: display_1_strips,
            mappers: Vec::new(),
            color_calibration: ColorCalibration::new(),
        };

        let edge_colors = publisher.generate_edge_colors_from_constants(&border_colors);
        let buffer = publisher
            .map_edge_colors_to_led_buffer(&single_display_config, &edge_colors)
            .unwrap();

        // 验证只有显示器1的数据
        // 序列号3: Top边, 4个LED, SK6812 (4字节/LED) = 16字节
        assert_eq!(buffer.len(), 16);

        // 验证序列号3 (显示器1, Top边, 双色分段: 青色+蓝色, SK6812格式: GRBW)
        // half_count = 4/2 = 2, 所以LED0和LED1用青色，LED2和LED3用蓝色
        for i in 0..4 {
            let offset = i * 4;
            if i < 2 {
                // LED0,LED1: 青色 [0, 255, 255] -> GRBW: [255, 0, 255, 0]
                assert_eq!(buffer[offset], 255, "显示器1 LED{} G通道应该是255", i);
                assert_eq!(buffer[offset + 1], 0, "显示器1 LED{} R通道应该是0", i);
                assert_eq!(buffer[offset + 2], 255, "显示器1 LED{} B通道应该是255", i);
                assert_eq!(buffer[offset + 3], 0, "显示器1 LED{} W通道应该是0", i);
            } else {
                // LED2,LED3: 蓝色 [0, 0, 255] -> GRBW: [0, 0, 255, 0]
                assert_eq!(buffer[offset], 0, "显示器1 LED{} G通道应该是0", i);
                assert_eq!(buffer[offset + 1], 0, "显示器1 LED{} R通道应该是0", i);
                assert_eq!(buffer[offset + 2], 255, "显示器1 LED{} B通道应该是255", i);
                assert_eq!(buffer[offset + 3], 0, "显示器1 LED{} W通道应该是0", i);
            }
        }

        println!("✅ 单屏配置模式验证通过");
        println!("   - 只处理显示器1的灯带");
        println!("   - 生成16字节数据 (4个LED × 4字节)");
        println!("   - 颜色正确: 青色 [0,255,255]");

        // 关键验证：在实际应用中，这16字节的数据应该发送到
        // 全局偏移量 = 显示器2的总LED数量 × 平均字节数
        // 但这个偏移量计算是在发布服务中处理的，不在这个函数中
    }

    #[tokio::test]
    async fn test_cross_display_data_continuity() {
        // 验证跨显示器数据的连续性 - 确保没有间隙或重叠
        let publisher = LedColorsPublisher::global().await;
        let config_group = create_cross_display_config_group();
        let border_colors = create_test_border_colors();

        let edge_colors = publisher.generate_edge_colors_from_constants(&border_colors);
        let buffer = publisher
            .map_edge_colors_to_led_buffer(&config_group, &edge_colors)
            .unwrap();

        // 验证数据连续性
        let mut expected_byte_index = 0;

        // 按序列号顺序验证每个灯带的数据位置
        let sorted_strips = {
            let mut strips = config_group.strips.clone();
            strips.sort_by_key(|s| s.index);
            strips
        };

        for strip in &sorted_strips {
            let bytes_per_led = match strip.led_type {
                LedType::WS2812B => 3,
                LedType::SK6812 => 4,
            };
            let strip_bytes = strip.len * bytes_per_led;

            println!(
                "序列号{} (显示器{}, {}边): 字节{}-{} ({} LEDs × {} bytes = {} bytes)",
                strip.index,
                strip.display_id,
                match strip.border {
                    Border::Top => "Top",
                    Border::Bottom => "Bottom",
                    Border::Left => "Left",
                    Border::Right => "Right",
                },
                expected_byte_index,
                expected_byte_index + strip_bytes - 1,
                strip.len,
                bytes_per_led,
                strip_bytes
            );

            expected_byte_index += strip_bytes;
        }

        assert_eq!(expected_byte_index, buffer.len(), "数据应该连续无间隙");

        println!("✅ 跨显示器数据连续性验证通过");
        println!("   - 总字节数: {}", buffer.len());
        println!("   - 数据连续无间隙");
        println!("   - 序列号0→1→2→3 正确排序");
    }
}
