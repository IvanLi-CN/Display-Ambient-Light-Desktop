use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BoardConnectStatus {
    Connected,
    Connecting(u8),
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoardInfo {
    pub fullname: String,
    pub host: String,
    pub address: Ipv4Addr,
    pub port: u16,
    pub connect_status: BoardConnectStatus,
    pub checked_at: Option<std::time::SystemTime>,
    pub ttl: Option<u128>,
}

impl BoardInfo {
    pub fn new(fullname: String, host: String, address: Ipv4Addr, port: u16) -> Self {
        Self {
            fullname,
            host,
            address,
            port,
            connect_status: BoardConnectStatus::Unknown,
            checked_at: None,
            ttl: None,
        }
    }
}
