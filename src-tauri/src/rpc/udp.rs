use std::{collections::HashSet, sync::Arc, time::Duration};

use futures::future::join_all;
use itertools::Itertools;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use paris::{error, info, warn};
use tokio::{
    net::UdpSocket,
    sync::{watch, Mutex, OnceCell, RwLock},
};

use super::BoardInfo;

#[derive(Debug, Clone)]
pub struct UdpRpc {
    socket: Arc<UdpSocket>,
    boards: Arc<RwLock<HashSet<BoardInfo>>>,
    boards_change_sender: Arc<Mutex<watch::Sender<HashSet<BoardInfo>>>>,
    boards_change_receiver: Arc<Mutex<watch::Receiver<HashSet<BoardInfo>>>>,
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
        let boards = Arc::new(RwLock::new(HashSet::new()));
        let (boards_change_sender, boards_change_receiver) = watch::channel(HashSet::new());
        let boards_change_sender = Arc::new(Mutex::new(boards_change_sender));
        let boards_change_receiver = Arc::new(Mutex::new(boards_change_receiver));
        Ok(Self {
            socket,
            boards,
            boards_change_sender,
            boards_change_receiver,
        })
    }

    async fn initialize(&self) {
        let shared_self = Arc::new(self.clone());
        tokio::spawn(async move {
            loop {
                match shared_self.search_boards().await {
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
    }

    async fn search_boards(&self) -> anyhow::Result<()> {
        let service_type = "_ambient_light._udp.local.";
        let mdns = ServiceDaemon::new()?;
        let shared_self = Arc::new(Mutex::new(self.clone()));
        let receiver = mdns.browse(&service_type).map_err(|e| {
            warn!("Failed to browse for {:?}: {:?}", service_type, e);
            e
        })?;

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

                    let shared_self = shared_self.lock().await;
                    let mut boards = shared_self.boards.write().await;

                    let board = BoardInfo {
                        name: info.get_fullname().to_string(),
                        address: info.get_addresses().iter().next().unwrap().clone(),
                        port: info.get_port(),
                    };

                    if boards.insert(board.clone()) {
                        info!("added board {:?}", board);
                    }

                    let sender = self.boards_change_sender.clone().lock_owned().await;
                    sender.send(boards.clone())?;
                }
                other_event => {
                    warn!("{:?}", &other_event);
                }
            }
        }

        Ok(())
    }

    pub async fn clone_boards_change_receiver(
        &self,
    ) -> watch::Receiver<HashSet<BoardInfo>> {
        let boards_change_receiver = self.boards_change_receiver.clone().lock_owned().await;
        boards_change_receiver.clone()
    }

    pub async fn get_boards(&self) -> HashSet<BoardInfo> {
        let boards = self.boards.read().await;
        boards.clone()
    }

    pub async fn send_to_all(&self, buff: &Vec<u8>) -> anyhow::Result<()> {
        let boards = self.get_boards().await;
        let socket = self.socket.clone();
        
        let handlers = boards.into_iter()
        .map(|board| {
            let socket = socket.clone();
            let buff = buff.clone();
            tokio::spawn(async move {
                match socket.send_to(&buff, (board.address, board.port)).await {
                    Ok(_) => {},
                    Err(err) => {
                        error!("failed to send to {}: {:?}", board.name, err);
                    },
                }
            })
        });

        join_all(handlers).await;

        Ok(())
    }
}
