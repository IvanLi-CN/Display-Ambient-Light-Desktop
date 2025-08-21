#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::display::{
        DisplayConfig, DisplayConfigGroup, DisplayMatcher, DisplayRegistry, MatchType,
    };

    /// 创建测试用的显示器信息
    fn create_test_display_info(
        id: u32,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        is_primary: bool,
    ) -> display_info::DisplayInfo {
        display_info::DisplayInfo {
            id,
            x,
            y,
            width,
            height,
            rotation: 0.0,
            scale_factor: 1.0,
            is_primary,
            frequency: 60.0,
            raw_handle: unsafe { std::mem::zeroed() },
        }
    }

    #[test]
    fn test_display_config_creation() {
        let config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);

        assert_eq!(config.name, "Test Display");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.scale_factor, 1.0);
        assert!(config.is_primary);
        assert!(config.internal_id.starts_with("display_"));
        assert!(config.last_detected_at.is_none());
    }

    #[test]
    fn test_display_config_from_display_info() {
        let display_info = create_test_display_info(1, 0, 0, 1920, 1080, true);
        let config = DisplayConfig::from_display_info(&display_info);

        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert!(config.is_primary);
        assert_eq!(config.last_system_id, Some(1));
        assert!(config.last_detected_at.is_some());
        assert!(config.last_position.is_some());
    }

    #[test]
    fn test_display_config_matching() {
        let mut config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);

        let display_info = create_test_display_info(1, 0, 0, 1920, 1080, true);

        // 测试精确匹配
        assert!(config.exact_match(&display_info));
        assert!(config.partial_match(&display_info));

        // 测试匹配分数
        let score = config.match_score(&display_info);
        assert_eq!(score, 80); // 尺寸(40) + 缩放(20) + 主显示器(20) = 80

        // 更新检测信息后测试位置匹配
        config.update_last_detected(&display_info);
        let score_with_position = config.match_score(&display_info);
        assert_eq!(score_with_position, 100); // 80 + 位置(20) = 100

        // 测试部分匹配
        let different_display = create_test_display_info(2, 100, 100, 1920, 1080, false);
        assert!(!config.exact_match(&different_display));
        assert!(config.partial_match(&different_display)); // 尺寸相同
    }

    #[test]
    fn test_display_config_group() {
        let mut group = DisplayConfigGroup::new();
        assert_eq!(group.displays.len(), 0);

        let config1 = DisplayConfig::new("Display 1".to_string(), 1920, 1080, 1.0, true);
        let config2 = DisplayConfig::new("Display 2".to_string(), 2560, 1440, 2.0, false);

        let id1 = config1.internal_id.clone();
        let id2 = config2.internal_id.clone();

        group.add_display(config1);
        group.add_display(config2);

        assert_eq!(group.displays.len(), 2);

        // 测试查找
        let found = group.find_by_internal_id(&id1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Display 1");

        // 测试更新
        let mut updated_config = group.find_by_internal_id(&id1).unwrap().clone();
        updated_config.name = "Updated Display 1".to_string();
        assert!(group.update_display(updated_config));

        let updated = group.find_by_internal_id(&id1).unwrap();
        assert_eq!(updated.name, "Updated Display 1");

        // 测试删除
        assert!(group.remove_display(&id2));
        assert_eq!(group.displays.len(), 1);
        assert!(group.find_by_internal_id(&id2).is_none());
    }

    #[test]
    fn test_display_matcher_exact_matching() {
        let mut config_group = DisplayConfigGroup::new();
        let config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);
        config_group.add_display(config);

        let matcher = DisplayMatcher::new(config_group);

        let system_display = create_test_display_info(1, 0, 0, 1920, 1080, true);
        let results = matcher.match_displays(&[system_display]).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Exact);
        assert!(results[0].match_score >= 80);
    }

    #[test]
    fn test_display_matcher_new_display_detection() {
        let config_group = DisplayConfigGroup::new(); // 空配置
        let matcher = DisplayMatcher::new(config_group);

        let system_display = create_test_display_info(1, 0, 0, 1920, 1080, true);
        let results = matcher.match_displays(&[system_display]).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::New);
        assert_eq!(results[0].match_score, 0);
        assert!(results[0].config_internal_id.is_empty());
    }

    #[test]
    fn test_display_matcher_partial_matching() {
        let mut config_group = DisplayConfigGroup::new();
        let mut config = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);

        // 设置不同的主显示器状态，但尺寸相同
        config.is_primary = false;
        config_group.add_display(config);

        let matcher = DisplayMatcher::new(config_group);

        let system_display = create_test_display_info(1, 0, 0, 1920, 1080, true);
        let results = matcher.match_displays(&[system_display]).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Partial);
        assert!(results[0].match_score >= 40); // 至少有尺寸匹配分数
        assert!(results[0].match_score < 80); // 但不是精确匹配
    }

    #[tokio::test]
    async fn test_display_registry_basic_operations() {
        let config_group = DisplayConfigGroup::new();
        let registry = DisplayRegistry::new(config_group);

        // 测试添加显示器
        let display = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);
        let internal_id = display.internal_id.clone();

        registry.add_display(display).await.unwrap();

        // 测试查找
        let found = registry.find_display_by_internal_id(&internal_id).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Display");

        // 测试获取所有显示器
        let all_displays = registry.get_all_displays().await;
        assert_eq!(all_displays.len(), 1);

        // 测试统计信息
        let stats = registry.get_display_stats().await;
        assert_eq!(stats.total_displays, 1);
        assert_eq!(stats.primary_displays, 1);

        // 测试删除
        let removed = registry.remove_display(&internal_id).await.unwrap();
        assert!(removed);

        let all_displays_after_remove = registry.get_all_displays().await;
        assert_eq!(all_displays_after_remove.len(), 0);
    }

    #[test]
    fn test_config_migrator_display_id_mapping() {
        let mut display_config_group = DisplayConfigGroup::new();

        // 添加两个显示器配置
        let display1 = DisplayConfig::new("Display 1".to_string(), 1920, 1080, 1.0, true);
        let display2 = DisplayConfig::new("Display 2".to_string(), 1920, 1080, 1.0, false);

        let id1 = display1.internal_id.clone();
        let id2 = display2.internal_id.clone();

        display_config_group.add_display(display1);
        display_config_group.add_display(display2);

        // ConfigMigrator已被移除，这些测试不再适用
        // 现在使用DisplayRegistry进行显示器ID映射
        println!("Display 1 ID: {}", id1);
        println!("Display 2 ID: {}", id2);
    }

    #[test]
    fn test_led_strip_config_v2_creation() {
        let strip = crate::ambient_light::LedStripConfigV2 {
            index: 0,
            border: crate::ambient_light::Border::Top,
            display_internal_id: "display_test_123".to_string(),
            len: 30,
            led_type: crate::ambient_light::LedType::WS2812B,
            reversed: false,
        };

        assert_eq!(strip.index, 0);
        assert_eq!(strip.display_internal_id, "display_test_123");
        assert_eq!(strip.len, 30);
        assert!(!strip.reversed);
    }

    #[test]
    fn test_led_strip_config_group_v2_creation() {
        let mut config = crate::ambient_light::LedStripConfigGroupV2::new();
        assert_eq!(config.version, 2);
        assert_eq!(config.strips.len(), 0);
        assert_eq!(config.display_config.displays.len(), 0);

        // 添加显示器配置
        let display = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);
        let display_id = display.internal_id.clone();
        config.display_config.add_display(display);

        // 添加LED灯带配置
        let strip = crate::ambient_light::LedStripConfigV2 {
            index: 0,
            border: crate::ambient_light::Border::Top,
            display_internal_id: display_id,
            len: 30,
            led_type: crate::ambient_light::LedType::WS2812B,
            reversed: false,
        };
        config.strips.push(strip);

        // 生成mappers
        config.generate_mappers();

        assert_eq!(config.strips.len(), 1);
        assert_eq!(config.mappers.len(), 1);
        assert_eq!(config.display_config.displays.len(), 1);
    }

    #[test]
    fn test_position_relations_calculation() {
        let displays = vec![
            create_test_display_info(1, 0, 0, 1920, 1080, true), // 左显示器
            create_test_display_info(2, 2000, 0, 1920, 1080, false), // 右显示器（增加间距避免重叠）
        ];

        let config_group = DisplayConfigGroup::new();
        let matcher = DisplayMatcher::new(config_group);
        let relations = matcher.calculate_position_relations(&displays);

        // 检查第一个显示器的关系（应该有一个关系指向第二个显示器）
        let display0_relations = relations.get(&0).unwrap();
        assert!(!display0_relations.is_empty());

        // 检查第二个显示器的关系（应该有一个关系指向第一个显示器）
        let display1_relations = relations.get(&1).unwrap();
        assert!(!display1_relations.is_empty());

        // 验证关系的正确性
        // 第一个显示器应该看到第二个显示器在右边
        assert!(display0_relations.iter().any(|r| r.contains("right_of")));
        // 第二个显示器应该看到第一个显示器在左边
        assert!(display1_relations.iter().any(|r| r.contains("left_of")));
    }
}
