use std::net::IpAddr;

pub struct Node {
    pub version: i32,
    pub services: u64,
    pub peer_addr: Vec<IpAddr>,
}