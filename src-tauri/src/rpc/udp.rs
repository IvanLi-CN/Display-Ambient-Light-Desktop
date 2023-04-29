use std::{collections::HashSet, sync::Arc, time::Duration};

use mdns_sd::{ServiceDaemon, ServiceEvent};
use paris::{error, info, warn};
use tokio::{
    net::UdpSocket,
    sync::{Mutex, OnceCell},
};

use super::BoardInfo;

#[derive(Debug, Clone)]
pub struct UdpRpc {
    socket: Arc<Mutex<UdpSocket>>,
    boards: Arc<Mutex<HashSet<BoardInfo>>>,
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
        let socket = Arc::new(Mutex::new(socket));
        let boards = Arc::new(Mutex::new(HashSet::new()));
        Ok(Self { socket, boards })
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

    pub async fn search_boards(&self) -> anyhow::Result<()> {
        let mdns = ServiceDaemon::new()?;
        let shared_self = Arc::new(Mutex::new(self.clone()));

        let service_type = "_ambient_light._udp.local.";

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
                    let mut boards = shared_self.boards.clone().lock_owned().await;

                    let board = BoardInfo {
                        name: info.get_fullname().to_string(),
                        address: info.get_addresses().iter().next().unwrap().clone(),
                        port: info.get_port(),
                    };

                    if boards.insert(board.clone()) {
                        info!("added board {:?}", board);
                    }
                }
                other_event => {
                    warn!("{:?}", &other_event);
                }
            }
        }

        Ok(())
    }
}
