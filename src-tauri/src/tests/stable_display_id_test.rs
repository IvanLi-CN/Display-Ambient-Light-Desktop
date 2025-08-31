use crate::ambient_light::{
    Border, ColorCalibration, LedStripConfigGroupV2, LedStripConfigV2, LedType,
};
use crate::display::{DisplayConfig, DisplayConfigGroup};
use std::time::SystemTime;

/// 测试稳定显示器ID系统的基本功能
#[tokio::test]
async fn test_stable_display_id_basic_functionality() {
    // 创建测试显示器配置
    let mut display_config = DisplayConfigGroup::new();

    let display1 = DisplayConfig::new("测试显示器1".to_string(), 1920, 1080, 1.0, true);
    let display1_id = display1.internal_id.clone();

    let display2 = DisplayConfig::new("测试显示器2".to_string(), 2560, 1440, 1.0, false);
    let display2_id = display2.internal_id.clone();

    display_config.add_display(display1);
    display_config.add_display(display2);

    // 创建LED灯带配置
    let strips = vec![
        LedStripConfigV2 {
            index: 0,
            border: Border::Top,
            display_internal_id: display1_id.clone(),
            len: 30,
            led_type: LedType::WS2812B,
            reversed: false,
        },
        LedStripConfigV2 {
            index: 1,
            border: Border::Right,
            display_internal_id: display1_id.clone(),
            len: 20,
            led_type: LedType::WS2812B,
            reversed: false,
        },
        LedStripConfigV2 {
            index: 2,
            border: Border::Top,
            display_internal_id: display2_id.clone(),
            len: 40,
            led_type: LedType::SK6812,
            reversed: true,
        },
    ];

    // 创建配置组
    let mut config = LedStripConfigGroupV2 {
        version: 2,
        display_config,
        strips,
        mappers: Vec::new(),
        color_calibration: ColorCalibration::new(),
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    };

    // 生成mappers
    config.generate_mappers();

    // 验证mappers生成正确
    assert_eq!(config.mappers.len(), 3);

    // 验证第一个灯带的mapper
    assert_eq!(config.mappers[0].start, 0);
    assert_eq!(config.mappers[0].end, 30);
    assert_eq!(config.mappers[0].pos, 0);

    // 验证第二个灯带的mapper
    assert_eq!(config.mappers[1].start, 30);
    assert_eq!(config.mappers[1].end, 50);
    assert_eq!(config.mappers[1].pos, 30);

    // 验证第三个灯带的mapper（反向）
    assert_eq!(config.mappers[2].start, 90); // 反向时start和end交换
    assert_eq!(config.mappers[2].end, 50);
    assert_eq!(config.mappers[2].pos, 50);

    println!("✅ 稳定显示器ID系统基本功能测试通过");
}

/// 测试显示器配置的精确匹配功能
#[tokio::test]
async fn test_display_exact_match() {
    let display_config = DisplayConfig::new("测试显示器".to_string(), 1920, 1080, 2.0, true);

    // 创建匹配的显示器信息
    let matching_display = display_info::DisplayInfo {
        id: 1,
        width: 1920,
        height: 1080,
        scale_factor: 2.0,
        is_primary: true,
        x: 0,
        y: 0,
        rotation: 0.0,
        frequency: 60.0,
        raw_handle: unsafe { std::mem::zeroed() },
    };

    // 创建不匹配的显示器信息
    let non_matching_display = display_info::DisplayInfo {
        id: 2,
        width: 2560,
        height: 1440,
        scale_factor: 1.0,
        is_primary: false,
        x: 1920,
        y: 0,
        rotation: 0.0,
        frequency: 60.0,
        raw_handle: unsafe { std::mem::zeroed() },
    };

    // 测试精确匹配
    assert!(display_config.exact_match(&matching_display));
    assert!(!display_config.exact_match(&non_matching_display));

    println!("✅ 显示器精确匹配功能测试通过");
}

/// 测试配置序列化和反序列化
#[tokio::test]
async fn test_config_serialization() {
    let mut config = LedStripConfigGroupV2::new();

    // 添加测试数据
    let display = DisplayConfig::new("序列化测试显示器".to_string(), 1920, 1080, 1.0, true);
    let display_id = display.internal_id.clone();
    config.display_config.add_display(display);

    let strip = LedStripConfigV2 {
        index: 0,
        border: Border::Top,
        display_internal_id: display_id,
        len: 30,
        led_type: LedType::WS2812B,
        reversed: false,
    };
    config.strips.push(strip);
    config.generate_mappers();

    // 序列化
    let serialized = toml::to_string_pretty(&config).expect("序列化失败");
    println!("序列化结果:\n{}", serialized);

    // 反序列化
    let mut deserialized: LedStripConfigGroupV2 =
        toml::from_str(&serialized).expect("反序列化失败");
    deserialized.generate_mappers();

    // 验证数据一致性
    assert_eq!(config.version, deserialized.version);
    assert_eq!(config.strips.len(), deserialized.strips.len());
    assert_eq!(
        config.display_config.displays.len(),
        deserialized.display_config.displays.len()
    );
    assert_eq!(config.mappers.len(), deserialized.mappers.len());

    println!("✅ 配置序列化和反序列化测试通过");
}

/// 测试LED灯带起始位置计算
#[tokio::test]
async fn test_led_strip_start_position_calculation() {
    let display_id = "test_display".to_string();

    let strips = vec![
        LedStripConfigV2 {
            index: 0,
            border: Border::Top,
            display_internal_id: display_id.clone(),
            len: 10,
            led_type: LedType::WS2812B,
            reversed: false,
        },
        LedStripConfigV2 {
            index: 1,
            border: Border::Right,
            display_internal_id: display_id.clone(),
            len: 15,
            led_type: LedType::WS2812B,
            reversed: false,
        },
        LedStripConfigV2 {
            index: 2,
            border: Border::Bottom,
            display_internal_id: display_id.clone(),
            len: 20,
            led_type: LedType::WS2812B,
            reversed: false,
        },
    ];

    // 测试每个灯带的起始位置
    assert_eq!(strips[0].calculate_start_pos(&strips), 0); // 第一个灯带从0开始
    assert_eq!(strips[1].calculate_start_pos(&strips), 10); // 第二个灯带从10开始
    assert_eq!(strips[2].calculate_start_pos(&strips), 25); // 第三个灯带从25开始

    println!("✅ LED灯带起始位置计算测试通过");
}
