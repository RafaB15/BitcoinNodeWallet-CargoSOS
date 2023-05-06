use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::net::Ipv6Addr;
use chrono::{
    DateTime,
    offset::Utc
};

use std::io::{Read, Write};

use crate::connections::{
    p2p_protocol::ProtocolVersionP2P,
    suppored_services::SupportedServices,
};

pub const VERSION_TYPE: [u8; 12] = [118, 101, 114, 115, 105, 111, 110, 0, 0, 0, 0, 1];

pub struct VersionMessage {
    pub version: ProtocolVersionP2P,
    pub services: SupportedServices,
    pub timestamp: DateTime<Utc>,
    pub recv_services: SupportedServices,
    pub recv_addr: Ipv6Addr,
    pub recv_port: u16,
    pub trans_addr: Ipv6Addr,
    pub trans_port: u16, // tal vez es el mismo que el recv_port
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

impl VersionMessage {

    pub fn new(
        version: ProtocolVersionP2P,
        services: SupportedServices,
        timestamp: DateTime<Utc>,
        recv_services: SupportedServices,
        recv_addr: Ipv6Addr,
        recv_port: u16,
        trans_addr: Ipv6Addr,
        trans_port: u16,
        nonce: u64,
        user_agent: String,
        start_height: i32,
        relay: bool,
    ) -> Self {
        todo!();
    }
}

impl Serializable for VersionMessage {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage>{
        todo!()
    }
}

impl Deserializable for VersionMessage {
    type Value = Self;
    fn deserialize(stream: &mut dyn Read) ->  Result<Self::Value, ErrorMessage> {
        todo!()
    }
}