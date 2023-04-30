use std::{net::Ipv4Addr, time::Duration};

use paris::warn;
use serde::{Deserialize, Serialize};
use tokio::{net::UdpSocket, time::timeout};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BoardInfo {
    pub host: String,
    pub address: Ipv4Addr,
    pub port: u16,
    pub is_online: bool,
    pub checked_at: Option<std::time::SystemTime>,
    pub ttl: Option<u128>,
}

impl BoardInfo {
    pub fn new(host: String, address: Ipv4Addr, port: u16) -> Self {
        Self {
            host,
            address,
            port,
            is_online: false,
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

        match timeout(Duration::from_secs(5), recv_future).await {
            Ok(_) => {
                let ttl = instant.elapsed();
                log::info!("buf: {:?}", buf);
                if buf == [1] {
                    self.is_online = true;
                } else {
                    self.is_online = false;
                }
                self.ttl = Some(ttl.as_millis());
            }
            Err(_) => {
                warn!("timeout");
                self.is_online = false;
                self.ttl = None;
            }
        }

        self.checked_at = Some(std::time::SystemTime::now());

        Ok(())
    }
}
