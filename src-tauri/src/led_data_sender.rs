use dirs::config_dir;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
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
    /// 测试模式下的目标地址
    test_target_address: Arc<RwLock<Option<SocketAddr>>>,
}

impl LedDataSender {
    /// 获取全局实例
    pub async fn global() -> &'static Self {
        static LED_DATA_SENDER: OnceCell<LedDataSender> = OnceCell::const_new();

        LED_DATA_SENDER
            .get_or_init(|| async {
                LedDataSender {
                    current_mode: Arc::new(RwLock::new(DataSendMode::default())),
                    test_target_address: Arc::new(RwLock::new(None)),
                }
            })
            .await
    }

    /// 获取UDP日志文件路径
    fn get_udp_log_path() -> PathBuf {
        let config_dir = config_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
        config_dir
            .join("cc.ivanli.ambient_light")
            .join("udp_packets.log")
    }

    /// 写入UDP数据包到日志文件
    async fn write_udp_packet_to_file(&self, offset: u16, packet_data: &[u8]) {
        let log_path = Self::get_udp_log_path();

        // 确保目录存在
        if let Some(parent) = log_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                error!("Failed to create UDP log directory: {}", e);
                return;
            }
        }

        // 格式化时间戳
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

        // 格式化十六进制数据
        let hex_data = packet_data
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        // 构建日志行
        let log_line = format!(
            "[{}] UDP Packet (offset={}): {}\n",
            timestamp, offset, hex_data
        );

        // 写入文件
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(log_line.as_bytes()).await {
                    error!("Failed to write UDP packet to log file: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to open UDP log file: {}", e);
            }
        }
    }

    /// 设置测试模式的目标地址
    pub async fn set_test_target(&self, address: Option<String>) {
        let mut target = self.test_target_address.write().await;
        *target = match address {
            Some(addr_str) => match SocketAddr::from_str(&addr_str) {
                Ok(addr) => {
                    info!("Test target address set to: {}", addr);
                    Some(addr)
                }
                Err(e) => {
                    error!("Failed to parse test target address '{}': {}", addr_str, e);
                    None
                }
            },
            None => {
                info!("Test target address cleared");
                None
            }
        };
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

        // 打印UDP数据包的十六进制内容（仅前64字节以避免日志过长）
        let hex_data = if packet_data.len() <= 64 {
            packet_data
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            let preview = &packet_data[..64];
            let hex_preview = preview
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            format!(
                "{} ... (truncated, total {} bytes)",
                hex_preview,
                packet_data.len()
            )
        };
        log::info!(
            "📦 UDP packet data (offset={}): {}",
            packet.offset,
            hex_data
        );

        // 写入UDP数据包到日志文件
        self.write_udp_packet_to_file(packet.offset, &packet_data)
            .await;

        // 根据模式选择发送方式
        log::info!("🔍 Checking send mode: expected_mode={:?}", expected_mode);
        let send_result = if expected_mode == DataSendMode::TestEffect
            || expected_mode == DataSendMode::StripConfig
        {
            let target_addr_option = *self.test_target_address.read().await;
            log::info!("🎯 Target address option: {:?}", target_addr_option);

            if let Some(target_addr) = target_addr_option {
                log::info!(
                    "✅ Sending {} data to specific address: {}",
                    packet.source,
                    target_addr
                );
                udp_rpc.send_to(&packet_data, target_addr).await
            } else {
                warn!(
                    "⚠️ {} mode is active, but no target address is set. Using broadcast mode.",
                    packet.source
                );
                udp_rpc.send_to_all(&packet_data).await
            }
        } else {
            log::info!(
                "📡 Sending {} data to all devices (broadcast mode)",
                packet.source
            );
            udp_rpc.send_to_all(&packet_data).await
        };

        match send_result {
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

    /// 发送完整的LED数据流（由发布服务负责拆包）
    pub async fn send_complete_led_data(
        &self,
        start_offset: u16,
        complete_data: Vec<u8>,
        source: &str,
    ) -> anyhow::Result<()> {
        let mode = match source {
            "AmbientLight" => DataSendMode::AmbientLight,
            "StripConfig" => DataSendMode::StripConfig,
            "TestEffect" => DataSendMode::TestEffect,
            _ => DataSendMode::AmbientLight,
        };

        // 拆分数据为UDP包
        let max_data_size = 500; // 每个UDP包的最大数据大小
        let mut current_offset = start_offset;
        let mut remaining_data = complete_data.as_slice();

        log::info!(
            "📦 Splitting complete LED data: total_size={} bytes, start_offset={}, source={}",
            complete_data.len(),
            start_offset,
            source
        );

        let mut packet_count = 0;
        while !remaining_data.is_empty() {
            let chunk_size = std::cmp::min(max_data_size, remaining_data.len());
            let chunk = remaining_data[..chunk_size].to_vec();
            remaining_data = &remaining_data[chunk_size..];

            packet_count += 1;
            log::info!(
                "📤 Sending packet {}: offset={}, size={} bytes, progress={}/{}",
                packet_count,
                current_offset,
                chunk.len(),
                complete_data.len() - remaining_data.len(),
                complete_data.len()
            );

            let packet = LedDataPacket::new(current_offset, chunk, source.to_string());
            self.send_packet(packet, mode).await?;

            current_offset += chunk_size as u16;
        }

        log::info!(
            "🎉 All data sent: {} packets, {} total bytes",
            packet_count,
            complete_data.len()
        );

        Ok(())
    }

    /// 发送屏幕氛围光数据（单个数据包，保持向后兼容）
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
