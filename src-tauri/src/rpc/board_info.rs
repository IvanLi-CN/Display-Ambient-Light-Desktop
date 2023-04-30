use std::{net::Ipv4Addr, time::Duration};

use paris::{warn, info};
use serde::{Deserialize, Serialize};
use tokio::{net::UdpSocket, time::timeout};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BoardConnectStatus {
    Connected,
    Connecting(u8),
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoardInfo {
    pub host: String,
    pub address: Ipv4Addr,
    pub port: u16,
    pub connect_status: BoardConnectStatus,
    pub checked_at: Option<std::time::SystemTime>,
    pub ttl: Option<u128>,
}

impl BoardInfo {
    pub fn new(host: String, address: Ipv4Addr, port: u16) -> Self {
        Self {
            host,
            address,
            port,
            connect_status: BoardConnectStatus::Unknown,
            checked_at: None,
            ttl: None,
        }
    }

    pub async fn check(&mut self) -> anyhow::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect((self.address, self.port)).await?;

        let instant = std::time::Instant::now();

        socket.send(&[1]).await?;
        let mut buf = [0u8; 1];
        let recv_future = socket.recv(&mut buf);

        match timeout(Duration::from_secs(1), recv_future).await {
            Ok(_) => {
                let ttl = instant.elapsed();
                if buf == [1] {
                    self.connect_status = BoardConnectStatus::Connected;
                } else {
                    if let BoardConnectStatus::Connecting(retry) = self.connect_status {
                        if retry < 10 {
                            self.connect_status = BoardConnectStatus::Connecting(retry + 1);
                            info!("reconnect: {}", retry + 1);
                        } else {
                            self.connect_status = BoardConnectStatus::Disconnected;
                            warn!("board Disconnected: bad pong.");
                        }
                    } else if self.connect_status != BoardConnectStatus::Disconnected {
                        self.connect_status = BoardConnectStatus::Connecting(1);
                    }
                }
                self.ttl = Some(ttl.as_millis());
            }
            Err(_) => {
                if let BoardConnectStatus::Connecting(retry) = self.connect_status {
                    if retry < 10 {
                        self.connect_status = BoardConnectStatus::Connecting(retry + 1);
                        info!("reconnect: {}", retry + 1);
                    } else {
                        self.connect_status = BoardConnectStatus::Disconnected;
                        warn!("board Disconnected: timeout");
                    }
                } else if self.connect_status != BoardConnectStatus::Disconnected {
                    self.connect_status = BoardConnectStatus::Connecting(1);
                }

                self.ttl = None;
            }
        }

        self.checked_at = Some(std::time::SystemTime::now());

        Ok(())
    }
}
