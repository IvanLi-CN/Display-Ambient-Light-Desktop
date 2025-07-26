use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{broadcast, RwLock};

use crate::http_server::AppState;

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// LED颜色变化
    LedColorsChanged { colors: Vec<u8> },
    /// LED排序颜色变化
    LedSortedColorsChanged {
        sorted_colors: Vec<u8>,
        mode: crate::led_data_sender::DataSendMode,
        /// LED偏移量（用于前端组装完整预览）
        led_offset: usize,
    },
    /// LED状态变化
    LedStatusChanged { status: serde_json::Value },
    /// 配置变化
    ConfigChanged { config: serde_json::Value },
    /// 设备列表变化
    BoardsChanged { boards: serde_json::Value },
    /// 显示器状态变化
    DisplaysChanged { displays: serde_json::Value },
    /// 环境光状态变化
    AmbientLightStateChanged { state: serde_json::Value },
    /// LED预览状态变化
    LedPreviewStateChanged { state: serde_json::Value },
    /// 导航事件
    Navigate { path: String },
    /// 订阅事件
    Subscribe { event_types: Vec<String> },
    /// 取消订阅事件
    Unsubscribe { event_types: Vec<String> },
    /// 订阅确认
    SubscriptionConfirmed { event_types: Vec<String> },
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

/// 连接ID类型
pub type ConnectionId = u64;

/// 连接订阅信息
#[derive(Debug, Clone)]
pub struct ConnectionSubscriptions {
    pub connection_id: ConnectionId,
    pub subscribed_events: HashSet<String>,
}

/// WebSocket连接管理器
#[derive(Clone)]
pub struct WebSocketManager {
    sender: broadcast::Sender<WsMessage>,
    /// 连接订阅状态 - 连接ID -> 订阅的事件类型集合
    subscriptions: Arc<RwLock<HashMap<ConnectionId, HashSet<String>>>>,
    /// 连接ID计数器
    connection_counter: Arc<AtomicU64>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            connection_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 生成新的连接ID
    pub fn generate_connection_id(&self) -> ConnectionId {
        self.connection_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// 添加连接订阅
    pub async fn add_connection(&self, connection_id: ConnectionId) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(connection_id, HashSet::new());
        log::debug!("🔌 Added connection {connection_id}");
    }

    /// 移除连接
    pub async fn remove_connection(&self, connection_id: ConnectionId) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(&connection_id);
        log::debug!("🔌 Removed connection {connection_id}");
    }

    /// 订阅事件
    pub async fn subscribe_events(&self, connection_id: ConnectionId, event_types: Vec<String>) {
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(connection_events) = subscriptions.get_mut(&connection_id) {
            for event_type in event_types.iter() {
                connection_events.insert(event_type.clone());
            }
            log::debug!("📝 Connection {connection_id} subscribed to events: {event_types:?}");
        }
    }

    /// 取消订阅事件
    pub async fn unsubscribe_events(&self, connection_id: ConnectionId, event_types: Vec<String>) {
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(connection_events) = subscriptions.get_mut(&connection_id) {
            for event_type in event_types.iter() {
                connection_events.remove(event_type);
            }
            log::debug!("📝 Connection {connection_id} unsubscribed from events: {event_types:?}");
        }
    }

    /// 广播消息给所有连接的客户端（旧方法，保持兼容性）
    pub fn broadcast(
        &self,
        message: WsMessage,
    ) -> Result<(), broadcast::error::SendError<WsMessage>> {
        self.sender.send(message).map(|_| ())
    }

    /// 根据订阅情况发送消息
    pub async fn send_to_subscribers(
        &self,
        event_type: &str,
        message: WsMessage,
    ) -> Result<usize, broadcast::error::SendError<WsMessage>> {
        let subscriptions = self.subscriptions.read().await;
        let subscriber_count = subscriptions
            .values()
            .filter(|events| events.contains(event_type))
            .count();

        if subscriber_count > 0 {
            self.sender.send(message)?;
            log::debug!("📤 Sent {event_type} event to {subscriber_count} subscribers");
            Ok(subscriber_count)
        } else {
            log::debug!("📤 No subscribers for {event_type} event, skipping");
            Ok(0)
        }
    }

    /// 获取接收器
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.sender.subscribe()
    }

    /// 获取连接的订阅信息（用于调试）
    pub async fn get_connection_subscriptions(
        &self,
        connection_id: ConnectionId,
    ) -> Option<HashSet<String>> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(&connection_id).cloned()
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

    // 生成连接ID并注册连接
    let connection_id = ws_manager.generate_connection_id();
    ws_manager.add_connection(connection_id).await;

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

    // 立即推送当前LED状态（WebSocket连接建立时）
    {
        use crate::led_status_manager::LedStatusManager;
        let led_status_manager = LedStatusManager::global().await;
        let current_status = led_status_manager.get_status().await;

        let status_message = WsMessage::LedStatusChanged {
            status: serde_json::to_value(&current_status).unwrap_or_default(),
        };

        if sender
            .send(Message::Text(
                serde_json::to_string(&status_message).unwrap(),
            ))
            .await
            .is_err()
        {
            log::warn!("❌ Failed to send initial LED status message");
        } else {
            log::info!("✅ Initial LED status message sent to WebSocket client");
        }
    }

    // 处理客户端消息的任务
    let ws_manager_for_recv = ws_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Ping => {
                                log::debug!("收到WebSocket心跳");
                            }
                            WsMessage::Subscribe { event_types } => {
                                log::debug!("收到订阅请求: {event_types:?}");
                                ws_manager_for_recv
                                    .subscribe_events(connection_id, event_types.clone())
                                    .await;

                                // 发送订阅确认
                                let confirmation = WsMessage::SubscriptionConfirmed { event_types };
                                if let Err(e) = ws_manager_for_recv.broadcast(confirmation) {
                                    log::warn!("发送订阅确认失败: {e}");
                                }
                            }
                            WsMessage::Unsubscribe { event_types } => {
                                log::debug!("收到取消订阅请求: {event_types:?}");
                                ws_manager_for_recv
                                    .unsubscribe_events(connection_id, event_types)
                                    .await;
                            }
                            _ => {
                                // 处理其他客户端消息
                                log::debug!("收到WebSocket消息: {ws_msg:?}");
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

        // 连接关闭时清理订阅
        ws_manager_for_recv.remove_connection(connection_id).await;
    });

    // 广播消息给客户端的任务
    let mut send_task = tokio::spawn(async move {
        // 实现从ws_receiver接收广播消息并发送给客户端
        while let Ok(msg) = ws_receiver.recv().await {
            log::debug!("📤 Sending WebSocket message: {msg:?}");
            let text = match serde_json::to_string(&msg) {
                Ok(text) => text,
                Err(e) => {
                    log::error!("序列化WebSocket消息失败: {e}");
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
