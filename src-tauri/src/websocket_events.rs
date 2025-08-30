use tokio::sync::OnceCell;

use crate::{
    ambient_light::LedStripConfigGroup,
    ambient_light_state::AmbientLightState,
    display::DisplayState,
    http_server::websocket::{
        LedColorsChangedData, LedSortedColorsChangedData, LedStripColorsChangedData, NavigateData,
        WebSocketManager, WsMessage,
    },
    led_data_sender::DataSendMode,
    led_preview_state::LedPreviewState,
    rpc::BoardInfo,
    user_preferences::UserPreferences,
};

/// WebSocket事件发布器
/// 负责将应用中的各种状态变化通过WebSocket广播给前端
pub struct WebSocketEventPublisher {
    ws_manager: WebSocketManager,
}

impl WebSocketEventPublisher {
    /// 获取全局实例
    pub async fn global() -> &'static Self {
        static WEBSOCKET_EVENT_PUBLISHER_GLOBAL: OnceCell<WebSocketEventPublisher> =
            OnceCell::const_new();
        WEBSOCKET_EVENT_PUBLISHER_GLOBAL
            .get_or_init(|| async {
                log::info!("🔌 Initializing WebSocket Event Publisher...");
                Self {
                    ws_manager: WebSocketManager::new(),
                }
            })
            .await
    }

    /// 获取WebSocket管理器的引用
    pub fn get_websocket_manager(&self) -> &WebSocketManager {
        &self.ws_manager
    }

    /// 发布LED颜色变化事件
    pub async fn publish_led_colors_changed(&self, colors: &[u8]) {
        log::info!(
            "🎨 Publishing LED colors changed event: {} bytes",
            colors.len()
        );
        let message = WsMessage::LedColorsChanged {
            data: LedColorsChangedData {
                colors: colors.to_vec(),
            },
        };
        match self
            .ws_manager
            .send_to_subscribers("LedColorsChanged", message)
            .await
        {
            Ok(subscriber_count) => {
                if subscriber_count > 0 {
                    log::info!("✅ LED颜色变化事件已发送给 {subscriber_count} 个订阅者");
                } else {
                    log::info!("📭 没有订阅者接收LED颜色变化事件");
                }
            }
            Err(e) => {
                log::error!("❌ 发送LED颜色变化事件失败: {e}");
            }
        }
    }

    /// 发布LED颜色变化事件（按物理顺序排列的颜色数据）
    pub async fn publish_led_sorted_colors_changed(&self, sorted_colors: &[u8], led_offset: usize) {
        // 获取当前模式信息和时间戳
        let sender = crate::led_data_sender::LedDataSender::global().await;
        let current_mode = sender.get_mode().await;

        // 🔧 从LED状态管理器获取真实的数据更新时间戳
        let status_manager = crate::led_status_manager::LedStatusManager::global().await;
        let status = status_manager.get_status().await;
        let timestamp = status.last_updated;

        let message = WsMessage::LedSortedColorsChanged {
            data: LedSortedColorsChangedData {
                sorted_colors: sorted_colors.to_vec(),
                mode: current_mode,
                led_offset,
                timestamp,
            },
        };
        match self
            .ws_manager
            .send_to_subscribers("LedSortedColorsChanged", message)
            .await
        {
            Ok(subscriber_count) => {
                if subscriber_count > 0 {
                    log::info!(
                        "✅ LED颜色变化事件（按物理顺序排列）已发送给 {subscriber_count} 个订阅者"
                    );
                } else {
                }
            }
            Err(e) => {
                log::error!("❌ 发送LED颜色变化事件（按物理顺序排列）失败: {e}");
            }
        }
    }

    /// 发布LED灯带颜色变化事件（按灯带分组）
    pub async fn publish_led_strip_colors_changed(
        &self,
        display_id: u32,
        border: &str,
        strip_index: usize,
        colors: &[u8],
    ) {
        let sender = crate::led_data_sender::LedDataSender::global().await;
        let current_mode = sender.get_mode().await;

        log::debug!(
            "🎨 Publishing LED strip colors changed event: display_id={}, border={}, strip_index={}, {} bytes, mode={:?}",
            display_id,
            border,
            strip_index,
            colors.len(),
            current_mode
        );

        let message = WsMessage::LedStripColorsChanged {
            data: LedStripColorsChangedData {
                display_id,
                border: border.to_string(),
                strip_index,
                colors: colors.to_vec(),
                mode: current_mode,
            },
        };

        // 支持按显示器过滤的订阅
        let display_event = format!("LedStripColorsChanged:display_{}", display_id);

        // 发送到特定显示器订阅者
        match self
            .ws_manager
            .send_to_subscribers(&display_event, message.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                log::error!("❌ 发送LED灯带颜色变化事件到显示器 {display_id} 失败: {e}");
            }
        }

        // 发送到通用订阅者（向后兼容）
        match self
            .ws_manager
            .send_to_subscribers("LedStripColorsChanged", message)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                log::error!("❌ 发送LED灯带颜色变化事件失败: {e}");
            }
        }
    }

    /// 发布LED状态变化事件
    pub async fn publish_led_status_changed(&self) {
        self.publish_led_status_changed_with_mode(None).await;
    }

    /// 发布LED状态变化事件（带指定模式）
    pub async fn publish_led_status_changed_with_mode(&self, mode_override: Option<DataSendMode>) {
        // 获取当前LED状态
        let sender = crate::led_data_sender::LedDataSender::global().await;
        let config_manager = crate::ambient_light::ConfigManagerV2::global().await;

        // 获取当前模式（如果没有提供覆盖值）
        let mode = if let Some(mode) = mode_override {
            mode
        } else {
            sender.get_mode().await
        };

        // 获取LED配置以计算总数量和数据长度
        let configs = config_manager.get_config().await;
        let total_led_count: u32 = configs.strips.iter().map(|strip| strip.len as u32).sum();

        // 计算数据长度（每个LED 3字节 RGB 或 4字节 RGBW）
        let data_length: u32 = configs
            .strips
            .iter()
            .map(|strip| {
                match strip.led_type {
                    crate::ambient_light::LedType::WS2812B => strip.len as u32 * 3, // RGB
                    crate::ambient_light::LedType::SK6812 => strip.len as u32 * 4,  // RGBW
                }
            })
            .sum();

        // 根据模式确定频率
        let frequency = match mode {
            DataSendMode::AmbientLight => 30.0,    // 氛围光模式30Hz
            DataSendMode::StripConfig => 30.0,     // 配置模式30Hz
            DataSendMode::TestEffect => 1.0,       // 测试效果1Hz
            DataSendMode::ColorCalibration => 1.0, // 颜色校准1Hz
            DataSendMode::None => 0.0,             // 无发送
        };

        // 创建状态对象
        let status = serde_json::json!({
            "mode": mode,
            "frequency": frequency,
            "data_length": data_length,
            "total_led_count": total_led_count,
            "test_mode_active": mode == DataSendMode::TestEffect,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let message = WsMessage::LedStatusChanged { data: status };
        match self
            .ws_manager
            .send_to_subscribers("LedStatusChanged", message)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                log::warn!("发送LED状态变化事件失败: {e}");
            }
        }
    }

    /// 发布配置变化事件
    pub async fn publish_config_changed(&self, config: &LedStripConfigGroup) {
        if let Ok(config_json) = serde_json::to_value(config) {
            let message = WsMessage::ConfigChanged { data: config_json };
            match self
                .ws_manager
                .send_to_subscribers("ConfigChanged", message)
                .await
            {
                Ok(subscriber_count) => {
                    if subscriber_count > 0 {
                        log::debug!("✅ 配置变化事件已发送给 {subscriber_count} 个订阅者");
                    }
                }
                Err(e) => {
                    log::debug!("发送配置变化事件失败: {e}");
                }
            }
        } else {
            log::error!("序列化配置数据失败");
        }
    }

    /// 发布设备列表变化事件
    pub async fn publish_boards_changed(&self, boards: &[BoardInfo]) {
        if let Ok(boards_json) = serde_json::to_value(boards) {
            let message = WsMessage::BoardsChanged { data: boards_json };
            match self
                .ws_manager
                .send_to_subscribers("BoardsChanged", message)
                .await
            {
                Ok(subscriber_count) => {
                    if subscriber_count > 0 {
                        log::debug!("✅ 设备列表变化事件已发送给 {subscriber_count} 个订阅者");
                    }
                }
                Err(e) => {
                    log::debug!("发送设备列表变化事件失败: {e}");
                }
            }
        } else {
            log::error!("序列化设备列表数据失败");
        }
    }

    /// 发布显示器状态变化事件
    pub async fn publish_displays_changed(&self, displays: &[DisplayState]) {
        if let Ok(displays_json) = serde_json::to_value(displays) {
            let message = WsMessage::DisplaysChanged {
                data: displays_json,
            };
            match self
                .ws_manager
                .send_to_subscribers("DisplaysChanged", message)
                .await
            {
                Ok(subscriber_count) => {
                    if subscriber_count > 0 {
                        log::debug!("✅ 显示器状态变化事件已发送给 {subscriber_count} 个订阅者");
                    }
                }
                Err(e) => {
                    log::debug!("发送显示器状态变化事件失败: {e}");
                }
            }
        } else {
            log::error!("序列化显示器状态数据失败");
        }
    }

    /// 发布环境光状态变化事件
    pub async fn publish_ambient_light_state_changed(&self, state: &AmbientLightState) {
        if let Ok(state_json) = serde_json::to_value(state) {
            let message = WsMessage::AmbientLightStateChanged { data: state_json };
            match self
                .ws_manager
                .send_to_subscribers("AmbientLightStateChanged", message)
                .await
            {
                Ok(subscriber_count) => {
                    if subscriber_count > 0 {
                        log::debug!("✅ 环境光状态变化事件已发送给 {subscriber_count} 个订阅者");
                    }
                }
                Err(e) => {
                    log::debug!("发送环境光状态变化事件失败: {e}");
                }
            }
        } else {
            log::error!("序列化环境光状态数据失败");
        }
    }

    /// 发布LED预览状态变化事件
    pub async fn publish_led_preview_state_changed(&self, state: &LedPreviewState) {
        if let Ok(state_json) = serde_json::to_value(state) {
            let message = WsMessage::LedPreviewStateChanged { data: state_json };
            match self
                .ws_manager
                .send_to_subscribers("LedPreviewStateChanged", message)
                .await
            {
                Ok(subscriber_count) => {
                    if subscriber_count > 0 {
                        log::debug!("✅ LED预览状态变化事件已发送给 {subscriber_count} 个订阅者");
                    }
                }
                Err(e) => {
                    log::debug!("发送LED预览状态变化事件失败: {e}");
                }
            }
        } else {
            log::error!("序列化LED预览状态数据失败");
        }
    }

    /// 发布用户偏好设置变化事件
    pub async fn publish_user_preferences_changed(&self, preferences: &UserPreferences) {
        if let Ok(preferences_json) = serde_json::to_value(preferences) {
            let message = WsMessage::ConfigChanged {
                data: preferences_json,
            };
            if let Err(e) = self.ws_manager.broadcast(message) {
                log::debug!("广播用户偏好设置变化失败: {e}");
            }
        } else {
            log::error!("序列化用户偏好设置数据失败");
        }
    }

    /// 发布导航事件
    pub async fn publish_navigate(&self, path: String) {
        let message = WsMessage::Navigate {
            data: NavigateData { path },
        };
        match self
            .ws_manager
            .send_to_subscribers("Navigate", message)
            .await
        {
            Ok(subscriber_count) => {
                if subscriber_count > 0 {
                    log::debug!("✅ 导航事件已发送给 {subscriber_count} 个订阅者");
                }
            }
            Err(e) => {
                log::debug!("发送导航事件失败: {e}");
            }
        }
    }

    /// 发布心跳事件
    pub async fn publish_ping(&self) {
        let message = WsMessage::Ping;
        if let Err(e) = self.ws_manager.broadcast(message) {
            log::debug!("广播心跳事件失败: {e}");
        }
    }
}

