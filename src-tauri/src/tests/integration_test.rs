use crate::ambient_light::{
    Border, ColorCalibration, ConfigManagerV2, LedStripConfigGroupV2, LedStripConfigV2, LedType,
    PublisherAdapter,
};
use crate::display::{ConfigMigrator, DisplayConfig, DisplayConfigGroup, DisplayRegistry};
use std::time::SystemTime;

/// 集成测试：完整的稳定显示器ID系统工作流程
#[tokio::test]
async fn test_complete_stable_display_id_workflow() {
    println!("🚀 开始稳定显示器ID系统集成测试...");

    // 步骤1: 创建显示器配置
    println!("📺 步骤1: 创建显示器配置");
    let mut display_config = DisplayConfigGroup::new();

    let display1 = DisplayConfig::new("主显示器".to_string(), 1920, 1080, 2.0, true);
    let display1_id = display1.internal_id.clone();
    println!("   创建显示器1: {} ({})", display1.name, display1_id);

    let display2 = DisplayConfig::new("副显示器".to_string(), 2560, 1440, 1.0, false);
    let display2_id = display2.internal_id.clone();
    println!("   创建显示器2: {} ({})", display2.name, display2_id);

    display_config.add_display(display1);
    display_config.add_display(display2);

    // 步骤2: 创建显示器注册管理器
    println!("🗂️ 步骤2: 创建显示器注册管理器");
    let display_registry = DisplayRegistry::new(display_config);

    // 步骤3: 创建LED灯带配置
    println!("💡 步骤3: 创建LED灯带配置");
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

    println!("   创建了 {} 个LED灯带配置", strips.len());

    // 步骤4: 创建完整的配置组
    println!("⚙️ 步骤4: 创建完整的配置组");
    let mut config = LedStripConfigGroupV2 {
        version: 2,
        display_config: display_registry.get_config_group().await,
        strips,
        mappers: Vec::new(),
        color_calibration: ColorCalibration::new(),
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    };

    // 生成mappers
    config.generate_mappers();
    println!("   生成了 {} 个LED映射器", config.mappers.len());

    // 步骤5: 验证mappers正确性
    println!("✅ 步骤5: 验证LED映射器正确性");
    assert_eq!(config.mappers.len(), 3);

    // 验证第一个灯带 (正向, 30个LED)
    assert_eq!(config.mappers[0].start, 0);
    assert_eq!(config.mappers[0].end, 30);
    assert_eq!(config.mappers[0].pos, 0);
    println!(
        "   灯带0: start={}, end={}, pos={}",
        config.mappers[0].start, config.mappers[0].end, config.mappers[0].pos
    );

    // 验证第二个灯带 (正向, 20个LED)
    assert_eq!(config.mappers[1].start, 30);
    assert_eq!(config.mappers[1].end, 50);
    assert_eq!(config.mappers[1].pos, 30);
    println!(
        "   灯带1: start={}, end={}, pos={}",
        config.mappers[1].start, config.mappers[1].end, config.mappers[1].pos
    );

    // 验证第三个灯带 (反向, 40个LED)
    assert_eq!(config.mappers[2].start, 90); // 反向时start和end交换
    assert_eq!(config.mappers[2].end, 50);
    assert_eq!(config.mappers[2].pos, 50);
    println!(
        "   灯带2: start={}, end={}, pos={} (反向)",
        config.mappers[2].start, config.mappers[2].end, config.mappers[2].pos
    );

    // 步骤6: 测试配置序列化
    println!("💾 步骤6: 测试配置序列化");
    let serialized = toml::to_string_pretty(&config).expect("序列化失败");
    println!("   配置序列化成功，长度: {} 字符", serialized.len());

    // 步骤7: 测试配置反序列化
    println!("📖 步骤7: 测试配置反序列化");
    let mut deserialized: LedStripConfigGroupV2 =
        toml::from_str(&serialized).expect("反序列化失败");
    deserialized.generate_mappers();

    // 验证反序列化后的数据一致性
    assert_eq!(config.version, deserialized.version);
    assert_eq!(config.strips.len(), deserialized.strips.len());
    assert_eq!(
        config.display_config.displays.len(),
        deserialized.display_config.displays.len()
    );
    assert_eq!(config.mappers.len(), deserialized.mappers.len());
    println!("   反序列化验证成功");

    // 步骤8: 测试适配器转换
    println!("🔄 步骤8: 测试配置格式转换");
    let adapter = PublisherAdapter::new(std::sync::Arc::new(display_registry));

    // v2 -> v1 转换
    let v1_config = adapter
        .convert_v2_to_v1_config(&config)
        .await
        .expect("v2到v1转换失败");
    println!("   v2->v1转换成功，灯带数量: {}", v1_config.strips.len());

    // v1 -> v2 转换
    let v2_config = adapter
        .convert_v1_to_v2_config(&v1_config)
        .await
        .expect("v1到v2转换失败");
    println!("   v1->v2转换成功，灯带数量: {}", v2_config.strips.len());

    // 验证往返转换的一致性
    assert_eq!(config.strips.len(), v2_config.strips.len());
    println!("   往返转换验证成功");

    println!("🎉 稳定显示器ID系统集成测试完成！");
}

