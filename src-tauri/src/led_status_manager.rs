use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{watch, OnceCell, RwLock};

use crate::{
    ambient_light::{BorderColors, LedStripConfig},
    led_data_sender::DataSendMode,
    websocket_events::WebSocketEventPublisher,
};

/// LED状态统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedStatusStats {
    /// 当前数据发送模式
    pub data_send_mode: DataSendMode,
    /// 单屏配置模式是否激活
    pub single_display_config_mode: bool,
    /// 当前活跃的呼吸灯带（display_id, border）
    pub active_breathing_strip: Option<(u32, String)>,
    /// 当前LED颜色数据字节数
    pub current_colors_bytes: usize,
    /// 当前排序颜色数据字节数
    pub sorted_colors_bytes: usize,
    /// 最后更新时间戳
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// 数据发送统计
    pub send_stats: LedSendStats,
}

/// LED数据发送统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LedSendStats {
    /// 总发送包数
    pub total_packets_sent: u64,
    /// 总发送字节数
    pub total_bytes_sent: u64,
    /// 最后发送时间
    pub last_send_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 发送错误次数
    pub send_errors: u64,
}

impl Default for LedStatusStats {
    fn default() -> Self {
        Self {
            data_send_mode: DataSendMode::None,
            single_display_config_mode: false,
            active_breathing_strip: None,
            current_colors_bytes: 0,
            sorted_colors_bytes: 0,
            last_updated: chrono::Utc::now(),
            send_stats: LedSendStats::default(),
        }
    }
}

/// 统一的LED状态管理器
/// 负责集中管理所有LED相关的状态数据，提供统一的状态获取和更新接口
pub struct LedStatusManager {
    /// LED状态数据
    status: Arc<RwLock<LedStatusStats>>,
    /// 当前LED颜色数据
    current_colors: Arc<RwLock<Vec<u8>>>,
    /// 当前排序LED颜色数据
    sorted_colors: Arc<RwLock<Vec<u8>>>,
    /// 单屏配置数据
    single_display_config_data: Arc<RwLock<Option<(Vec<LedStripConfig>, BorderColors)>>>,
    /// 状态变更通知发送器
    status_change_tx: watch::Sender<LedStatusStats>,
    /// 状态变更通知接收器
    status_change_rx: Arc<RwLock<watch::Receiver<LedStatusStats>>>,
}

impl LedStatusManager {
    /// 获取全局实例
    pub async fn global() -> &'static Self {
        static LED_STATUS_MANAGER: OnceCell<LedStatusManager> = OnceCell::const_new();

