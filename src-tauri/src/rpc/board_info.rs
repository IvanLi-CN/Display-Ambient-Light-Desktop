use std::net::{Ipv4Addr};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BoardInfo {
    pub name: String,
    pub address: Ipv4Addr,
    pub port: u16,
}