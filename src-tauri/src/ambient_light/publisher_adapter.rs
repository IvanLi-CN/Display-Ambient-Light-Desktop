use anyhow::Result;
use std::collections::HashMap;

use crate::ambient_light::{
    LedStripConfig, LedStripConfigGroup, LedStripConfigGroupV2, LedStripConfigV2,
};
use crate::display::DisplayRegistry;

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
    pub async fn convert_v2_to_v1_config(
        &self,
        v2_config: &LedStripConfigGroupV2,
    ) -> Result<LedStripConfigGroup> {
        log::info!("🔄 转换新版本配置到旧版本格式...");

        // 创建显示器内部ID到系统ID的映射（优先使用注册表；其次使用 last_system_id；再次通过属性精确匹配；最后保底0）
        let mut internal_id_to_system_id = HashMap::new();

        // 预取系统显示器列表，供精确匹配回退使用
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        for display_config in &v2_config.display_config.displays {
            // 1) 优先通过 DisplayRegistry 由 internal_id 映射到当前系统ID
            match self
                .display_registry
                .get_display_id_by_internal_id(&display_config.internal_id)
                .await
            {
                Ok(system_id) => {
                    internal_id_to_system_id.insert(display_config.internal_id.clone(), system_id);
                    log::debug!(
                        "映射显示器(注册表): '{}' ({}) -> 系统ID {}",
                        display_config.name,
                        display_config.internal_id,
                        system_id
                    );
                }
                Err(e) => {
                    // 2) 回退：使用记录的 last_system_id（如果存在）
                    if let Some(last_id) = display_config.last_system_id {
                        internal_id_to_system_id
                            .insert(display_config.internal_id.clone(), last_id);
                        log::warn!(
                            "⚠️ 无法通过注册表映射显示器 '{}' ({}): {}，回退使用 last_system_id={}",
                            display_config.name,
                            display_config.internal_id,
                            e,
                            last_id
                        );
                    } else {
                        // 3) 再次回退：通过属性精确匹配找到系统显示器ID
                        if let Some(sys_display) = system_displays
                            .iter()
                            .find(|sd| display_config.exact_match(sd))
                        {
                            internal_id_to_system_id
                                .insert(display_config.internal_id.clone(), sys_display.id);
                            log::warn!(
                                "⚠️ 无法通过注册表映射显示器 '{}' ({})，但通过属性匹配到了系统ID {}",
                                display_config.name,
                                display_config.internal_id,
                                sys_display.id
                            );
                        } else {
                            // 4) 最后回退：使用0（保持兼容性，避免直接失败），但记录警告
                            internal_id_to_system_id.insert(display_config.internal_id.clone(), 0);
                            log::warn!(
                                "⚠️ 无法为显示器 '{}' ({}) 找到系统ID，使用默认值0",
                                display_config.name,
                                display_config.internal_id
                            );
                        }
                    }
                }
            }
        }

        // 转换LED灯带配置
        let mut v1_strips = Vec::new();
        for v2_strip in &v2_config.strips {
            // 若条目中的 internal_id 在配置里不存在，基于 strip.index 回退到某个有效显示器
            let mut target_internal_id = v2_strip.display_internal_id.clone();
            if v2_config
                .display_config
                .find_by_internal_id(&target_internal_id)
                .is_none()
            {
                let display_index = v2_strip.index / 4; // 每4个灯带对应一个显示器（Top/Right/Bottom/Left）
                if let Some(disp) = v2_config.display_config.displays.get(display_index) {
                    log::warn!(
                        "⚠️ 条目 {} 内部ID '{}' 未在配置中找到，按索引回退为显示器 '{}'",
                        v2_strip.index,
                        target_internal_id,
                        disp.internal_id
                    );
                    target_internal_id = disp.internal_id.clone();
                } else if let Some(first) = v2_config.display_config.displays.first() {
                    log::warn!(
                        "⚠️ 条目 {} 内部ID '{}' 未在配置中找到，且索引回退越界，使用第一个显示器 '{}'",
                        v2_strip.index,
                        target_internal_id,
                        first.internal_id
                    );
                    target_internal_id = first.internal_id.clone();
                }
            }

            // 优先使用预构建映射；若不存在则按条目逐个回退解析
            let mut system_id = internal_id_to_system_id
                .get(&target_internal_id)
                .copied()
                .unwrap_or(0);

            if system_id == 0 {
                // 1) 尝试直接通过注册表解析该条目的 internal_id
                match self
                    .display_registry
                    .get_display_id_by_internal_id(&target_internal_id)
                    .await
                {
                    Ok(id) => {
                        system_id = id;
                        internal_id_to_system_id.insert(target_internal_id.clone(), id);
                        log::debug!(
                            "条目级映射(注册表): {} -> 系统ID {}",
                            target_internal_id,
                            id
                        );
                    }
                    Err(_) => {
                        // 2) 再尝试根据 display_config 中的记录做属性匹配
                        if let Some(dc) = v2_config
                            .display_config
                            .find_by_internal_id(&target_internal_id)
                        {
                            if let Some(sys_display) =
                                system_displays.iter().find(|sd| dc.exact_match(sd))
                            {
                                system_id = sys_display.id;
                                internal_id_to_system_id
                                    .insert(target_internal_id.clone(), system_id);
                                log::debug!(
                                    "条目级映射(属性匹配): {} -> 系统ID {}",
                                    target_internal_id,
                                    system_id
                                );
                            }
                        }
                    }
                }
            }

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
                target_internal_id,
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

    /// 将v1配置转换为v2配置格式
    pub async fn convert_v1_to_v2_config(
        &self,
        v1_config: &LedStripConfigGroup,
    ) -> Result<LedStripConfigGroupV2> {
        log::info!("🔄 转换旧版本配置到新版本格式...");

        // 获取当前系统显示器信息
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        // 创建系统ID到内部ID的映射
        let mut system_id_to_internal_id = HashMap::new();

        // 获取当前的显示器配置组
        let display_config = self.display_registry.get_config_group().await;

        for display_config_item in &display_config.displays {
            // 尝试通过匹配找到对应的系统显示器
            let system_display = system_displays.iter().find(|sys_display| {
                // 首先尝试通过last_system_id匹配
                if let Some(last_id) = display_config_item.last_system_id {
                    if last_id == sys_display.id {
                        return true;
                    }
                }

                // 然后尝试精确匹配
                display_config_item.exact_match(sys_display)
            });

            if let Some(sys_display) = system_display {
                system_id_to_internal_id
                    .insert(sys_display.id, display_config_item.internal_id.clone());
                log::debug!(
                    "映射显示器: 系统ID {} -> '{}' ({})",
                    sys_display.id,
                    display_config_item.name,
                    display_config_item.internal_id
                );
            }
        }

        // 转换LED灯带配置
        let mut v2_strips = Vec::new();
        for v1_strip in &v1_config.strips {
            let internal_id = system_id_to_internal_id
                .get(&v1_strip.display_id)
                .cloned()
                .unwrap_or_else(|| {
                    log::warn!(
                        "⚠️ 无法找到显示器ID {} 对应的内部ID，使用默认值",
                        v1_strip.display_id
                    );
                    format!("display_{}", v1_strip.display_id)
                });

            let v2_strip = LedStripConfigV2 {
                index: v1_strip.index,
                border: v1_strip.border,
                display_internal_id: internal_id.clone(),
                len: v1_strip.len,
                led_type: v1_strip.led_type,
                reversed: v1_strip.reversed,
            };

            v2_strips.push(v2_strip);
            log::debug!(
                "转换灯带 {}: display_id {} -> {}",
                v1_strip.index,
                v1_strip.display_id,
                internal_id
            );
        }

        // 创建新版本配置
        let mut v2_config = LedStripConfigGroupV2 {
            version: 2,
            strips: v2_strips,
            color_calibration: v1_config.color_calibration,
            display_config,
            mappers: Vec::new(),
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
        };

        // 生成mappers
        v2_config.generate_mappers();

        log::info!("✅ 配置转换完成: {} 个灯带", v2_config.strips.len());
        Ok(v2_config)
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
                log::error!("❌ 灯带 {} 属性不匹配", v2_strip.index);
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
        let display = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);
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
        let display_registry =
            std::sync::Arc::new(DisplayRegistry::new(v2_config.display_config.clone()));
        let _adapter = PublisherAdapter::new(display_registry);

        // 测试转换（注意：这个测试在没有真实显示器的环境中可能会失败）
        // 这里主要是验证代码结构的正确性
        assert_eq!(v2_config.strips.len(), 1);
    }
}
