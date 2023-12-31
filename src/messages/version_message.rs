use super::{
    bitfield_services::BitfieldServices, command_name::CommandName, compact_size::CompactSize,
    message::Message,
};

use crate::serialization::{
    deserializable_big_endian::DeserializableBigEndian,
    deserializable_fix_size::DeserializableFixSize,
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_big_endian::SerializableBigEndian,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::connections::p2p_protocol::ProtocolVersionP2P;

use std::{
    io::{Read, Write},
    net::Ipv6Addr,
};

use chrono::{offset::Utc, DateTime};

/// It's the version message
#[derive(Debug, std::cmp::PartialEq)]
pub struct VersionMessage {
    pub version: ProtocolVersionP2P,
    pub services: BitfieldServices,
    pub timestamp: DateTime<Utc>,
    pub recv_services: BitfieldServices,
    pub recv_addr: Ipv6Addr,
    pub recv_port: u16,
    pub trans_addr: Ipv6Addr,
    pub trans_port: u16,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

impl Message for VersionMessage {
    fn get_command_name() -> CommandName {
        CommandName::Version
    }
}

impl SerializableInternalOrder for VersionMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.le_serialize(stream)?;
        self.services.le_serialize(stream)?;
        self.timestamp.le_serialize(stream)?;
        self.recv_services.le_serialize(stream)?;

        self.recv_addr.be_serialize(stream)?;
        self.recv_port.be_serialize(stream)?;

        self.services.le_serialize(stream)?;

        self.trans_addr.be_serialize(stream)?;
        self.trans_port.be_serialize(stream)?;

        self.nonce.le_serialize(stream)?;
        CompactSize::new(self.user_agent.len() as u64).le_serialize(stream)?;
        self.user_agent.le_serialize(stream)?;
        self.start_height.le_serialize(stream)?;
        self.relay.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for VersionMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version = ProtocolVersionP2P::le_deserialize(stream)?;
        let services = BitfieldServices::le_deserialize(stream)?;
        let timestamp = DateTime::<Utc>::le_deserialize(stream)?;
        let recv_services = BitfieldServices::le_deserialize(stream)?;

        let recv_addr = Ipv6Addr::be_deserialize(stream)?;
        let recv_port = u16::be_deserialize(stream)?;

        let trans_services = BitfieldServices::le_deserialize(stream)?;
        if trans_services != services {
            return Err(ErrorSerialization::ErrorInDeserialization(format!(
                "Transceiver service isn't the same as the service: {:?}",
                trans_services
            )));
        }

        let trans_addr = Ipv6Addr::be_deserialize(stream)?;
        let trans_port = u16::be_deserialize(stream)?;

        let nonce = u64::le_deserialize(stream)?;
        let user_agent_len = CompactSize::le_deserialize(stream)?;
        let user_agent = String::deserialize_fix_size(stream, user_agent_len.value as usize)?;
        let start_height = i32::le_deserialize(stream)?;
        let relay = bool::le_deserialize(stream)?;

        Ok(VersionMessage {
            version,
            services,
            timestamp,
            recv_services,
            recv_addr,
            recv_port,
            trans_addr,
            trans_port,
            nonce,
            user_agent,
            start_height,
            relay,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        connections::{p2p_protocol::ProtocolVersionP2P, supported_services::SupportedServices},
        messages::{bitfield_services::BitfieldServices, compact_size::CompactSize},
        serialization::error_serialization::ErrorSerialization,
    };

    use super::{
        DeserializableInternalOrder, SerializableBigEndian, SerializableInternalOrder,
        SerializableLittleEndian, VersionMessage,
    };

    use chrono::{offset::Utc, DateTime, NaiveDateTime};

    use std::net::Ipv6Addr;

    #[test]
    fn test01_serialize() -> Result<(), ErrorSerialization> {
        let version = ProtocolVersionP2P::V31402;
        let services = BitfieldServices::new(vec![SupportedServices::NodeNetworkLimited]);

        let naive = NaiveDateTime::from_timestamp_opt(1628, 0).unwrap();
        let timestamp: DateTime<Utc> = DateTime::<Utc>::from_utc(naive, Utc);

        let recv_services = BitfieldServices::new(vec![SupportedServices::NodeNetworkLimited]);
        let recv_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let recv_port: u16 = 80;
        let trans_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let trans_port: u16 = 64;
        let nonce: u64 = 00001111;
        let user_agent: String = "abc".to_string();
        let length = CompactSize::new(user_agent.len() as u64);
        let start_height: i32 = 3;
        let relay: bool = false;
        let mut stream: Vec<u8> = Vec::new();

        let mut expected_stream: Vec<u8> = Vec::new();

        version.le_serialize(&mut expected_stream)?;
        services.le_serialize(&mut expected_stream)?;
        timestamp.le_serialize(&mut expected_stream)?;
        recv_services.le_serialize(&mut expected_stream)?;

        recv_addr.be_serialize(&mut expected_stream)?;
        recv_port.be_serialize(&mut expected_stream)?;

        services.le_serialize(&mut expected_stream)?;

        trans_addr.be_serialize(&mut expected_stream)?;
        trans_port.be_serialize(&mut expected_stream)?;

        nonce.le_serialize(&mut expected_stream)?;
        length.le_serialize(&mut expected_stream)?;
        user_agent.le_serialize(&mut expected_stream)?;
        start_height.le_serialize(&mut expected_stream)?;
        relay.le_serialize(&mut expected_stream)?;

        let version_message = VersionMessage {
            version,
            services,
            timestamp,
            recv_services,
            recv_addr,
            recv_port,
            trans_addr,
            trans_port,
            nonce,
            user_agent,
            start_height,
            relay,
        };

        version_message.io_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize() -> Result<(), ErrorSerialization> {
        let version = ProtocolVersionP2P::V31402;
        let services = BitfieldServices::new(vec![SupportedServices::NodeNetworkLimited]);

        let naive = NaiveDateTime::from_timestamp_opt(1628, 0).unwrap();
        let timestamp: DateTime<Utc> = DateTime::<Utc>::from_utc(naive, Utc);

        let recv_services = BitfieldServices::new(vec![SupportedServices::NodeNetworkLimited]);
        let recv_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let recv_port: u16 = 80;
        let trans_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let trans_port: u16 = 64;
        let nonce: u64 = 00001111;
        let user_agent: String = "abc".to_string();
        let length = CompactSize::new(user_agent.len() as u64);
        let start_height: i32 = 3;
        let relay: bool = false;

        let mut stream: Vec<u8> = Vec::new();

        version.le_serialize(&mut stream)?;
        services.le_serialize(&mut stream)?;
        timestamp.le_serialize(&mut stream)?;
        recv_services.le_serialize(&mut stream)?;

        recv_addr.be_serialize(&mut stream)?;
        recv_port.be_serialize(&mut stream)?;

        services.le_serialize(&mut stream)?;

        trans_addr.be_serialize(&mut stream)?;
        trans_port.be_serialize(&mut stream)?;

        nonce.le_serialize(&mut stream)?;
        length.le_serialize(&mut stream)?;
        user_agent.le_serialize(&mut stream)?;
        start_height.le_serialize(&mut stream)?;
        relay.le_serialize(&mut stream)?;

        let mut stream: &[u8] = &stream;

        let version_esperado = VersionMessage {
            version,
            services,
            timestamp,
            recv_services,
            recv_addr,
            recv_port,
            trans_addr,
            trans_port,
            nonce,
            user_agent,
            start_height,
            relay,
        };

        let version = VersionMessage::io_deserialize(&mut stream)?;

        assert_eq!(version_esperado, version);

        Ok(())
    }
}