        LED_STATUS_MANAGER
            .get_or_init(|| async {
                info!("🔧 Initializing LedStatusManager...");

                let initial_status = LedStatusStats::default();
                let (status_change_tx, status_change_rx) = watch::channel(initial_status.clone());

                LedStatusManager {
                    status: Arc::new(RwLock::new(initial_status)),
                    current_colors: Arc::new(RwLock::new(Vec::new())),
                    sorted_colors: Arc::new(RwLock::new(Vec::new())),
                    single_display_config_data: Arc::new(RwLock::new(None)),
                    status_change_tx,
                    status_change_rx: Arc::new(RwLock::new(status_change_rx)),
                }
            })
            .await
    }

    /// 获取当前LED状态
    pub async fn get_status(&self) -> LedStatusStats {
        self.status.read().await.clone()
    }

    /// 获取当前LED颜色数据
    pub async fn get_current_colors(&self) -> Vec<u8> {
        self.current_colors.read().await.clone()
    }

    /// 获取当前排序LED颜色数据
    pub async fn get_sorted_colors(&self) -> Vec<u8> {
        self.sorted_colors.read().await.clone()
    }

    /// 获取单屏配置数据
    pub async fn get_single_display_config_data(
        &self,
    ) -> Option<(Vec<LedStripConfig>, BorderColors)> {
        self.single_display_config_data.read().await.clone()
    }

    /// 设置数据发送模式
    pub async fn set_data_send_mode(&self, mode: DataSendMode) -> anyhow::Result<()> {
        {
            let mut status = self.status.write().await;
            status.data_send_mode = mode;
            status.last_updated = chrono::Utc::now();
        }

        self.notify_status_changed().await?;
        info!("LED data send mode changed to: {mode}");
        Ok(())
    }



    /// 设置单屏配置模式状态
    pub async fn set_single_display_config_mode(
        &self,
        active: bool,
        config_data: Option<(Vec<LedStripConfig>, BorderColors)>,
    ) -> anyhow::Result<()> {
        {
            let mut status = self.status.write().await;
            status.single_display_config_mode = active;
            status.last_updated = chrono::Utc::now();
        }

        {
            let mut config_data_guard = self.single_display_config_data.write().await;
            *config_data_guard = config_data;
        }

        self.notify_status_changed().await?;
        info!(
            "LED single display config mode changed to: {}",
            if active { "active" } else { "inactive" }
        );
        Ok(())
    }

    /// 设置活跃呼吸灯带
    pub async fn set_active_breathing_strip(
        &self,
        display_id: Option<u32>,
        border: Option<String>,
    ) -> anyhow::Result<()> {
        {
            let mut status = self.status.write().await;
            status.active_breathing_strip = match (display_id, border) {
                (Some(id), Some(b)) => Some((id, b)),
                _ => None,
            };
            status.last_updated = chrono::Utc::now();
        }

        self.notify_status_changed().await?;

        let current_status = self.get_status().await;
        info!(
            "LED active breathing strip changed to: {:?}",
            current_status.active_breathing_strip
        );
        Ok(())
    }

    /// 更新LED颜色数据
    pub async fn update_colors(
        &self,
        colors: Vec<u8>,
        sorted_colors: Vec<u8>,
    ) -> anyhow::Result<()> {
        {
            let mut current_colors_guard = self.current_colors.write().await;
            *current_colors_guard = colors;
        }

        {
            let mut sorted_colors_guard = self.sorted_colors.write().await;
            *sorted_colors_guard = sorted_colors;
        }

        {
            let mut status = self.status.write().await;
            status.current_colors_bytes = self.current_colors.read().await.len();
            status.sorted_colors_bytes = self.sorted_colors.read().await.len();
            status.last_updated = chrono::Utc::now();
        }

        self.notify_status_changed().await?;

        let current_status = self.get_status().await;
        debug!(
            "LED colors updated: {} bytes current, {} bytes sorted",
            current_status.current_colors_bytes, current_status.sorted_colors_bytes
        );
        Ok(())
    }

    /// 记录数据发送统计
    pub async fn record_send_stats(
        &self,
        packets_sent: u64,
        bytes_sent: u64,
        success: bool,
    ) -> anyhow::Result<()> {
        {
            let mut status = self.status.write().await;
            status.send_stats.total_packets_sent += packets_sent;
            status.send_stats.total_bytes_sent += bytes_sent;
            status.send_stats.last_send_time = Some(chrono::Utc::now());

            if !success {
                status.send_stats.send_errors += 1;
            }

            status.last_updated = chrono::Utc::now();
        }

        self.notify_status_changed().await?;
        debug!(
            "LED send stats updated: {packets_sent} packets, {bytes_sent} bytes, success: {success}"
        );
        Ok(())
    }

    /// 获取状态变更通知接收器
    pub async fn subscribe_status_changes(&self) -> watch::Receiver<LedStatusStats> {
        self.status_change_rx.read().await.clone()
    }

    /// 通知状态变更
    async fn notify_status_changed(&self) -> anyhow::Result<()> {
        let current_status = self.get_status().await;

        // 发送状态变更通知
        if let Err(e) = self.status_change_tx.send(current_status.clone()) {
            warn!("Failed to send status change notification: {e}");
        }

        // 通过WebSocket广播状态变更
        let websocket_publisher = WebSocketEventPublisher::global().await;
        websocket_publisher.publish_led_status_changed().await;

        info!(
            "🔄 LED状态变更已通知: mode={:?}, send_stats={:?}",
            current_status.data_send_mode,
            current_status.send_stats
        );

        Ok(())
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) -> anyhow::Result<()> {
        {
            let mut status = self.status.write().await;
            status.send_stats = LedSendStats::default();
            status.last_updated = chrono::Utc::now();
        }

        self.notify_status_changed().await?;
        info!("LED statistics reset");
        Ok(())
    }

    /// 获取详细的状态信息（用于调试）
    pub async fn get_debug_info(&self) -> String {
        let status = self.get_status().await;
        let current_colors_len = self.current_colors.read().await.len();
        let sorted_colors_len = self.sorted_colors.read().await.len();
        let config_data = self.single_display_config_data.read().await.clone();

        format!(
            "LED Status Manager Debug Info:\n\
             - Data Send Mode: {:?}\n\
             - Single Display Config Mode: {}\n\
             - Active Breathing Strip: {:?}\n\
             - Current Colors: {} bytes\n\
             - Sorted Colors: {} bytes\n\
             - Config Data: {}\n\
             - Send Stats: {} packets, {} bytes, {} errors\n\
             - Last Updated: {}",
            status.data_send_mode,
            status.single_display_config_mode,
            status.active_breathing_strip,
            current_colors_len,
            sorted_colors_len,
            if config_data.is_some() {
                "Present"
            } else {
                "None"
            },
            status.send_stats.total_packets_sent,
            status.send_stats.total_bytes_sent,
            status.send_stats.send_errors,
            status.last_updated.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::led_data_sender::DataSendMode;

    #[tokio::test]
    async fn test_led_status_manager_initialization() {
        use crate::led_data_sender::LedDataSender;

        // 重置LED数据发送器状态到初始值
        let sender = LedDataSender::global().await;
        sender.set_mode(DataSendMode::None).await;

        let manager = LedStatusManager::global().await;

        // 重置状态管理器状态到初始值，因为其他测试可能已经修改了全局状态
        let _ = manager.set_data_send_mode(DataSendMode::None).await;
        let _ = manager.set_single_display_config_mode(false, None).await;
        let _ = manager.set_active_breathing_strip(None, None).await;

        let status = manager.get_status().await;

        // 验证重置后的状态
        assert_eq!(status.data_send_mode, DataSendMode::None);
        assert!(!status.single_display_config_mode);
        assert_eq!(status.active_breathing_strip, None);
        // 注意：current_colors_bytes 和 sorted_colors_bytes 可能被其他测试影响，
        // 所以我们不检查这些字段的具体值

        // 测试结束时保持重置状态，避免影响其他测试
        let sender = LedDataSender::global().await;
        sender.set_mode(DataSendMode::None).await;
    }

    #[tokio::test]
    async fn test_set_data_send_mode() {
        use crate::led_data_sender::LedDataSender;

        // 重置LED数据发送器状态到初始值
        let sender = LedDataSender::global().await;
        sender.set_mode(DataSendMode::None).await;

        let manager = LedStatusManager::global().await;

        // 首先重置状态管理器
        let _ = manager.set_data_send_mode(DataSendMode::None).await;

        // 设置发送模式
        manager
            .set_data_send_mode(DataSendMode::AmbientLight)
            .await
            .unwrap();

        // 立即检查状态，避免被其他测试影响
        let status = manager.get_status().await;
        assert_eq!(status.data_send_mode, DataSendMode::AmbientLight);

        // 测试设置为其他模式
        manager
            .set_data_send_mode(DataSendMode::TestEffect)
            .await
            .unwrap();

        let status = manager.get_status().await;
        assert_eq!(status.data_send_mode, DataSendMode::TestEffect);

        // 测试结束时重置状态，避免影响其他测试
        let _ = manager.set_data_send_mode(DataSendMode::None).await;
        sender.set_mode(DataSendMode::None).await;
    }

    #[tokio::test]
    async fn test_update_colors() {
        let manager = LedStatusManager::global().await;

        let test_colors = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // RGB data
        let test_sorted_colors = vec![255, 255, 255, 0, 0, 0]; // Sorted data

        manager
            .update_colors(test_colors.clone(), test_sorted_colors.clone())
            .await
            .unwrap();

        let current_colors = manager.get_current_colors().await;
        let sorted_colors = manager.get_sorted_colors().await;
        let status = manager.get_status().await;

        assert_eq!(current_colors, test_colors);
        assert_eq!(sorted_colors, test_sorted_colors);
        assert_eq!(status.current_colors_bytes, test_colors.len());
        assert_eq!(status.sorted_colors_bytes, test_sorted_colors.len());
    }
}
