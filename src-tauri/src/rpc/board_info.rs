use std::net::{Ipv4Addr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BoardInfo {
    pub name: String,
    pub address: Ipv4Addr,
    pub port: u16,
}