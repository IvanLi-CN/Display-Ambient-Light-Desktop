use anyhow::Result;
use std::sync::Arc;
use tauri::async_runtime::RwLock;
use tokio::sync::OnceCell;

use crate::ambient_light::{ColorCalibration, LedStripConfigGroupV2, LedStripConfigV2};
use crate::display::DisplayRegistry;

/// 新版本的配置管理器，支持稳定的显示器ID系统
pub struct ConfigManagerV2 {
    /// LED灯带配置
    config: Arc<RwLock<LedStripConfigGroupV2>>,
    /// 显示器注册管理器
    display_registry: Arc<DisplayRegistry>,
    /// 配置更新通知
    config_update_sender: tokio::sync::watch::Sender<LedStripConfigGroupV2>,
}

impl ConfigManagerV2 {
    /// 获取全局配置管理器实例
    pub async fn global() -> &'static Self {
        static CONFIG_MANAGER_V2_GLOBAL: OnceCell<ConfigManagerV2> = OnceCell::const_new();
        CONFIG_MANAGER_V2_GLOBAL
            .get_or_init(|| async {
                log::info!("🏗️ [COLOR_CALIBRATION] Initializing ConfigManagerV2 global instance");

                // 直接尝试读取V2配置，不进行任何迁移
                match LedStripConfigGroupV2::read_config().await {
                    Ok(config) => {
                        log::info!(
                            "✅ [COLOR_CALIBRATION] Successfully loaded V2 config with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
                            config.color_calibration.r,
                            config.color_calibration.g,
                            config.color_calibration.b,
                            config.color_calibration.w
                        );
                        Self::create_from_config(config).await
                    }
                    Err(e) => {
                        log::warn!("⚠️ [COLOR_CALIBRATION] Unable to load V2 config: {}", e);
                        log::info!("🏗️ [COLOR_CALIBRATION] Creating default ConfigManagerV2 instance");
                        Self::create_default().await
                    }
                }
            })
            .await
    }

    /// 从配置创建管理器
    async fn create_from_config(config: LedStripConfigGroupV2) -> Self {
        log::info!(
            "🏗️ [COLOR_CALIBRATION] Creating ConfigManagerV2 from existing config with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
            config.color_calibration.r,
            config.color_calibration.g,
            config.color_calibration.b,
            config.color_calibration.w
        );

        let display_registry = Arc::new(DisplayRegistry::new(config.display_config.clone()));

        // 检测并注册当前显示器
        if let Err(e) = display_registry.detect_and_register_displays().await {
            log::warn!("⚠️ [COLOR_CALIBRATION] Display detection failed: {}", e);
        }

        let (config_update_sender, _) = tokio::sync::watch::channel(config.clone());

        log::info!(
            "✅ [COLOR_CALIBRATION] ConfigManagerV2 created from config with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
            config.color_calibration.r,
            config.color_calibration.g,
            config.color_calibration.b,
            config.color_calibration.w
        );

        Self {
            config: Arc::new(RwLock::new(config)),
            display_registry,
            config_update_sender,
        }
    }

    /// 创建默认配置管理器
    async fn create_default() -> Self {
        log::info!("🏗️ [COLOR_CALIBRATION] Creating default ConfigManagerV2");

        match LedStripConfigGroupV2::get_default_config().await {
            Ok(config) => {
                log::info!(
                    "✅ [COLOR_CALIBRATION] Successfully created default config with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
                    config.color_calibration.r,
                    config.color_calibration.g,
                    config.color_calibration.b,
                    config.color_calibration.w
                );
                Self::create_from_config(config).await
            }
            Err(e) => {
                log::error!(
                    "❌ [COLOR_CALIBRATION] Failed to create default config: {}",
                    e
                );
                log::info!("🆘 [COLOR_CALIBRATION] Creating minimal fallback config");

                // 创建最小配置
                let config = LedStripConfigGroupV2::new();
                log::info!(
                    "🆘 [COLOR_CALIBRATION] Fallback config created with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
                    config.color_calibration.r,
                    config.color_calibration.g,
                    config.color_calibration.b,
                    config.color_calibration.w
                );
                Self::create_from_config(config).await
            }
        }
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> LedStripConfigGroupV2 {
        self.config.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: LedStripConfigGroupV2) -> Result<()> {
        log::info!(
            "🔄 [COLOR_CALIBRATION] ConfigManagerV2::update_config called with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
            new_config.color_calibration.r,
            new_config.color_calibration.g,
            new_config.color_calibration.b,
            new_config.color_calibration.w
        );

        // 保存到文件
        log::info!("💾 [COLOR_CALIBRATION] Saving config to file...");
        new_config.write_config().await?;

        // 更新内存中的配置
        log::info!("🧠 [COLOR_CALIBRATION] Updating in-memory config...");
        {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        }

        // 更新显示器注册管理器
        log::info!("📺 [COLOR_CALIBRATION] Updating display registry...");
        self.display_registry
            .update_config_group(new_config.display_config.clone())
            .await?;

        // 发送更新通知
        log::info!("📡 [COLOR_CALIBRATION] Sending config update notification...");
        if let Err(e) = self.config_update_sender.send(new_config.clone()) {
            log::error!(
                "❌ [COLOR_CALIBRATION] Failed to send config update notification: {}",
                e
            );
        }

        // 通过适配器转换为v1格式并广播配置变化
        log::info!("🔄 [COLOR_CALIBRATION] Converting to v1 format for WebSocket broadcast...");
        let adapter = crate::ambient_light::PublisherAdapter::new(self.display_registry.clone());
        match adapter.convert_v2_to_v1_config(&new_config).await {
            Ok(v1_config) => {
                log::info!(
                    "📡 [COLOR_CALIBRATION] Broadcasting config change via WebSocket with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
                    v1_config.color_calibration.r,
                    v1_config.color_calibration.g,
                    v1_config.color_calibration.b,
                    v1_config.color_calibration.w
                );
                crate::websocket_events::publish_config_changed(&v1_config).await;
            }
            Err(e) => {
                log::error!(
                    "❌ [COLOR_CALIBRATION] Failed to convert v2 config to v1 for WebSocket broadcast: {}",
                    e
                );
            }
        }

        log::info!(
            "✅ [COLOR_CALIBRATION] ConfigManagerV2::update_config completed successfully with color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
            new_config.color_calibration.r,
            new_config.color_calibration.g,
            new_config.color_calibration.b,
            new_config.color_calibration.w
        );

        Ok(())
    }

    /// 重新加载配置
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = LedStripConfigGroupV2::read_config().await?;

        {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        }

        // 更新显示器注册管理器
        self.display_registry
            .update_config_group(new_config.display_config.clone())
            .await?;

        Ok(())
    }

    /// 获取显示器注册管理器
    pub fn get_display_registry(&self) -> Arc<DisplayRegistry> {
        self.display_registry.clone()
    }

    /// 获取配置更新接收器
    pub fn subscribe_config_updates(&self) -> tokio::sync::watch::Receiver<LedStripConfigGroupV2> {
        self.config_update_sender.subscribe()
    }

    /// 添加LED灯带
    pub async fn add_led_strip(&self, strip: LedStripConfigV2) -> Result<()> {
        let mut config = self.get_config().await;
        config.strips.push(strip);
        config.generate_mappers();
        self.update_config(config).await
    }

    /// 更新LED灯带
    pub async fn update_led_strip(&self, index: usize, strip: LedStripConfigV2) -> Result<()> {
        let mut config = self.get_config().await;

        if let Some(existing_strip) = config.strips.iter_mut().find(|s| s.index == index) {
            *existing_strip = strip;
            config.generate_mappers();
            self.update_config(config).await
        } else {
            Err(anyhow::anyhow!("LED灯带索引 {} 不存在", index))
        }
    }

    /// 删除LED灯带
    pub async fn remove_led_strip(&self, index: usize) -> Result<()> {
        let mut config = self.get_config().await;

        let initial_len = config.strips.len();
        config.strips.retain(|s| s.index != index);

        if config.strips.len() < initial_len {
            config.generate_mappers();
            self.update_config(config).await
        } else {
            Err(anyhow::anyhow!("LED灯带索引 {} 不存在", index))
        }
    }

    /// 更新颜色校准
    pub async fn update_color_calibration(&self, calibration: ColorCalibration) -> Result<()> {
        let mut config = self.get_config().await;

        // 记录旧的配置值
        let old_calibration = config.color_calibration;
        log::info!("🎨 [COLOR_CALIBRATION] ConfigManagerV2::update_color_calibration called");
        log::info!(
            "🔄 [COLOR_CALIBRATION] Old calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
            old_calibration.r,
            old_calibration.g,
            old_calibration.b,
            old_calibration.w
        );
        log::info!(
            "🆕 [COLOR_CALIBRATION] New calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
            calibration.r,
            calibration.g,
            calibration.b,
            calibration.w
        );

        // 检查是否有实际变化
        if old_calibration.r == calibration.r
            && old_calibration.g == calibration.g
            && old_calibration.b == calibration.b
            && old_calibration.w == calibration.w
        {
            log::info!("ℹ️ [COLOR_CALIBRATION] No changes detected, skipping update");
            return Ok(());
        }

        config.color_calibration = calibration;

        match self.update_config(config).await {
            Ok(_) => {
                log::info!(
                    "✅ [COLOR_CALIBRATION] Successfully updated color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}",
                    calibration.r,
                    calibration.g,
                    calibration.b,
                    calibration.w
                );
                Ok(())
            }
            Err(e) => {
                log::error!(
                    "❌ [COLOR_CALIBRATION] Failed to update color calibration: r={:.3}, g={:.3}, b={:.3}, w={:.3}, error: {}",
                    calibration.r,
                    calibration.g,
                    calibration.b,
                    calibration.w,
                    e
                );
                Err(e)
            }
        }
    }

    /// 获取指定显示器的LED灯带
    pub async fn get_strips_for_display(&self, display_internal_id: &str) -> Vec<LedStripConfigV2> {
        let config = self.config.read().await;
        config
            .strips
            .iter()
            .filter(|s| s.display_internal_id == display_internal_id)
            .cloned()
            .collect()
    }

    /// 检查显示器变化并更新配置
    pub async fn check_and_update_displays(&self) -> Result<bool> {
        log::info!("🔍 检查显示器变化...");

        let match_results = self.display_registry.detect_and_register_displays().await?;
        let config_changed = false;

        // 检查是否有新显示器，但不再自动创建灯带配置
        for match_result in &match_results {
            if matches!(match_result.match_type, crate::display::MatchType::New) {
                log::info!("🆕 检测到新显示器，等待用户手动配置LED灯带");
                // 不再自动创建灯带配置，让用户通过前端界面手动添加
            }
        }

        if config_changed {
            log::info!("✅ 显示器配置已更新");
        } else {
            log::info!("ℹ️ 显示器配置无变化");
        }

        Ok(config_changed)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ConfigStats {
        let config = self.config.read().await;
        let display_stats = self.display_registry.get_display_stats().await;

        ConfigStats {
            total_strips: config.strips.len(),
            total_displays: display_stats.total_displays,
            config_version: config.version,
            has_color_calibration: true,
        }
    }
}

/// 配置统计信息
#[derive(Debug, Clone)]
pub struct ConfigStats {
    pub total_strips: usize,
    pub total_displays: usize,
    pub config_version: u8,
    pub has_color_calibration: bool,
}

// 为了兼容性，提供从新配置到旧配置的转换
impl From<LedStripConfigGroupV2> for crate::ambient_light::LedStripConfigGroup {
    fn from(v2_config: LedStripConfigGroupV2) -> Self {
        let strips = v2_config
            .strips
            .into_iter()
            .map(|strip| crate::ambient_light::LedStripConfig {
                index: strip.index,
                border: strip.border,
                display_id: 0, // 临时设为0，需要在使用时动态解析
                len: strip.len,
                led_type: strip.led_type,
                reversed: strip.reversed,
            })
            .collect();

        let mut config = crate::ambient_light::LedStripConfigGroup {
            strips,
            mappers: Vec::new(),
            color_calibration: v2_config.color_calibration,
        };

        config.generate_mappers();
        config
    }
}
