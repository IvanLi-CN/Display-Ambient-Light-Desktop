use std::env::current_dir;
use std::path::PathBuf;
use anyhow::Result;
use dirs::config_dir;

use super::{DisplayConfig, DisplayConfigGroup};
use crate::ambient_light::{LedStripConfigGroup, LedStripConfigGroupV2, LedStripConfigV2};

/// 配置文件迁移器
/// 负责将旧版本的配置文件迁移到新版本
pub struct ConfigMigrator;

impl ConfigMigrator {
    /// 检查是否需要迁移
    pub async fn needs_migration() -> bool {
        let v2_config_path = Self::get_v2_config_path();
        let legacy_config_path = Self::get_legacy_config_path();

        // 如果新版本配置不存在，但旧版本配置存在，则需要迁移
        !v2_config_path.exists() && legacy_config_path.exists()
    }

    /// 执行完整的配置迁移
    pub async fn migrate_all_configs() -> Result<LedStripConfigGroupV2> {
        log::info!("🔄 开始配置迁移过程...");

        // 检查是否需要迁移
        if !Self::needs_migration().await {
            log::info!("✅ 无需迁移，直接读取新版本配置");
            return LedStripConfigGroupV2::read_config().await;
        }

        log::info!("📦 检测到旧版本配置，开始迁移...");

        // 1. 迁移显示器配置
        let display_config_group = Self::migrate_display_config().await?;

        // 2. 迁移LED灯带配置
        let led_config_group = Self::migrate_led_strip_config(display_config_group).await?;

        // 3. 保存新版本配置
        led_config_group.write_config().await?;

        // 4. 备份旧配置文件
        Self::backup_legacy_configs().await?;

        log::info!("✅ 配置迁移完成");
        Ok(led_config_group)
    }

    /// 迁移显示器配置
    async fn migrate_display_config() -> Result<DisplayConfigGroup> {
        log::info!("🖥️ 开始迁移显示器配置...");

        let mut display_config_group = DisplayConfigGroup::new();

        // 获取当前系统显示器信息
        match display_info::DisplayInfo::all() {
            Ok(system_displays) => {
                log::info!("检测到 {} 个系统显示器", system_displays.len());

                for display_info in &system_displays {
                    let display_config = DisplayConfig::from_display_info(display_info);
                    log::info!(
                        "创建显示器配置: '{}' ({}x{}, 主显示器: {})",
                        display_config.name,
                        display_config.width,
                        display_config.height,
                        display_config.is_primary
                    );
                    display_config_group.add_display(display_config);
                }
            }
            Err(e) => {
                log::warn!("⚠️ 无法检测系统显示器: {}，创建默认配置", e);

                // 创建默认显示器配置
                for i in 0..2 {
                    let display_config = DisplayConfig::new(
                        if i == 0 { "主显示器".to_string() } else { format!("显示器 {}", i + 1) },
                        1920,
                        1080,
                        1.0,
                        i == 0,
                    );
                    display_config_group.add_display(display_config);
                }
            }
        }

        log::info!("✅ 显示器配置迁移完成，共 {} 个显示器", display_config_group.displays.len());
        Ok(display_config_group)
    }

    /// 迁移LED灯带配置
    async fn migrate_led_strip_config(display_config_group: DisplayConfigGroup) -> Result<LedStripConfigGroupV2> {
        log::info!("💡 开始迁移LED灯带配置...");

        // 读取旧版本LED配置
        let legacy_config = LedStripConfigGroup::read_config().await?;
        log::info!("读取到 {} 个旧版本灯带配置", legacy_config.strips.len());

        // 创建新版本配置
        let mut new_config = LedStripConfigGroupV2::new();
        new_config.display_config = display_config_group;
        new_config.color_calibration = legacy_config.color_calibration;

        // 迁移灯带配置
        for old_strip in &legacy_config.strips {
            let display_internal_id = Self::map_display_id_to_internal_id(
                old_strip.display_id,
                old_strip.index,
                &new_config.display_config,
            );

            let new_strip = LedStripConfigV2 {
                index: old_strip.index,
                border: old_strip.border,
                display_internal_id,
                len: old_strip.len,
                led_type: old_strip.led_type,
                reversed: old_strip.reversed,
            };

            log::debug!(
                "迁移灯带 {}: display_id {} -> internal_id {}",
                old_strip.index,
                old_strip.display_id,
                new_strip.display_internal_id
            );

            new_config.strips.push(new_strip);
        }

        // 生成mappers
        new_config.generate_mappers();

        log::info!("✅ LED灯带配置迁移完成，共 {} 个灯带", new_config.strips.len());
        Ok(new_config)
    }

