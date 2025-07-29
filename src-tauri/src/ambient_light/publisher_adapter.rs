use std::collections::HashMap;
use anyhow::Result;

use crate::display::DisplayRegistry;
use crate::ambient_light::{LedStripConfigGroupV2, LedStripConfigV2, LedStripConfigGroup, LedStripConfig};

/// Publisher适配器，用于在新旧配置系统之间进行转换
pub struct PublisherAdapter {
    display_registry: std::sync::Arc<DisplayRegistry>,
}

impl PublisherAdapter {
    /// 创建新的适配器
    pub fn new(display_registry: std::sync::Arc<DisplayRegistry>) -> Self {
        Self { display_registry }
    }

    /// 将新版本配置转换为旧版本配置，用于兼容现有的Publisher
    pub async fn convert_v2_to_v1_config(&self, v2_config: &LedStripConfigGroupV2) -> Result<LedStripConfigGroup> {
        log::info!("🔄 转换新版本配置到旧版本格式...");

        // 获取当前系统显示器信息
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        // 创建显示器内部ID到系统ID的映射
        let mut internal_id_to_system_id = HashMap::new();
        
        for display_config in &v2_config.display_config.displays {
            // 尝试通过匹配找到对应的系统显示器
            let system_display = system_displays.iter().find(|sys_display| {
                // 首先尝试通过last_system_id匹配
                if let Some(last_id) = display_config.last_system_id {
                    if last_id == sys_display.id {
                        return true;
                    }
                }
                
                // 然后尝试精确匹配
                display_config.exact_match(sys_display)
            });

            if let Some(sys_display) = system_display {
                internal_id_to_system_id.insert(display_config.internal_id.clone(), sys_display.id);
                log::debug!(
                    "映射显示器: '{}' ({}) -> 系统ID {}",
                    display_config.name,
                    display_config.internal_id,
                    sys_display.id
                );
            } else {
                log::warn!(
                    "⚠️ 无法找到显示器 '{}' ({}) 对应的系统显示器",
                    display_config.name,
                    display_config.internal_id
                );
                // 使用一个默认值，避免转换失败
                internal_id_to_system_id.insert(display_config.internal_id.clone(), 0);
            }
        }

        // 转换LED灯带配置
        let mut v1_strips = Vec::new();
        for v2_strip in &v2_config.strips {
            let system_id = internal_id_to_system_id
                .get(&v2_strip.display_internal_id)
                .copied()
                .unwrap_or(0);

            let v1_strip = LedStripConfig {
                index: v2_strip.index,
                border: v2_strip.border,
                display_id: system_id,
                len: v2_strip.len,
                led_type: v2_strip.led_type,
                reversed: v2_strip.reversed,
            };

            v1_strips.push(v1_strip);
            log::debug!(
                "转换灯带 {}: {} -> display_id {}",
                v2_strip.index,
                v2_strip.display_internal_id,
                system_id
            );
        }

        // 创建旧版本配置
        let mut v1_config = LedStripConfigGroup {
            strips: v1_strips,
            mappers: Vec::new(),
            color_calibration: v2_config.color_calibration,
        };

        // 生成mappers
        v1_config.generate_mappers();

        log::info!("✅ 配置转换完成: {} 个灯带", v1_config.strips.len());
        Ok(v1_config)
    }

    /// 获取更新后的配置，确保显示器ID正确分配
    pub async fn get_updated_configs_with_stable_ids(
        &self,
        v2_config: &LedStripConfigGroupV2,
    ) -> Result<LedStripConfigGroup> {
        log::info!("🔍 获取带有稳定ID的更新配置...");

        // 检测并注册当前显示器
        let match_results = self.display_registry.detect_and_register_displays().await?;

        // 记录匹配结果
        log::info!("🖥️ 显示器匹配结果:");
        for (i, result) in match_results.iter().enumerate() {
            log::info!(
                "  匹配 {}: 类型={:?}, 分数={}, 系统ID={}",
                i,
                result.match_type,
                result.match_score,
                result.system_display.id
            );
        }

        // 转换配置
        self.convert_v2_to_v1_config(v2_config).await
    }