/// 测试配置迁移功能
#[tokio::test]
async fn test_config_migration() {
    println!("🔄 开始配置迁移测试...");

    // 检查是否需要迁移
    let needs_migration = ConfigMigrator::needs_migration().await;
    println!("   需要迁移: {}", needs_migration);

    // 如果需要迁移，执行迁移
    if needs_migration {
        match ConfigMigrator::migrate_all_configs().await {
            Ok(_) => println!("   ✅ 配置迁移成功"),
            Err(e) => println!("   ❌ 配置迁移失败: {}", e),
        }
    } else {
        println!("   ℹ️ 无需迁移");
    }

    println!("✅ 配置迁移测试完成");
}

/// 测试ConfigManagerV2的基本功能
#[tokio::test]
async fn test_config_manager_v2_basic_operations() {
    println!("⚙️ 开始ConfigManagerV2基本操作测试...");

    // 获取全局配置管理器
    let config_manager = ConfigManagerV2::global().await;
    println!("   ✅ ConfigManagerV2创建成功");

    // 获取当前配置
    let current_config = config_manager.get_config().await;
    println!("   📋 当前配置版本: {}", current_config.version);
    println!("   💡 当前灯带数量: {}", current_config.strips.len());
    println!(
        "   📺 显示器数量: {}",
        current_config.display_config.displays.len()
    );

    // 测试颜色校准更新
    let mut new_calibration = ColorCalibration::new();
    new_calibration.r = 0.9;
    new_calibration.g = 1.0;
    new_calibration.b = 1.1;
    new_calibration.w = 0.8;

    match config_manager
        .update_color_calibration(new_calibration)
        .await
    {
        Ok(_) => println!("   ✅ 颜色校准更新成功"),
        Err(e) => println!("   ❌ 颜色校准更新失败: {}", e),
    }

    // 验证更新后的配置
    let updated_config = config_manager.get_config().await;
    assert!((updated_config.color_calibration.r - 0.9).abs() < 0.01);
    assert!((updated_config.color_calibration.w - 0.8).abs() < 0.01);
    println!("   ✅ 颜色校准验证成功");

    println!("🎉 ConfigManagerV2基本操作测试完成！");
}

/// 性能测试：大量显示器和灯带配置
#[tokio::test]
async fn test_performance_with_many_displays() {
    println!("🚀 开始性能测试...");

    let start_time = std::time::Instant::now();

    // 创建大量显示器配置
    let mut display_config = DisplayConfigGroup::new();
    for i in 0..10 {
        let display = DisplayConfig::new(
            format!("显示器{}", i + 1),
            1920 + i * 100,
            1080 + i * 50,
            1.0 + i as f32 * 0.1,
            i == 0,
        );
        display_config.add_display(display);
    }

    // 创建大量LED灯带配置
    let mut strips = Vec::new();
    for (display_idx, display) in display_config.displays.iter().enumerate() {
        for border_idx in 0..4 {
            let border = match border_idx {
                0 => Border::Top,
                1 => Border::Right,
                2 => Border::Bottom,
                _ => Border::Left,
            };

            strips.push(LedStripConfigV2 {
                index: display_idx * 4 + border_idx,
                border,
                display_internal_id: display.internal_id.clone(),
                len: 20 + border_idx * 5,
                led_type: if border_idx % 2 == 0 {
                    LedType::WS2812B
                } else {
                    LedType::SK6812
                },
                reversed: border_idx % 2 == 1,
            });
        }
    }

    // 创建配置组并生成mappers
    let mut config = LedStripConfigGroupV2 {
        version: 2,
        display_config,
        strips,
        mappers: Vec::new(),
        color_calibration: ColorCalibration::new(),
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    };

    config.generate_mappers();

    let generation_time = start_time.elapsed();
    println!("   📊 生成配置耗时: {:?}", generation_time);
    println!("   📺 显示器数量: {}", config.display_config.displays.len());
    println!("   💡 灯带数量: {}", config.strips.len());
    println!("   🗺️ 映射器数量: {}", config.mappers.len());

    // 测试序列化性能
    let serialize_start = std::time::Instant::now();
    let _serialized = toml::to_string_pretty(&config).expect("序列化失败");
    let serialize_time = serialize_start.elapsed();
    println!("   💾 序列化耗时: {:?}", serialize_time);

    // 验证性能要求（应该在合理时间内完成）
    assert!(generation_time.as_millis() < 100, "配置生成耗时过长");
    assert!(serialize_time.as_millis() < 50, "序列化耗时过长");

    println!("✅ 性能测试通过！");
}
