use super::{
    serializable::Serializable,
    deserializable::Deserializable,
};
use std::net::Ipv6Addr;

use crate::connections::{
    p2p_protocol::ProtocolVersionP2P,
    suppored_services::SupportedServices,
};

pub struct VersionMessage {
    pub version: ProtocolVersionP2P,
    pub services: SupportedServices,
    pub timestamp: i64,
    pub recv_services: u64,
    pub recv_addr: Ipv6Addr,
    pub recv_port: u16,
    pub trans_services: u64,
    pub trans_addr: Ipv6Addr,
    pub trans_port: u16,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

impl VersionMessage {

    pub fn new(
        version: ProtocolVersionP2P,
        services: SupportedServices,
    ) -> Self {
        todo!();
    }
}

impl Serializable for VersionMessage {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

impl Deserializable for VersionMessage {
    fn deserialize(data: Vec<u8>) -> Self {
        todo!()
    }
}