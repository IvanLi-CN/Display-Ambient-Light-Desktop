use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

use crate::rpc::UdpRpc;

/// LED数据发送模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSendMode {
    /// 不发送任何数据
    None,
    /// 屏幕氛围光数据
    AmbientLight,
    /// 单灯条配置数据
    StripConfig,
    /// 测试效果数据
    TestEffect,
}

impl Default for DataSendMode {
    fn default() -> Self {
        DataSendMode::None
    }
}

impl std::fmt::Display for DataSendMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSendMode::None => write!(f, "None"),
            DataSendMode::AmbientLight => write!(f, "AmbientLight"),
            DataSendMode::StripConfig => write!(f, "StripConfig"),
            DataSendMode::TestEffect => write!(f, "TestEffect"),
        }
    }
}

/// LED数据包信息
#[derive(Debug, Clone)]
pub struct LedDataPacket {
    /// 字节偏移量
    pub offset: u16,
    /// 颜色数据
    pub data: Vec<u8>,
    /// 数据源描述（用于日志）
    pub source: String,
}

impl LedDataPacket {
    pub fn new(offset: u16, data: Vec<u8>, source: String) -> Self {
        Self {
            offset,
            data,
            source,
        }
    }

    /// 构建0x02协议数据包
    pub fn build_packet(&self) -> Vec<u8> {
        let mut packet = vec![0x02]; // Header
        packet.push((self.offset >> 8) as u8); // Offset high
        packet.push((self.offset & 0xff) as u8); // Offset low
        packet.extend_from_slice(&self.data); // Color data
        packet
    }
}

/// 统一的LED数据发送管理器
pub struct LedDataSender {
    /// 当前发送模式
    current_mode: Arc<RwLock<DataSendMode>>,
}

impl LedDataSender {
    /// 获取全局实例
    pub async fn global() -> &'static Self {
        static LED_DATA_SENDER: OnceCell<LedDataSender> = OnceCell::const_new();

        LED_DATA_SENDER
            .get_or_init(|| async {
                LedDataSender {
                    current_mode: Arc::new(RwLock::new(DataSendMode::default())),
                }
            })
            .await
    }

    /// 获取当前发送模式
    pub async fn get_mode(&self) -> DataSendMode {
        *self.current_mode.read().await
    }

    /// 设置发送模式
    pub async fn set_mode(&self, mode: DataSendMode) {
        let mut current_mode = self.current_mode.write().await;
        let old_mode = *current_mode;
        *current_mode = mode;

        info!("LED data send mode changed: {} -> {}", old_mode, mode);
    }

    /// 检查是否可以发送指定模式的数据
    pub async fn can_send(&self, mode: DataSendMode) -> bool {
        let current_mode = self.get_mode().await;
        current_mode == mode
    }

    /// 发送LED数据包（统一入口）
    pub async fn send_packet(
        &self,
        packet: LedDataPacket,
        expected_mode: DataSendMode,
    ) -> anyhow::Result<()> {
        // 检查当前模式是否允许发送
        if !self.can_send(expected_mode).await {
            let current_mode = self.get_mode().await;
            return Err(anyhow::anyhow!(
                "Cannot send {} data in {} mode",
                expected_mode,
                current_mode
            ));
        }

        // 获取UDP RPC实例
        let udp_rpc = UdpRpc::global().await;
        if let Err(err) = udp_rpc {
            warn!("UDP RPC not available: {}", err);
            return Err(anyhow::anyhow!("UDP RPC not available: {}", err));
        }
        let udp_rpc = udp_rpc.as_ref().unwrap();

        // 构建并发送数据包
        let packet_data = packet.build_packet();

        log::debug!(
            "Sending LED packet: mode={}, source={}, offset={}, data_len={}, packet_len={}",
            expected_mode,
            packet.source,
            packet.offset,
            packet.data.len(),
            packet_data.len()
        );

        match udp_rpc.send_to_all(&packet_data).await {
            Ok(_) => {
                log::debug!(
                    "✅ Successfully sent LED packet: {} (offset={}, {} bytes)",
                    packet.source,
                    packet.offset,
                    packet_data.len()
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "❌ Failed to send LED packet: {} (offset={}, {} bytes): {}",
                    packet.source,
                    packet.offset,
                    packet_data.len(),
                    e
                );
                Err(e)
            }
        }
    }

    /// 发送屏幕氛围光数据
    pub async fn send_ambient_light_data(&self, offset: u16, data: Vec<u8>) -> anyhow::Result<()> {
        let packet = LedDataPacket::new(offset, data, "AmbientLight".to_string());
        self.send_packet(packet, DataSendMode::AmbientLight).await
    }

    /// 发送单灯条配置数据
    pub async fn send_strip_config_data(&self, offset: u16, data: Vec<u8>) -> anyhow::Result<()> {
        let packet = LedDataPacket::new(offset, data, "StripConfig".to_string());
        self.send_packet(packet, DataSendMode::StripConfig).await
    }

    /// 发送测试效果数据
    pub async fn send_test_effect_data(&self, offset: u16, data: Vec<u8>) -> anyhow::Result<()> {
        let packet = LedDataPacket::new(offset, data, "TestEffect".to_string());
        self.send_packet(packet, DataSendMode::TestEffect).await
    }

    /// 强制发送数据包（忽略模式检查，用于特殊情况如关闭LED）
    pub async fn force_send_packet(&self, packet: LedDataPacket) -> anyhow::Result<()> {
        let udp_rpc = UdpRpc::global().await;
        if let Err(err) = udp_rpc {
            warn!("UDP RPC not available: {}", err);
            return Err(anyhow::anyhow!("UDP RPC not available: {}", err));
        }
        let udp_rpc = udp_rpc.as_ref().unwrap();

        let packet_data = packet.build_packet();

        log::info!(
            "Force sending LED packet: source={}, offset={}, data_len={}",
            packet.source,
            packet.offset,
            packet.data.len()
        );

        udp_rpc.send_to_all(&packet_data).await
    }

    /// Get statistics about the current state (for testing/debugging)
    pub async fn get_stats(&self) -> String {
        let mode = self.get_mode().await;
        format!("Current mode: {}", mode)
    }
}
