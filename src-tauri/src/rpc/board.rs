use std::{sync::Arc, time::Duration};

use paris::{info, warn, error};
use tokio::{net::UdpSocket, sync::RwLock, time::timeout, io};

use super::{BoardConnectStatus, BoardInfo};

#[derive(Debug)]
pub struct Board {
    pub info: Arc<RwLock<BoardInfo>>,
    socket: Option<Arc<UdpSocket>>,
    listen_handler: Option<tokio::task::JoinHandle<()>>,
}

impl Board {
    pub fn new(info: BoardInfo) -> Self {
        Self {
            info: Arc::new(RwLock::new(info)),
            socket: None,
            listen_handler: None,
        }
    }

    pub async fn init_socket(&mut self) -> anyhow::Result<()> {
        let info = self.info.read().await;
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        socket.connect((info.address, info.port)).await?;
        let socket = Arc::new(socket);
        self.socket = Some(socket.clone());

        let info = self.info.clone();

        let handler=tokio::spawn(async move {
            let mut buf = [0u8; 128];
            if let Err(err) = socket.readable().await {
                error!("socket read error: {:?}", err);
                return;
            }
            loop {
                match socket.try_recv(&mut buf) {
                    Ok(len) => {
                        log::info!("recv: {:?}", &buf[..len]);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        error!("socket recv error: {:?}", e);
                        break;
                    }
                }
            }
        });
        self.listen_handler = Some(handler);

        Ok(())
    }

    pub async fn send_colors(&self, buf: &[u8]) {
        let info = self.info.read().await;
        if self.socket.is_none() || info.connect_status != BoardConnectStatus::Connected {
            return;
        }

        let socket = self.socket.as_ref().unwrap();

        socket.send(buf).await.unwrap();
    }

    pub async fn check(&self) -> anyhow::Result<()> {
        let info = self.info.read().await;
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect((info.address, info.port)).await?;
        drop(info);

        let instant = std::time::Instant::now();

        socket.send(&[1]).await?;
        let mut buf = [0u8; 1];
        let recv_future = socket.recv(&mut buf);

        let check_result = timeout(Duration::from_secs(1), recv_future).await;
        let mut info = self.info.write().await;
        match check_result {
            Ok(_) => {
                let ttl = instant.elapsed();
                if buf == [1] {
                    info.connect_status = BoardConnectStatus::Connected;
                } else {
                    if let BoardConnectStatus::Connecting(retry) = info.connect_status {
                        if retry < 10 {
                            info.connect_status = BoardConnectStatus::Connecting(retry + 1);
                            info!("reconnect: {}", retry + 1);
                        } else {
                            info.connect_status = BoardConnectStatus::Disconnected;
                            warn!("board Disconnected: bad pong.");
                        }
                    } else if info.connect_status != BoardConnectStatus::Disconnected {
                        info.connect_status = BoardConnectStatus::Connecting(1);
                    }
                }
                info.ttl = Some(ttl.as_millis());
            }
            Err(_) => {
                if let BoardConnectStatus::Connecting(retry) = info.connect_status {
                    if retry < 10 {
                        info.connect_status = BoardConnectStatus::Connecting(retry + 1);
                        info!("reconnect: {}", retry + 1);
                    } else {
                        info.connect_status = BoardConnectStatus::Disconnected;
                        warn!("board Disconnected: timeout");
                    }
                } else if info.connect_status != BoardConnectStatus::Disconnected {
                    info.connect_status = BoardConnectStatus::Connecting(1);
                }

                info.ttl = None;
            }
        }

        info.checked_at = Some(std::time::SystemTime::now());

        Ok(())
    }
}


impl Drop for Board {
    fn drop(&mut self) {
        if let Some(handler) = self.listen_handler.take() {
            info!("aborting listen handler");
            tokio::task::block_in_place(move || {
                handler.abort();
            });
            info!("listen handler aborted");
        }
    }
}