use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::http_server::AppState;

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// LED颜色变化
    LedColorsChanged { colors: Vec<u8> },
    /// LED排序颜色变化
    LedSortedColorsChanged { sorted_colors: Vec<u8> },
    /// 配置变化
    ConfigChanged { config: serde_json::Value },
    /// 设备列表变化
    BoardsChanged { boards: serde_json::Value },
    /// 显示器状态变化
    DisplaysChanged { displays: serde_json::Value },
    /// 环境光状态变化
    AmbientLightStateChanged { state: serde_json::Value },
    /// 导航事件
    Navigate { path: String },
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

/// WebSocket连接管理器
#[derive(Clone)]
pub struct WebSocketManager {
    sender: broadcast::Sender<WsMessage>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// 广播消息给所有连接的客户端
    pub fn broadcast(
        &self,
        message: WsMessage,
    ) -> Result<(), broadcast::error::SendError<WsMessage>> {
        self.sender.send(message).map(|_| ())
    }

    /// 获取接收器
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.sender.subscribe()
    }
}

/// WebSocket升级处理器
pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// 处理WebSocket连接
async fn handle_socket(socket: WebSocket, state: AppState) {
    log::info!("🔌 New WebSocket connection established for LED events");
    let (mut sender, mut receiver) = socket.split();

    // 从AppState获取WebSocketManager
    let ws_manager = state.websocket_manager.clone();
    let mut ws_receiver = ws_manager.subscribe();

    // 发送连接确认消息
    if sender
        .send(Message::Text(
            serde_json::to_string(&WsMessage::Pong).unwrap(),
        ))
        .await
        .is_err()
    {
        log::warn!("❌ Failed to send connection confirmation message");
        return;
    }
    log::info!("✅ Connection confirmation message sent to LED events WebSocket");

    // 发送一个测试LED颜色事件
    let test_colors = vec![255u8, 0, 0, 255, 255, 255, 0, 255, 0, 255, 255, 255]; // 红色和绿色LED
    let test_message = WsMessage::LedColorsChanged {
        colors: test_colors,
    };
    if sender
        .send(Message::Text(serde_json::to_string(&test_message).unwrap()))
        .await
        .is_err()
    {
        log::warn!("❌ Failed to send test LED colors message");
        return;
    }
    log::info!("✅ Test LED colors message sent to WebSocket client");

    // 处理客户端消息的任务
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Ping => {
                                log::debug!("收到WebSocket心跳");
                            }
                            _ => {
                                // 处理其他客户端消息
                                log::debug!("收到WebSocket消息: {:?}", ws_msg);
                            }
                        }
                    }
                }
                Message::Binary(_) => {
                    // 处理二进制消息
                    log::debug!("收到WebSocket二进制消息");
                }
                Message::Close(_) => {
                    log::info!("WebSocket连接关闭");
                    break;
                }
                _ => {}
            }
        }
    });

    // 广播消息给客户端的任务
    let mut send_task = tokio::spawn(async move {
        // 实现从ws_receiver接收广播消息并发送给客户端
        while let Ok(msg) = ws_receiver.recv().await {
            log::debug!("📤 Sending WebSocket message: {:?}", msg);
            let text = match serde_json::to_string(&msg) {
                Ok(text) => text,
                Err(e) => {
                    log::error!("序列化WebSocket消息失败: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(text)).await.is_err() {
                log::debug!("WebSocket发送消息失败，连接可能已断开");
                break;
            } else {
                log::debug!("✅ WebSocket消息发送成功");
            }
        }
    });

    // 等待任一任务完成
    tokio::select! {
        _ = (&mut recv_task) => {
            send_task.abort();
        },
        _ = (&mut send_task) => {
            recv_task.abort();
        }
    }

    log::info!("WebSocket连接已断开");
}
