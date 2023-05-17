use super::{
    bitfield_services::BitfieldServices, 
    compact_size::CompactSize
};

use crate::serialization::{
    deserializable::Deserializable, deserializable_big_endian::DeserializableBigEndian,
    deserializable_fix_size::DeserializableFixSize, error_serialization::ErrorSerialization,
    serializable::Serializable, serializable_big_endian::SerializableBigEndian,
};

use super::{
    message::Message,
    command_name::CommandName,
};

use std::net::{
    Ipv6Addr, 
    SocketAddr
};

use chrono::{
    DateTime,
    offset::Utc
};

use std::io::{
    Read, 
    Write
};

use crate::connections::{
    p2p_protocol::ProtocolVersionP2P, 
    socket_conversion::socket_to_ipv6_port,
};

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

impl VersionMessage {
    pub fn new(
        version: ProtocolVersionP2P,
        services: BitfieldServices,
        recv_services: BitfieldServices,
        recv_socket_addr: &SocketAddr,
        trans_socket_addr: &SocketAddr,
        nonce: u64,
        user_agent: String,
        start_height: i32,
        relay: bool,
    ) -> Self {
        let timestamp = Utc::now();
        let (recv_addr, recv_port) = socket_to_ipv6_port(recv_socket_addr);
        let (trans_addr, trans_port) = socket_to_ipv6_port(trans_socket_addr);

        Self {
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
        }
    }
}

impl Message for VersionMessage {
    fn get_command_name() -> CommandName {
        CommandName::Version
    }
}

impl Serializable for VersionMessage {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization>{
      
        self.version.serialize(stream)?;
        self.services.serialize(stream)?;
        self.timestamp.serialize(stream)?;
        self.recv_services.serialize(stream)?;

        self.recv_addr.serialize_big_endian(stream)?;
        self.recv_port.serialize_big_endian(stream)?;

        self.services.serialize(stream)?;

        self.trans_addr.serialize_big_endian(stream)?;
        self.trans_port.serialize_big_endian(stream)?;

        self.nonce.serialize(stream)?;
        CompactSize::new(self.user_agent.len() as u64).serialize(stream)?;
        self.user_agent.serialize(stream)?;
        self.start_height.serialize(stream)?;
        self.relay.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for VersionMessage {

    fn deserialize(stream: &mut dyn Read) ->  Result<Self, ErrorSerialization> {

        let version = ProtocolVersionP2P::deserialize(stream)?;
        let services = BitfieldServices::deserialize(stream)?;
        let timestamp = DateTime::<Utc>::deserialize(stream)?;
        let recv_services = BitfieldServices::deserialize(stream)?;

        let recv_addr = Ipv6Addr::deserialize_big_endian(stream)?;
        let recv_port = u16::deserialize_big_endian(stream)?;

        let trans_services = BitfieldServices::deserialize(stream)?;
        if trans_services != services {
            return Err(ErrorSerialization::ErrorInDeserialization(format!(
                "Transceiver service isn't the same as the service: {:?}",
                trans_services
            )));
        }

        let trans_addr = Ipv6Addr::deserialize_big_endian(stream)?;
        let trans_port = u16::deserialize_big_endian(stream)?;

        let nonce = u64::deserialize(stream)?;
        let user_agent_len = CompactSize::deserialize(stream)?;
        let user_agent = String::deserialize_fix_size(stream, user_agent_len.value as usize)?;
        let start_height = i32::deserialize(stream)?;
        let relay = bool::deserialize(stream)?;

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
        connections::{p2p_protocol::ProtocolVersionP2P, suppored_services::SupportedServices},
        messages::{
            bitfield_services::BitfieldServices, compact_size::CompactSize,
            error_message::ErrorMessage,
        },
        serialization::{
            deserializable::Deserializable, serializable::Serializable,
            serializable_big_endian::SerializableBigEndian,
        },
    };

    use super::VersionMessage;

    use chrono::{offset::Utc, DateTime, NaiveDateTime};
  
    use std::net::Ipv6Addr;

    #[test]
    fn test01_serialize() -> Result<(), ErrorMessage>{

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
        
        version.serialize(&mut expected_stream)?;
        services.serialize(&mut expected_stream)?;
        timestamp.serialize(&mut expected_stream)?;
        recv_services.serialize(&mut expected_stream)?;
        
        recv_addr.serialize_big_endian(&mut expected_stream)?;
        recv_port.serialize_big_endian(&mut expected_stream)?;

        services.serialize(&mut expected_stream)?;

        trans_addr.serialize_big_endian(&mut expected_stream)?;
        trans_port.serialize_big_endian(&mut expected_stream)?; 
        
        nonce.serialize(&mut expected_stream)?;
        length.serialize(&mut expected_stream)?; 
        user_agent.serialize(&mut expected_stream)?;
        start_height.serialize(&mut expected_stream)?; 
        relay.serialize(&mut expected_stream)?;

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

        version_message.serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize() -> Result<(), ErrorMessage> {
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

        version.serialize(&mut stream)?;
        services.serialize(&mut stream)?;
        timestamp.serialize(&mut stream)?;
        recv_services.serialize(&mut stream)?;
        
        recv_addr.serialize_big_endian(&mut stream)?;
        recv_port.serialize_big_endian(&mut stream)?;

        services.serialize(&mut stream)?;

        trans_addr.serialize_big_endian(&mut stream)?;
        trans_port.serialize_big_endian(&mut stream)?; 
        
        nonce.serialize(&mut stream)?;
        length.serialize(&mut stream)?; 
        user_agent.serialize(&mut stream)?;
        start_height.serialize(&mut stream)?; 
        relay.serialize(&mut stream)?;
        
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

        let version = VersionMessage::deserialize(&mut stream)?;

        assert_eq!(version_esperado, version);

        Ok(())
    }
}
