//! 跨显示器串联LED灯带全局位置计算测试
//! 
//! 这个测试模块验证跨显示器串联LED灯带的全局位置计算逻辑是否正确。
//! 测试场景基于实际配置文件中的数据。

use crate::ambient_light::config::{LedStripConfig, LedStripConfigGroup, Border, LedType};

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用的LED灯带配置
    /// 模拟实际配置文件中的跨显示器串联场景
    fn create_test_strips() -> Vec<LedStripConfig> {
        vec![
            // 显示器2的灯带 (序列号0-2)
            LedStripConfig {
                index: 0,
                border: Border::Bottom,
                display_id: 2,
                len: 38,
                led_type: LedType::SK6812,
                reversed: false,
            },
            LedStripConfig {
                index: 1,
                border: Border::Right,
                display_id: 2,
                len: 22,
                led_type: LedType::WS2812B,
                reversed: false,
            },
            LedStripConfig {
                index: 2,
                border: Border::Top,
                display_id: 2,
                len: 38,
                led_type: LedType::WS2812B,
                reversed: false,
            },
            // 显示器1的灯带 (序列号3，继续串联)
            LedStripConfig {
                index: 3,
                border: Border::Top,
                display_id: 1,
                len: 38,
                led_type: LedType::SK6812,
                reversed: false,
            },
        ]
    }

    #[test]
    fn test_global_start_pos_calculation() {
        let strips = create_test_strips();
        
        // 测试每个灯带的全局起始位置计算
        
        // 序列号0的灯带：应该从LED 0开始
        let strip_0_start = strips[0].calculate_start_pos(&strips);
        assert_eq!(strip_0_start, 0, "序列号0的灯带应该从LED 0开始");
        
        // 序列号1的灯带：应该从LED 38开始 (0 + 38)
        let strip_1_start = strips[1].calculate_start_pos(&strips);
        assert_eq!(strip_1_start, 38, "序列号1的灯带应该从LED 38开始");
        
        // 序列号2的灯带：应该从LED 60开始 (38 + 22)
        let strip_2_start = strips[2].calculate_start_pos(&strips);
        assert_eq!(strip_2_start, 60, "序列号2的灯带应该从LED 60开始");
        
        // 序列号3的灯带：应该从LED 98开始 (60 + 38)
        let strip_3_start = strips[3].calculate_start_pos(&strips);
        assert_eq!(strip_3_start, 98, "序列号3的灯带应该从LED 98开始");
    }

    #[test]
    fn test_display_specific_start_positions() {
        let strips = create_test_strips();
        
        // 测试显示器2的灯带起始位置
        let display_2_strips: Vec<_> = strips.iter().filter(|s| s.display_id == 2).collect();
        assert_eq!(display_2_strips.len(), 3, "显示器2应该有3个灯带");
        
        // 显示器2的第一个灯带（序列号0）
        assert_eq!(display_2_strips[0].calculate_start_pos(&strips), 0);
        
        // 测试显示器1的灯带起始位置
        let display_1_strips: Vec<_> = strips.iter().filter(|s| s.display_id == 1).collect();
        assert_eq!(display_1_strips.len(), 1, "显示器1应该有1个灯带");
        
        // 显示器1的灯带（序列号3）应该在显示器2的所有灯带之后
        let display_1_start = display_1_strips[0].calculate_start_pos(&strips);
        assert_eq!(display_1_start, 98, "显示器1的灯带应该从LED 98开始");
        
        // 验证这确实是在显示器2的所有灯带之后
        let display_2_total_leds: usize = display_2_strips.iter().map(|s| s.len).sum();
        assert_eq!(display_2_total_leds, 98, "显示器2的总LED数量应该是98");
        assert_eq!(display_1_start, display_2_total_leds, "显示器1应该紧接在显示器2之后");
    }

    #[test]
    fn test_total_led_count() {
        let strips = create_test_strips();
        
        let total_leds: usize = strips.iter().map(|s| s.len).sum();
        assert_eq!(total_leds, 136, "总LED数量应该是136 (38+22+38+38)");
        
        // 验证最后一个LED的位置
        let last_strip = &strips[3];
        let last_strip_start = last_strip.calculate_start_pos(&strips);
        let last_led_position = last_strip_start + last_strip.len - 1;
        assert_eq!(last_led_position, 135, "最后一个LED的位置应该是135");
    }

    #[test]
    fn test_mappers_generation() {
        let strips = create_test_strips();
        let mut config = LedStripConfigGroup {
            strips: strips.clone(),
            mappers: Vec::new(),
            color_calibration: crate::ambient_light::config::ColorCalibration::new(),
        };
        
        // 生成mappers
        config.generate_mappers();
        
        assert_eq!(config.mappers.len(), 4, "应该生成4个mappers");
        
        // 验证每个mapper的范围
        assert_eq!(config.mappers[0].start, 0);   // 序列号0: 0-38
        assert_eq!(config.mappers[0].end, 38);
        
        assert_eq!(config.mappers[1].start, 38);  // 序列号1: 38-60
        assert_eq!(config.mappers[1].end, 60);
        
        assert_eq!(config.mappers[2].start, 60);  // 序列号2: 60-98
        assert_eq!(config.mappers[2].end, 98);
        
        assert_eq!(config.mappers[3].start, 98);  // 序列号3: 98-136
        assert_eq!(config.mappers[3].end, 136);
    }

    #[test]
    fn test_unordered_strips_calculation() {
        // 测试乱序的灯带数组，验证计算逻辑的健壮性
        let mut strips = create_test_strips();
        
        // 打乱顺序
        strips.reverse();
        
        // 即使顺序被打乱，计算结果应该保持一致
        assert_eq!(strips.iter().find(|s| s.index == 0).unwrap().calculate_start_pos(&strips), 0);
        assert_eq!(strips.iter().find(|s| s.index == 1).unwrap().calculate_start_pos(&strips), 38);
        assert_eq!(strips.iter().find(|s| s.index == 2).unwrap().calculate_start_pos(&strips), 60);
        assert_eq!(strips.iter().find(|s| s.index == 3).unwrap().calculate_start_pos(&strips), 98);
    }

    #[test]
    fn test_single_display_offset_calculation() {
        let strips = create_test_strips();
        
        // 模拟单屏配置界面的场景：只显示显示器1的灯带
        let display_1_strips: Vec<_> = strips.iter().filter(|s| s.display_id == 1).cloned().collect();
        
        // 但计算全局位置时需要考虑所有灯带
        let display_1_strip = &display_1_strips[0];
        let global_start_pos = display_1_strip.calculate_start_pos(&strips); // 传入所有灯带
        
        assert_eq!(global_start_pos, 98, "显示器1的灯带在全局串联中应该从LED 98开始");
        
        // 这就是修复的关键：单屏界面显示的是全局位置，而不是本地位置
        let local_start_pos = display_1_strip.calculate_start_pos(&display_1_strips); // 只传入本显示器的灯带
        assert_eq!(local_start_pos, 0, "如果只考虑本显示器，起始位置是0（这是错误的）");
        
        // 证明我们的修复是正确的：必须使用全局位置
        assert_ne!(global_start_pos, local_start_pos, "全局位置和本地位置应该不同");
    }

    #[test]
    fn test_real_world_scenario() {
        // 测试真实世界的使用场景
        let strips = create_test_strips();

        println!("\n🔍 真实场景测试：跨显示器串联LED灯带");
        println!("配置文件内容:");

        let mut sorted_strips = strips.clone();
        sorted_strips.sort_by_key(|s| s.index);

        for strip in &sorted_strips {
            let start_pos = strip.calculate_start_pos(&strips);
            let end_pos = start_pos + strip.len - 1;

            println!("  序列号{}: 显示器{}, {}边, {}个LED, LED范围: {}-{}",
                strip.index,
                strip.display_id,
                match strip.border {
                    Border::Top => "Top",
                    Border::Bottom => "Bottom",
                    Border::Left => "Left",
                    Border::Right => "Right",
                },
                strip.len,
                start_pos,
                end_pos
            );
        }

        println!("\n📊 验证结果:");

        // 验证显示器1的灯带确实在显示器2之后
        let display_1_strip = strips.iter().find(|s| s.display_id == 1).unwrap();
        let display_1_start = display_1_strip.calculate_start_pos(&strips);

        let display_2_total: usize = strips.iter()
            .filter(|s| s.display_id == 2)
            .map(|s| s.len)
            .sum();

        assert_eq!(display_1_start, display_2_total,
            "显示器1应该紧接在显示器2的所有LED之后");

        println!("  ✅ 显示器1的灯带正确地从LED {}开始", display_1_start);
        println!("  ✅ 这正好是显示器2的{}个LED之后", display_2_total);

        // 验证总LED数量
        let total_leds: usize = strips.iter().map(|s| s.len).sum();
        println!("  ✅ 总LED数量: {}", total_leds);

        // 验证没有LED重叠
        let mut led_ranges = Vec::new();
        for strip in &sorted_strips {
            let start = strip.calculate_start_pos(&strips);
            let end = start + strip.len;
            led_ranges.push((start, end));
        }

        for i in 1..led_ranges.len() {
            assert_eq!(led_ranges[i-1].1, led_ranges[i].0,
                "LED范围应该连续，不能有重叠或间隙");
        }

        println!("  ✅ 所有LED范围连续，无重叠或间隙");
    }
}
