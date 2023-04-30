use std::{
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
    sync::Arc,
    time::Duration,
};

use futures::future::join_all;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use paris::{error, info, warn};
use tokio::{
    net::UdpSocket,
    sync::{watch, OnceCell, RwLock},
};

use super::{BoardConnectStatus, BoardInfo};

#[derive(Debug, Clone)]
pub struct UdpRpc {
    socket: Arc<UdpSocket>,
    boards: Arc<RwLock<HashMap<Ipv4Addr, BoardInfo>>>,
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
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let socket = Arc::new(socket);
        let boards = Arc::new(RwLock::new(HashMap::new()));
        let (boards_change_sender, _) = watch::channel(Vec::new());
        let boards_change_sender = Arc::new(boards_change_sender);

        Ok(Self {
            socket,
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

        // let shared_self_for_watch = shared_self.clone();
        // tokio::spawn(async move {
        //     let mut rx = shared_self_for_watch.clone_boards_change_receiver().await;

        //     // let mut rx  = sub_tx.subscribe();
        //     // drop(sub_tx);
        //     while rx.changed().await.is_ok() {
        //         let boards = rx.borrow().clone();
        //         info!("boards changed: {:?}", boards);
        //     }
        // });
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

                    let board = BoardInfo::new(
                        info.get_fullname().to_string(),
                        info.get_addresses().iter().next().unwrap().clone(),
                        info.get_port(),
                    );

                    if boards.insert(board.address, board.clone()).is_some() {
                        info!("added board {:?}", board);
                    }

                    let tx_boards = boards.values().cloned().collect();
                    drop(boards);

                    sender.send(tx_boards)?;
                    tokio::task::yield_now().await;
                }
                other_event => {
                    warn!("{:?}", &other_event);
                }
            }
        }

        Ok(())
    }

    pub fn subscribe_boards_change(&self) -> watch::Receiver<Vec<BoardInfo>> {
        self.boards_change_sender.subscribe()
    }

    pub async fn get_boards(&self) -> Vec<BoardInfo> {
        let boards = self.boards.read().await;
        boards.values().cloned().collect()
    }

    pub async fn send_to_all(&self, buff: &Vec<u8>) -> anyhow::Result<()> {
        let boards = self.get_boards().await;
        let socket = self.socket.clone();

        let handlers = boards.into_iter().map(|board| {
            let socket = socket.clone();
            let buff = buff.clone();
            tokio::spawn(async move {
                match socket.send_to(&buff, (board.address, board.port)).await {
                    Ok(_) => {}
                    Err(err) => {
                        error!("failed to send to {}: {:?}", board.host, err);
                    }
                }
            })
        });

        join_all(handlers).await;

        Ok(())
    }

    pub async fn check_boards(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            let mut boards = self.boards.clone().write_owned().await;

            if boards.is_empty() {
                info!("no boards found");
                continue;
            }

            for (_, board) in boards.iter_mut() {
                if let Err(err) = board.check().await {
                    error!("failed to check board {}: {:?}", board.host, err);
                }
            }

            let board_vec = boards.values().cloned().collect::<Vec<_>>();
            drop(boards);

            let board_change_sender = self.boards_change_sender.clone();
            if let Err(err) = board_change_sender.send(board_vec) {
                error!("failed to send board change: {:?}", err);
            }
            drop(board_change_sender);
            interval.tick().await;
        }
    }
}
