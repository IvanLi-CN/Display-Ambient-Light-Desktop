#[cfg(test)]
mod tests {
    use super::*;
    use crate::led_data_sender::DataSendMode;
    use tokio;

    #[tokio::test]
    async fn test_led_status_manager_initialization() {
        let manager = LedStatusManager::global().await;
        let status = manager.get_status().await;
        
        // 验证初始状态
        assert_eq!(status.data_send_mode, DataSendMode::None);
        assert_eq!(status.test_mode_active, false);
        assert_eq!(status.single_display_config_mode, false);
        assert_eq!(status.active_breathing_strip, None);
        assert_eq!(status.current_colors_bytes, 0);
        assert_eq!(status.sorted_colors_bytes, 0);
    }

    #[tokio::test]
    async fn test_set_data_send_mode() {
        let manager = LedStatusManager::global().await;
        
        // 设置发送模式
        manager.set_data_send_mode(DataSendMode::AmbientLight).await.unwrap();
        
        let status = manager.get_status().await;
        assert_eq!(status.data_send_mode, DataSendMode::AmbientLight);
    }

    #[tokio::test]
    async fn test_set_test_mode() {
        let manager = LedStatusManager::global().await;
        
        // 启用测试模式
        manager.set_test_mode_active(true).await.unwrap();
        
        let status = manager.get_status().await;
        assert_eq!(status.test_mode_active, true);
        
        // 禁用测试模式
        manager.set_test_mode_active(false).await.unwrap();
        
        let status = manager.get_status().await;
        assert_eq!(status.test_mode_active, false);
    }

    #[tokio::test]
    async fn test_update_colors() {
        let manager = LedStatusManager::global().await;
        
        let test_colors = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // RGB data
        let test_sorted_colors = vec![255, 255, 255, 0, 0, 0]; // Sorted data
        
        manager.update_colors(test_colors.clone(), test_sorted_colors.clone()).await.unwrap();
        
        let current_colors = manager.get_current_colors().await;
        let sorted_colors = manager.get_sorted_colors().await;
        let status = manager.get_status().await;
        
        assert_eq!(current_colors, test_colors);
        assert_eq!(sorted_colors, test_sorted_colors);
        assert_eq!(status.current_colors_bytes, test_colors.len());
        assert_eq!(status.sorted_colors_bytes, test_sorted_colors.len());
    }

    #[tokio::test]
    async fn test_record_send_stats() {
        let manager = LedStatusManager::global().await;
        
        // 重置统计信息
        manager.reset_stats().await.unwrap();
        
        let initial_status = manager.get_status().await;
        assert_eq!(initial_status.send_stats.total_packets_sent, 0);
        assert_eq!(initial_status.send_stats.total_bytes_sent, 0);
        
        // 记录发送统计
        manager.record_send_stats(5, 1024, true).await.unwrap();
        
        let status = manager.get_status().await;
        assert_eq!(status.send_stats.total_packets_sent, 5);
        assert_eq!(status.send_stats.total_bytes_sent, 1024);
        assert!(status.send_stats.last_send_time.is_some());
        
        // 记录失败的发送
        manager.record_send_stats(1, 256, false).await.unwrap();
        
        let status = manager.get_status().await;
        assert_eq!(status.send_stats.total_packets_sent, 6);
        assert_eq!(status.send_stats.total_bytes_sent, 1280);
        assert_eq!(status.send_stats.send_errors, 1);
    }

    #[tokio::test]
    async fn test_set_active_breathing_strip() {
        let manager = LedStatusManager::global().await;
        
        // 设置活跃呼吸灯带
        manager.set_active_breathing_strip(Some(1), Some("top".to_string())).await.unwrap();
        
        let status = manager.get_status().await;
        assert_eq!(status.active_breathing_strip, Some((1, "top".to_string())));
        
        // 清除活跃呼吸灯带
        manager.set_active_breathing_strip(None, None).await.unwrap();
        
        let status = manager.get_status().await;
        assert_eq!(status.active_breathing_strip, None);
    }

    #[tokio::test]
    async fn test_status_change_subscription() {
        let manager = LedStatusManager::global().await;
        
        // 订阅状态变更
        let mut receiver = manager.subscribe_status_changes().await;
        
        // 修改状态
        manager.set_data_send_mode(DataSendMode::TestEffect).await.unwrap();
        
        // 验证收到状态变更通知
        let changed_status = receiver.changed().await;
        assert!(changed_status.is_ok());
        
        let status = receiver.borrow().clone();
        assert_eq!(status.data_send_mode, DataSendMode::TestEffect);
    }

    #[tokio::test]
    async fn test_debug_info() {
        let manager = LedStatusManager::global().await;
        
        // 设置一些状态
        manager.set_data_send_mode(DataSendMode::AmbientLight).await.unwrap();
        manager.set_test_mode_active(true).await.unwrap();
        manager.update_colors(vec![255, 0, 0], vec![0, 255, 0]).await.unwrap();
        
        let debug_info = manager.get_debug_info().await;
        
        // 验证调试信息包含预期内容
        assert!(debug_info.contains("AmbientLight"));
        assert!(debug_info.contains("true"));
        assert!(debug_info.contains("3 bytes"));
        assert!(debug_info.contains("3 bytes"));
    }
}
