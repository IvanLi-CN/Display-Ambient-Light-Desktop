use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use futures::future::join_all;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use paris::{error, info, warn};
use tokio::sync::{watch, OnceCell, RwLock};

use super::{Board, BoardInfo};

#[derive(Debug, Clone)]
pub struct UdpRpc {
    boards: Arc<RwLock<HashMap<String, Board>>>,
    boards_change_sender: Arc<watch::Sender<Vec<BoardInfo>>>,
}

impl UdpRpc {
    pub async fn global() -> &'static anyhow::Result<Self> {
        static UDP_RPC: OnceCell<anyhow::Result<UdpRpc>> = OnceCell::const_new();

        UDP_RPC
            .get_or_init(|| async {
                let udp_rpc = UdpRpc::new().await?;
                udp_rpc.initialize().await;
                Ok(udp_rpc)
            })
            .await
    }

    async fn new() -> anyhow::Result<Self> {
        let boards = Arc::new(RwLock::new(HashMap::new()));
        let (boards_change_sender, _) = watch::channel(Vec::new());
        let boards_change_sender = Arc::new(boards_change_sender);

        Ok(Self {
            boards,
            boards_change_sender,
        })
    }

    async fn initialize(&self) {
        let shared_self = Arc::new(self.clone());

        // 添加虚拟设备用于调试
        self.add_virtual_debug_device().await;

        let shared_self_for_search = shared_self.clone();
        tokio::spawn(async move {
            loop {
                match shared_self_for_search.search_boards().await {
                    Ok(_) => {
                        info!("search_boards finished");
                    }
                    Err(err) => {
                        error!("search_boards failed: {:?}", err);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        let shared_self_for_check = shared_self.clone();
        tokio::spawn(async move {
            shared_self_for_check.check_boards().await;
        });
    }

    /// 添加虚拟调试设备到设备列表
    async fn add_virtual_debug_device(&self) {
        use std::net::Ipv4Addr;

        // 检查是否在调试模式或开发环境
        if cfg!(debug_assertions) {
            let virtual_board_info = BoardInfo::new(
                "Virtual LED Board Debug._ambient_light._udp.local.".to_string(),
                "virtual-led-board-debug.local.".to_string(),
                Ipv4Addr::new(127, 0, 0, 1),
                8888,
            );

            let mut virtual_board = Board::new(virtual_board_info.clone());

            // 尝试初始化socket连接到虚拟设备
            match virtual_board.init_socket().await {
                Ok(_) => {
                    info!(
                        "✅ Virtual debug device added successfully: {:?}",
                        virtual_board_info
                    );

                    let mut boards = self.boards.write().await;
                    boards.insert(virtual_board_info.fullname.clone(), virtual_board);

                    // 通知设备列表更新
                    let tx_boards = boards
                        .values()
                        .map(|it| async move { it.info.read().await.clone() });
                    let tx_boards = futures::future::join_all(tx_boards).await;

                    drop(boards);

                    if let Err(err) = self.boards_change_sender.send(tx_boards) {
                        error!("Failed to notify board list change: {:?}", err);
                    }
                }
                Err(err) => {
                    warn!("⚠️ Virtual debug device not available (this is normal if virtual board is not running): {:?}", err);
                }
            }
        }
    }

    async fn search_boards(&self) -> anyhow::Result<()> {
        let service_type = "_ambient_light._udp.local.";
        let mdns = ServiceDaemon::new()?;
        let receiver = mdns.browse(&service_type).map_err(|e| {
            warn!("Failed to browse for {:?}: {:?}", service_type, e);
            e
        })?;
        let sender = self.boards_change_sender.clone();

        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    info!(
                        "Resolved a new service: {} host: {} port: {} IP: {:?} TXT properties: {:?}",
                        info.get_fullname(),
                        info.get_hostname(),
                        info.get_port(),
                        info.get_addresses(),
                        info.get_properties(),
                    );

                    let mut boards = self.boards.write().await;

                    let board_info = BoardInfo::new(
                        info.get_fullname().to_string(),
                        info.get_hostname().to_string(),
                        info.get_addresses().iter().next().unwrap().clone(),
                        info.get_port(),
                    );

                    let mut board = Board::new(board_info.clone());

                    if let Err(err) = board.init_socket().await {
                        error!("failed to init socket: {:?}", err);
                        continue;
                    }

                    if boards.insert(board_info.fullname.clone(), board).is_some() {
                        info!("replace board {:?}", board_info);
                    } else {
                        info!("add board {:?}", board_info);
                    }

                    let tx_boards = boards
                        .values()
                        .map(|it| async move { it.info.read().await.clone() });
                    let tx_boards = join_all(tx_boards).await;

                    drop(boards);

                    sender.send(tx_boards)?;
                }
                ServiceEvent::ServiceRemoved(_, fullname) => {
                    info!("removed board {:?}", fullname);
                    let mut boards = self.boards.write().await;
                    if boards.remove(&fullname).is_some() {
                        info!("removed board {:?} successful", fullname);
                    }

                    let tx_boards = boards
                        .values()
                        .map(|it| async move { it.info.read().await.clone() });
                    let tx_boards = join_all(tx_boards).await;

                    drop(boards);

                    sender.send(tx_boards)?;
                }
                _other_event => {
                    // log::info!("{:?}", &other_event);
                }
            }

            tokio::task::yield_now().await;
        }

        Ok(())
    }

    pub fn subscribe_boards_change(&self) -> watch::Receiver<Vec<BoardInfo>> {
        self.boards_change_sender.subscribe()
    }

    pub async fn get_boards(&self) -> Vec<BoardInfo> {
        self.boards_change_sender.borrow().clone()
    }

    pub async fn send_to_all(&self, buff: &Vec<u8>) -> anyhow::Result<()> {
        let boards = self.boards.read().await;

        if boards.is_empty() {
            log::debug!("No boards available to send colors to");
            return Ok(());
        }

        log::debug!("Sending {} bytes to {} boards", buff.len(), boards.len());

        for board in boards.values() {
            board.send_colors(buff).await;
        }

        Ok(())
    }

    pub async fn send_to(&self, buff: &Vec<u8>, target_addr: SocketAddr) -> anyhow::Result<()> {
        let boards = self.boards.read().await;

        if boards.is_empty() {
            log::debug!("No boards available to send colors to");
            return Err(anyhow::anyhow!("No boards available"));
        }

        log::info!("🔍 Looking for target board: {}", target_addr);
        log::info!("📋 Available boards:");
        for (name, board) in boards.iter() {
            if let Some(socket_addr) = board.get_socket_addr() {
                log::info!(
                    "  - {}: {} (match: {})",
                    name,
                    socket_addr,
                    socket_addr == target_addr
                );
            } else {
                log::info!("  - {}: No socket address", name);
            }
        }

        let target_board = boards.values().find(|board| {
            if let Some(socket_addr) = board.get_socket_addr() {
                socket_addr == target_addr
            } else {
                false
            }
        });

        if let Some(board) = target_board {
            log::info!(
                "✅ Found target board! Sending {} bytes to: {}",
                buff.len(),
                target_addr
            );
            board.send_colors(buff).await;
            Ok(())
        } else {
            warn!("❌ Target board with address {} not found", target_addr);
            Err(anyhow::anyhow!("Target board not found"))
        }
    }

    /// 直接发送数据到指定地址，不检查设备列表（用于调试和测试）
    pub async fn send_to_direct(
        &self,
        buff: &Vec<u8>,
        target_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        log::info!(
            "🚀 Direct send: {} bytes to {} (bypassing device check)",
            buff.len(),
            target_addr
        );

        // 创建临时UDP socket直接发送
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

        match socket.send_to(buff, target_addr).await {
            Ok(bytes_sent) => {
                log::info!(
                    "✅ Direct send successful: {} bytes sent to {}",
                    bytes_sent,
                    target_addr
                );
                Ok(())
            }
            Err(err) => {
                error!("❌ Direct send failed to {}: {}", target_addr, err);
                Err(anyhow::anyhow!("Direct send failed: {}", err))
            }
        }
    }

    pub async fn check_boards(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::task::yield_now().await;
            interval.tick().await;

            let boards = self.boards.read().await;

            if boards.is_empty() {
                info!("no boards found");
                continue;
            }

            // Store previous board states to detect changes
            let prev_boards = boards
                .values()
                .map(|it| async move { it.info.read().await.clone() });
            let prev_boards = join_all(prev_boards).await;

            // Check all boards
            for board in boards.values() {
                if let Err(err) = board.check().await {
                    error!("failed to check board: {:?}", err);
                }
            }

            // Get current board states after check
            let current_boards = boards
                .values()
                .map(|it| async move { it.info.read().await.clone() });
            let current_boards = join_all(current_boards).await;

            drop(boards);

            // Only send update if there are actual changes
            let has_changes = prev_boards.len() != current_boards.len()
                || prev_boards
                    .iter()
                    .zip(current_boards.iter())
                    .any(|(prev, current)| {
                        prev.connect_status != current.connect_status
                            || prev.ttl != current.ttl
                            || prev.checked_at != current.checked_at
                    });

            if has_changes {
                let board_change_sender = self.boards_change_sender.clone();
                if let Err(err) = board_change_sender.send(current_boards) {
                    error!("failed to send board change: {:?}", err);
                }
                drop(board_change_sender);
            }
        }
    }
}
