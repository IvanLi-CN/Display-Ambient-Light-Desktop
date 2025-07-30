use anyhow::Result;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{OnceCell, RwLock};

use super::{DisplayConfig, DisplayConfigGroup, DisplayMatcher, MatchResult, MatchType};

/// 显示器注册管理器
/// 负责管理显示器的注册、查找、更新等操作
pub struct DisplayRegistry {
    /// 显示器配置组
    config_group: Arc<RwLock<DisplayConfigGroup>>,
    /// 显示器匹配器
    matcher: Arc<RwLock<DisplayMatcher>>,
}

impl DisplayRegistry {
    /// 获取全局显示器注册管理器实例
    pub async fn global() -> Result<&'static Self> {
        static DISPLAY_REGISTRY: OnceCell<DisplayRegistry> = OnceCell::const_new();

        DISPLAY_REGISTRY
            .get_or_try_init(|| async {
                // 创建默认的显示器配置组
                let config_group = DisplayConfigGroup::new();
                let registry = Self::new(config_group);

                // 检测并注册当前显示器
                if let Err(e) = registry.detect_and_register_displays().await {
                    log::warn!("Failed to detect displays during initialization: {}", e);
                }

                Ok(registry)
            })
            .await
    }

    /// 创建新的显示器注册管理器
    pub fn new(config_group: DisplayConfigGroup) -> Self {
        let matcher = DisplayMatcher::new(config_group.clone());
        Self {
            config_group: Arc::new(RwLock::new(config_group)),
            matcher: Arc::new(RwLock::new(matcher)),
        }
    }

    /// 检测并注册当前系统中的所有显示器
    pub async fn detect_and_register_displays(&self) -> Result<Vec<MatchResult>> {
        log::info!("🔍 开始检测系统显示器...");

        // 获取系统显示器信息
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        log::info!("🖥️ 检测到 {} 个系统显示器", system_displays.len());
        for (i, display) in system_displays.iter().enumerate() {
            log::info!(
                "  显示器 {}: ID={}, {}x{}, 位置=({}, {}), 主显示器={}, 缩放={}",
                i,
                display.id,
                display.width,
                display.height,
                display.x,
                display.y,
                display.is_primary,
                display.scale_factor
            );
        }

        // 使用匹配器进行匹配
        let matcher = self.matcher.read().await;
        let match_results = matcher.match_displays(&system_displays)?;
        drop(matcher);

        // 处理匹配结果
        let mut config_group = self.config_group.write().await;

        for match_result in &match_results {
            match match_result.match_type {
                MatchType::Exact | MatchType::Partial | MatchType::Position => {
                    // 更新现有配置的检测信息
                    if let Some(config) =
                        config_group.find_by_internal_id_mut(&match_result.config_internal_id)
                    {
                        config.update_last_detected(&match_result.system_display);
                        log::info!("✅ 更新显示器配置 '{}' 的检测信息", config.name);
                    }
                }
                MatchType::New => {
                    // 为新显示器创建配置
                    let new_config = DisplayConfig::from_display_info(&match_result.system_display);
                    log::info!(
                        "🆕 为新显示器创建配置: '{}' ({}x{})",
                        new_config.name,
                        new_config.width,
                        new_config.height
                    );
                    config_group.add_display(new_config);
                }
            }
        }

        // 更新匹配器的配置组
        let mut matcher = self.matcher.write().await;
        matcher.update_config_group(config_group.clone());
        drop(matcher);

        log::info!("✅ 显示器检测和注册完成");
        Ok(match_results)
    }

    /// 根据内部ID查找显示器配置
    pub async fn find_display_by_internal_id(&self, internal_id: &str) -> Option<DisplayConfig> {
        let config_group = self.config_group.read().await;
        config_group.find_by_internal_id(internal_id).cloned()
    }

    /// 根据系统ID查找显示器配置
    pub async fn find_display_by_system_id(&self, system_id: u32) -> Option<DisplayConfig> {
        let config_group = self.config_group.read().await;
        config_group
            .displays
            .iter()
            .find(|d| d.last_system_id == Some(system_id))
            .cloned()
    }

    /// 获取所有显示器配置
    pub async fn get_all_displays(&self) -> Vec<DisplayConfig> {
        let config_group = self.config_group.read().await;
        config_group.displays.clone()
    }

    /// 通过系统ID获取内部ID
    pub async fn get_internal_id_by_display_id(&self, system_id: u32) -> Result<String> {
        // 获取当前系统显示器信息
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        // 找到对应的系统显示器
        let system_display = system_displays
            .iter()
            .find(|d| d.id == system_id)
            .ok_or_else(|| anyhow::anyhow!("System display with ID {} not found", system_id))?;

        // 在配置中查找匹配的显示器
        let config_group = self.config_group.read().await;
        for display_config in &config_group.displays {
            // 首先尝试通过last_system_id匹配
            if let Some(last_id) = display_config.last_system_id {
                if last_id == system_id {
                    return Ok(display_config.internal_id.clone());
                }
            }

            // 然后尝试精确匹配
            if display_config.exact_match(system_display) {
                return Ok(display_config.internal_id.clone());
            }
        }

        Err(anyhow::anyhow!(
            "No display config found for system ID {}",
            system_id
        ))
    }

    /// 通过内部ID获取系统ID
    pub async fn get_display_id_by_internal_id(&self, internal_id: &str) -> Result<u32> {
        // 获取当前系统显示器信息
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        // 找到对应的显示器配置
        let config_group = self.config_group.read().await;
        let display_config = config_group
            .find_by_internal_id(internal_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Display config with internal ID '{}' not found",
                    internal_id
                )
            })?;

        // 在系统显示器中查找匹配的显示器
        for system_display in &system_displays {
            // 首先尝试通过last_system_id匹配
            if let Some(last_id) = display_config.last_system_id {
                if last_id == system_display.id {
                    return Ok(system_display.id);
                }
            }

            // 然后尝试精确匹配
            if display_config.exact_match(system_display) {
                return Ok(system_display.id);
            }
        }

        Err(anyhow::anyhow!(
            "No system display found for internal ID '{}'",
            internal_id
        ))
    }

    /// 更新显示器配置
    pub async fn update_display(&self, display: DisplayConfig) -> Result<bool> {
        let mut config_group = self.config_group.write().await;
        let updated = config_group.update_display(display);

        if updated {
            // 更新匹配器的配置组
            let mut matcher = self.matcher.write().await;
            matcher.update_config_group(config_group.clone());
        }

        Ok(updated)
    }

    /// 添加新的显示器配置
    pub async fn add_display(&self, display: DisplayConfig) -> Result<()> {
        let mut config_group = self.config_group.write().await;
        config_group.add_display(display);

        // 更新匹配器的配置组
        let mut matcher = self.matcher.write().await;
        matcher.update_config_group(config_group.clone());

        Ok(())
    }

    /// 移除显示器配置
    pub async fn remove_display(&self, internal_id: &str) -> Result<bool> {
        let mut config_group = self.config_group.write().await;
        let removed = config_group.remove_display(internal_id);

        if removed {
            // 更新匹配器的配置组
            let mut matcher = self.matcher.write().await;
            matcher.update_config_group(config_group.clone());
        }

        Ok(removed)
    }

    /// 获取配置组的克隆
    pub async fn get_config_group(&self) -> DisplayConfigGroup {
        let config_group = self.config_group.read().await;
        config_group.clone()
    }

    /// 更新整个配置组
    pub async fn update_config_group(&self, new_config_group: DisplayConfigGroup) -> Result<()> {
        let mut config_group = self.config_group.write().await;
        *config_group = new_config_group;

        // 更新匹配器的配置组
        let mut matcher = self.matcher.write().await;
        matcher.update_config_group(config_group.clone());

        Ok(())
    }

    /// 检查显示器配置是否需要更新
    /// 返回需要更新的显示器列表
    pub async fn check_for_updates(&self) -> Result<Vec<String>> {
        let system_displays = display_info::DisplayInfo::all()
            .map_err(|e| anyhow::anyhow!("Failed to get display info: {}", e))?;

        let config_group = self.config_group.read().await;
        let mut outdated_displays = Vec::new();

        for config_display in &config_group.displays {
            // 检查是否有对应的系统显示器
            let system_match = system_displays.iter().find(|sys_display| {
                config_display.last_system_id == Some(sys_display.id)
                    || config_display.exact_match(sys_display)
            });

            if system_match.is_none() {
                // 显示器可能已断开连接
                outdated_displays.push(config_display.internal_id.clone());
                log::warn!("⚠️ 显示器配置 '{}' 可能已断开连接", config_display.name);
            } else if let Some(sys_display) = system_match {
                // 检查属性是否有变化
                if !config_display.exact_match(sys_display) {
                    outdated_displays.push(config_display.internal_id.clone());
                    log::info!(
                        "🔄 显示器配置 '{}' 属性已变化，需要更新",
                        config_display.name
                    );
                }
            }
        }

        Ok(outdated_displays)
    }

    /// 获取显示器统计信息
    pub async fn get_display_stats(&self) -> DisplayStats {
        let config_group = self.config_group.read().await;
        let total_displays = config_group.displays.len();

        let primary_displays = config_group
            .displays
            .iter()
            .filter(|d| d.is_primary)
            .count();

        let displays_with_last_detection = config_group
            .displays
            .iter()
            .filter(|d| d.last_detected_at.is_some())
            .count();

        let now = SystemTime::now();
        let recent_detections = config_group
            .displays
            .iter()
            .filter(|d| {
                if let Some(last_detected) = d.last_detected_at {
                    if let Ok(duration) = now.duration_since(last_detected) {
                        return duration.as_secs() < 300; // 5分钟内
                    }
                }
                false
            })
            .count();

        DisplayStats {
            total_displays,
            primary_displays,
            displays_with_last_detection,
            recent_detections,
        }
    }
}

/// 显示器统计信息
#[derive(Debug, Clone)]
pub struct DisplayStats {
    /// 总显示器数量
    pub total_displays: usize,
    /// 主显示器数量
    pub primary_displays: usize,
    /// 有检测记录的显示器数量
    pub displays_with_last_detection: usize,
    /// 最近检测到的显示器数量（5分钟内）
    pub recent_detections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_display_registry_creation() {
        let config_group = DisplayConfigGroup::new();
        let registry = DisplayRegistry::new(config_group);

        let stats = registry.get_display_stats().await;
        assert_eq!(stats.total_displays, 0);
    }

    #[tokio::test]
    async fn test_add_and_find_display() {
        let config_group = DisplayConfigGroup::new();
        let registry = DisplayRegistry::new(config_group);

        let display = DisplayConfig::new("Test Display".to_string(), 1920, 1080, 1.0, true);
        let internal_id = display.internal_id.clone();

        registry.add_display(display).await.unwrap();

        let found = registry.find_display_by_internal_id(&internal_id).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Display");

        let stats = registry.get_display_stats().await;
        assert_eq!(stats.total_displays, 1);
        assert_eq!(stats.primary_displays, 1);
    }
}
