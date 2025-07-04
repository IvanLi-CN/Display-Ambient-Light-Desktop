use std::{collections::HashMap, sync::Arc, time::Duration};

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
                other_event => {
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

        for board in boards.values() {
            board.send_colors(buff).await;
        }

        // let socket = self.socket.clone();

        // let handlers = boards.into_iter().map(|board| {
        //     if board.connect_status == BoardConnectStatus::Disconnected {
        //         return tokio::spawn(async move {
        //             log::debug!("board {} is disconnected, skip.", board.host);
        //         });
        //     }

        //     let socket = socket.clone();
        //     let buff = buff.clone();
        //     tokio::spawn(async move {
        //         match socket.send_to(&buff, (board.address, board.port)).await {
        //             Ok(_) => {}
        //             Err(err) => {
        //                 error!("failed to send to {}: {:?}", board.host, err);
        //             }
        //         }
        //     })
        // });

        // join_all(handlers).await;

        Ok(())
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

            for board in boards.values() {
                if let Err(err) = board.check().await {
                    error!("failed to check board: {:?}", err);
                }
            }

            let tx_boards = boards
                .values()
                .map(|it| async move { it.info.read().await.clone() });
            let tx_boards = join_all(tx_boards).await;

            drop(boards);

            let board_change_sender = self.boards_change_sender.clone();
            if let Err(err) = board_change_sender.send(tx_boards) {
                error!("failed to send board change: {:?}", err);
            }

            drop(board_change_sender);
        }
    }
}