    /// 将旧的display_id映射到新的internal_id
    pub fn map_display_id_to_internal_id(
        old_display_id: u32,
        strip_index: usize,
        display_config_group: &DisplayConfigGroup,
    ) -> String {
        if old_display_id == 0 {
            // 如果是0，根据灯带索引分配（每4个灯带对应一个显示器）
            let display_index = strip_index / 4;
            if display_index < display_config_group.displays.len() {
                display_config_group.displays[display_index].internal_id.clone()
            } else {
                // 如果索引超出范围，使用第一个显示器
                display_config_group.displays.first()
                    .map(|d| d.internal_id.clone())
                    .unwrap_or_else(|| "default_display".to_string())
            }
        } else {
            // 根据系统ID查找对应的显示器配置
            display_config_group
                .displays
                .iter()
                .find(|d| d.last_system_id == Some(old_display_id))
                .map(|d| d.internal_id.clone())
                .unwrap_or_else(|| {
                    // 如果找不到，使用第一个显示器
                    display_config_group.displays.first()
                        .map(|d| d.internal_id.clone())
                        .unwrap_or_else(|| "default_display".to_string())
                })
        }
    }

    /// 备份旧配置文件
    async fn backup_legacy_configs() -> Result<()> {
        log::info!("📦 备份旧配置文件...");

        let legacy_config_path = Self::get_legacy_config_path();
        if legacy_config_path.exists() {
            let backup_path = legacy_config_path.with_extension("toml.backup");
            tokio::fs::copy(&legacy_config_path, &backup_path).await?;
            log::info!("✅ 旧LED配置已备份到: {:?}", backup_path);
        }

        // 备份显示器配置（如果存在）
        let display_config_path = Self::get_legacy_display_config_path();
        if display_config_path.exists() {
            let backup_path = display_config_path.with_extension("toml.backup");
            tokio::fs::copy(&display_config_path, &backup_path).await?;
            log::info!("✅ 旧显示器配置已备份到: {:?}", backup_path);
        }

        Ok(())
    }

    /// 获取新版本配置文件路径
    fn get_v2_config_path() -> PathBuf {
        config_dir()
            .unwrap_or(current_dir().unwrap())
            .join("cc.ivanli.ambient_light/config_v2.toml")
    }

    /// 获取旧版本LED配置文件路径
    fn get_legacy_config_path() -> PathBuf {
        config_dir()
            .unwrap_or(current_dir().unwrap())
            .join("cc.ivanli.ambient_light/led_strip_config.toml")
    }

    /// 获取旧版本显示器配置文件路径
    fn get_legacy_display_config_path() -> PathBuf {
        config_dir()
            .unwrap_or(current_dir().unwrap())
            .join("cc.ivanli.ambient_light/displays.toml")
    }

    /// 清理旧配置文件（可选）
    pub async fn cleanup_legacy_configs() -> Result<()> {
        log::info!("🧹 清理旧配置文件...");

        let legacy_config_path = Self::get_legacy_config_path();
        if legacy_config_path.exists() {
            tokio::fs::remove_file(&legacy_config_path).await?;
            log::info!("🗑️ 已删除旧LED配置文件");
        }

        let display_config_path = Self::get_legacy_display_config_path();
        if display_config_path.exists() {
            tokio::fs::remove_file(&display_config_path).await?;
            log::info!("🗑️ 已删除旧显示器配置文件");
        }

        Ok(())
    }

    /// 验证迁移结果
    pub async fn validate_migration() -> Result<bool> {
        log::info!("🔍 验证迁移结果...");

        // 检查新配置文件是否存在
        let v2_config_path = Self::get_v2_config_path();
        if !v2_config_path.exists() {
            log::error!("❌ 新配置文件不存在");
            return Ok(false);
        }

        // 尝试读取新配置文件
        match LedStripConfigGroupV2::read_config().await {
            Ok(config) => {
                log::info!("✅ 新配置文件读取成功");
                log::info!("  - 显示器数量: {}", config.display_config.displays.len());
                log::info!("  - 灯带数量: {}", config.strips.len());
                log::info!("  - 配置版本: {}", config.version);
                Ok(true)
            }
            Err(e) => {
                log::error!("❌ 新配置文件读取失败: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_display_id_mapping() {
        let mut display_config_group = DisplayConfigGroup::new();
        
        // 添加两个显示器配置
        let display1 = DisplayConfig::new("Display 1".to_string(), 1920, 1080, 1.0, true);
        let display2 = DisplayConfig::new("Display 2".to_string(), 1920, 1080, 1.0, false);
        
        let id1 = display1.internal_id.clone();
        let id2 = display2.internal_id.clone();
        
        display_config_group.add_display(display1);
        display_config_group.add_display(display2);

        // 测试基于索引的映射
        let mapped_id = ConfigMigrator::map_display_id_to_internal_id(0, 0, &display_config_group);
        assert_eq!(mapped_id, id1);

        let mapped_id = ConfigMigrator::map_display_id_to_internal_id(0, 4, &display_config_group);
        assert_eq!(mapped_id, id2);
    }
}
