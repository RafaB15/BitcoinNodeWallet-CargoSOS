use std::net::Ipv6Addr;

pub struct VersionMessage {
    pub version: i32,
    pub services: u64,
    pub timestamp: i64,
    pub recv_services: u64,
    pub recv_addr: Ipv6Addr,
    pub recv_port: u16,
    pub trans_services: u64,
    pub trans_addr: Ipv6Addr,
    pub trans_port: u16,
    pub nonce: u64,
    pub start_height: i32,
    pub relay: bool,
}