/// 便捷函数：获取全局WebSocket事件发布器
pub async fn get_websocket_publisher() -> &'static WebSocketEventPublisher {
    WebSocketEventPublisher::global().await
}

/// 便捷函数：发布LED颜色变化
pub async fn publish_led_colors_changed(colors: Vec<u8>) {
    get_websocket_publisher()
        .await
        .publish_led_colors_changed(&colors)
        .await;
}

/// 便捷函数：发布配置变化
pub async fn publish_config_changed(config: &LedStripConfigGroup) {
    get_websocket_publisher()
        .await
        .publish_config_changed(config)
        .await;
}

/// 便捷函数：发布设备列表变化
pub async fn publish_boards_changed(boards: &[BoardInfo]) {
    get_websocket_publisher()
        .await
        .publish_boards_changed(boards)
        .await;
}

/// 便捷函数：发布显示器状态变化
pub async fn publish_displays_changed(displays: &[DisplayState]) {
    get_websocket_publisher()
        .await
        .publish_displays_changed(displays)
        .await;
}

/// 便捷函数：发布环境光状态变化
pub async fn publish_ambient_light_state_changed(state: &AmbientLightState) {
    get_websocket_publisher()
        .await
        .publish_ambient_light_state_changed(state)
        .await;
}

/// 便捷函数：发布LED预览状态变化
pub async fn publish_led_preview_state_changed(state: &LedPreviewState) {
    get_websocket_publisher()
        .await
        .publish_led_preview_state_changed(state)
        .await;
}

/// 便捷函数：发布导航事件
pub async fn publish_navigate(path: String) {
    get_websocket_publisher().await.publish_navigate(path).await;
}