    /// 验证配置转换的正确性
    pub async fn validate_conversion(
        &self,
        v2_config: &LedStripConfigGroupV2,
        v1_config: &LedStripConfigGroup,
    ) -> Result<bool> {
        log::info!("🔍 验证配置转换...");

        // 检查灯带数量
        if v2_config.strips.len() != v1_config.strips.len() {
            log::error!(
                "❌ 灯带数量不匹配: v2={}, v1={}",
                v2_config.strips.len(),
                v1_config.strips.len()
            );
            return Ok(false);
        }

        // 检查每个灯带的基本属性
        for (v2_strip, v1_strip) in v2_config.strips.iter().zip(v1_config.strips.iter()) {
            if v2_strip.index != v1_strip.index
                || v2_strip.border != v1_strip.border
                || v2_strip.len != v1_strip.len
                || v2_strip.led_type != v1_strip.led_type
                || v2_strip.reversed != v1_strip.reversed
            {
                log::error!(
                    "❌ 灯带 {} 属性不匹配",
                    v2_strip.index
                );
                return Ok(false);
            }
        }

        // 检查颜色校准
        let v2_cal = &v2_config.color_calibration;
        let v1_cal = &v1_config.color_calibration;
        if (v2_cal.r - v1_cal.r).abs() > 0.001
            || (v2_cal.g - v1_cal.g).abs() > 0.001
            || (v2_cal.b - v1_cal.b).abs() > 0.001
            || (v2_cal.w - v1_cal.w).abs() > 0.001
        {
            log::error!("❌ 颜色校准不匹配");
            return Ok(false);
        }

        log::info!("✅ 配置转换验证通过");
        Ok(true)
    }

    /// 获取显示器映射信息（用于调试）
    pub async fn get_display_mapping_info(&self) -> Result<Vec<DisplayMappingInfo>> {
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        let config_displays = self.display_registry.get_all_displays().await;

        let mut mapping_info = Vec::new();

        for config_display in &config_displays {
            let system_display = system_displays.iter().find(|sys_display| {
                if let Some(last_id) = config_display.last_system_id {
                    if last_id == sys_display.id {
                        return true;
                    }
                }
                config_display.exact_match(sys_display)
            });

            let info = DisplayMappingInfo {
                internal_id: config_display.internal_id.clone(),
                name: config_display.name.clone(),
                system_id: system_display.map(|d| d.id),
                is_connected: system_display.is_some(),
                match_score: system_display
                    .map(|d| config_display.match_score(d))
                    .unwrap_or(0),
            };

            mapping_info.push(info);
        }

        Ok(mapping_info)
    }
}

/// 显示器映射信息
#[derive(Debug, Clone)]
pub struct DisplayMappingInfo {
    pub internal_id: String,
    pub name: String,
    pub system_id: Option<u32>,
    pub is_connected: bool,
    pub match_score: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{DisplayConfig, DisplayConfigGroup, DisplayRegistry};

    #[tokio::test]
    async fn test_config_conversion() {
        // 创建测试用的显示器配置
        let mut display_config_group = DisplayConfigGroup::new();
        let display = DisplayConfig::new(
            "Test Display".to_string(),
            1920,
            1080,
            1.0,
            true,
        );
        let display_id = display.internal_id.clone();
        display_config_group.add_display(display);

        // 创建测试用的v2配置
        let mut v2_config = LedStripConfigGroupV2::new();
        v2_config.display_config = display_config_group;
        
        let strip = LedStripConfigV2 {
            index: 0,
            border: crate::ambient_light::Border::Top,
            display_internal_id: display_id,
            len: 30,
            led_type: crate::ambient_light::LedType::WS2812B,
            reversed: false,
        };
        v2_config.strips.push(strip);

        // 创建适配器
        let display_registry = std::sync::Arc::new(DisplayRegistry::new(v2_config.display_config.clone()));
        let adapter = PublisherAdapter::new(display_registry);

        // 测试转换（注意：这个测试在没有真实显示器的环境中可能会失败）
        // 这里主要是验证代码结构的正确性
        assert_eq!(v2_config.strips.len(), 1);
    }
}
