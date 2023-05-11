use super::{
    compact_size::CompactSize, 
    serializable::Serializable,
    serializable_big_endian::SerializableBigEndian,
    deserializable::Deserializable,
    deserializable_big_endian::DeserializableBigEndian, 
    deserializable_fix_size::DeserializableFixSize, 
    error_message::ErrorMessage, 
};

use std::net::{Ipv6Addr, SocketAddr};
use chrono::{
    DateTime,
    offset::Utc
};

use std::io::{Read, Write};

use crate::connections::{
    p2p_protocol::ProtocolVersionP2P,
    suppored_services::SupportedServices,
    socket_conversion::socket_to_ipv6_port,
};

use bitcoin_hashes::{
    sha256d,
    Hash,
};

pub const VERSION_TYPE: &[u8; 12] = b"version\0\0\0\0\0";

const MAGIC_BYTES_SIZE: usize = 4;
const MASSAGE_TYPE_SIZE: usize = 12;
const PAYLOAD_SIZE: usize = 4;
const CHECKSUM_SIZE: usize = 4;

const HEADER_SIZE: usize = MAGIC_BYTES_SIZE + MASSAGE_TYPE_SIZE + PAYLOAD_SIZE + CHECKSUM_SIZE;

#[derive(Debug, std::cmp::PartialEq)]
pub struct VersionMessage {
    pub magic_bytes: [u8; 4],
    pub version: ProtocolVersionP2P,
    pub services: SupportedServices,
    pub timestamp: DateTime<Utc>,
    pub recv_services: SupportedServices,
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
        magic_bytes: [u8; 4],
        version: ProtocolVersionP2P,
        services: SupportedServices,
        recv_services: SupportedServices,
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
            magic_bytes,
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

    pub(super) fn serializar_payload(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        
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

    pub(super) fn deserializar_payload(stream: &mut dyn Read, magic_bytes: [u8; 4]) ->  Result<VersionMessage, ErrorMessage> {

        let version = ProtocolVersionP2P::deserialize(stream)?;
        let services = SupportedServices::deserialize(stream)?;
        let timestamp = DateTime::<Utc>::deserialize(stream)?;
        let recv_services = SupportedServices::deserialize(stream)?;

        let recv_addr = Ipv6Addr::deserialize_big_endian(stream)?;
        let recv_port = u16::deserialize_big_endian(stream)?;

        let trans_services = SupportedServices::deserialize(stream)?;
        if trans_services != services {
            return Err(ErrorMessage::ErrorInDeserialization(format!("Transceiver service isn't the same as the service: {:?}", trans_services)));
        }

        let trans_addr = Ipv6Addr::deserialize_big_endian(stream)?;
        let trans_port = u16::deserialize_big_endian(stream)?;

        let nonce = u64::deserialize(stream)?;
        let user_agent_len = CompactSize::deserialize(stream)?;
        let user_agent = String::deserialize_fix_size(stream, user_agent_len.value as usize)?;
        let start_height = i32::deserialize(stream)?;
        let relay = bool::deserialize(stream)?;

        Ok(VersionMessage {
            magic_bytes,
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

    pub(super) fn calculate_checksum(payload: &Vec<u8>) -> Result<[u8; 4], ErrorMessage> {
        let hash_bytes: sha256d::Hash = sha256d::Hash::hash(payload); 
        let checksum: [u8; 4] = match hash_bytes[0..4].try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorChecksum),
        };
        Ok(checksum)
    }
}

impl Serializable for VersionMessage {
    
    //magic_bytes
    //message_type
    //payload_size: u32
    //checksum
    //Since for the checksum we need to hash the payload, we will first serialize the payload without writing it to the stream
    //version serialization
    //services serialization
    //timestamp serialization
    //recv_services serialization
    //recv_addr serialization
    //recv_port serialization
    //trans services serialization = es el mismo que services_bytes
    //trans addrs serialization
    //trans port serialization
    //nonce serialization
    //user_agent serialization
    //start_height serialization
    //relay serialization
    //We can now calculate the checksum
    //Now that we have both the checksum and the payload we can add them to the serialized message vector
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage>{
    
        let mut serialized_message = Vec::new();
        let mut payload = Vec::new();
        
        // magic bytes
        self.magic_bytes.serialize(&mut serialized_message)?;

        // command name
        VERSION_TYPE.serialize(&mut serialized_message)?;        
        
        self.serializar_payload(&mut payload)?;

        // payload size
        (payload.len() as u32).serialize(&mut serialized_message)?;       

        // checksum
        Self::calculate_checksum(&payload)?.serialize(&mut serialized_message)?;

        // payload
        payload.serialize(&mut serialized_message)?;
        
        serialized_message.serialize(stream)
    }
}

impl Deserializable for VersionMessage {


    fn deserialize(stream: &mut dyn Read) ->  Result<Self, ErrorMessage> {

        let mut buffer: Vec<u8> = vec![0; HEADER_SIZE];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorWhileReading);
        }
        let mut buffer: &[u8] = &buffer;

        let magic_bytes = <[u8; MAGIC_BYTES_SIZE] as Deserializable>::deserialize(&mut buffer)?;

        let message_type = <[u8; MASSAGE_TYPE_SIZE] as Deserializable>::deserialize(&mut buffer)?;
        if !VERSION_TYPE.eq(&message_type) {
            return Err(ErrorMessage::ErrorInDeserialization(format!("Type name not of version: {:?}", message_type)));
        }

        let payload_size = u32::deserialize(&mut buffer)?;        
        let receive_checksum = <[u8; CHECKSUM_SIZE] as Deserializable>::deserialize(&mut buffer)?;

        let mut buffer: Vec<u8> = vec![0; payload_size as usize];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorWhileReading);
        }
        let mut buffer: &[u8] = &buffer;
        let version_message = Self::deserializar_payload(&mut buffer, magic_bytes)?;

        let mut payload_bytes: Vec<u8> = Vec::new();
        version_message.serializar_payload(&mut payload_bytes)?;
        let checksum: [u8; 4] = Self::calculate_checksum(&payload_bytes)?;

        if !checksum.eq(&receive_checksum) {
            return Err(ErrorMessage::ErrorInDeserialization(format!("Checksum isn't the same: {:?} != {:?}", checksum, receive_checksum)));
        }

        Ok(version_message)        
    }

    
}


#[cfg(test)]
mod tests {
    use crate::{
        messages::{
            serializable::Serializable, 
            serializable_big_endian::SerializableBigEndian,
            deserializable::Deserializable,
            compact_size::CompactSize,
            error_message::ErrorMessage, 
        }, 
        connections::{
            p2p_protocol::ProtocolVersionP2P, 
            suppored_services::SupportedServices,
        },
    };

    use super::{
        VersionMessage,
        VERSION_TYPE,
    };

    use chrono::{
        DateTime,
        offset::Utc,
        NaiveDateTime,
    };

    use bitcoin_hashes::{
        sha256d,
        Hash,
    };

    use std::net::Ipv6Addr;
    
    #[test]
    fn test01_serializar() -> Result<(), ErrorMessage>{
        let magic_bytes = [0x55, 0x66, 0xee, 0xee];
        let version = ProtocolVersionP2P::V31402;
        let services = SupportedServices::NodeNetworkLimited;

        let naive = NaiveDateTime::from_timestamp_opt(1628, 0).unwrap();
        let timestamp: DateTime<Utc> = DateTime::<Utc>::from_utc(naive, Utc);

        let recv_services: SupportedServices = SupportedServices::NodeNetworkLimited;
        let recv_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let recv_port: u16 = 80;
        let trans_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let trans_port: u16 = 64;
        let nonce: u64 = 00001111;
        let user_agent: String = "abc".to_string();
        let user_agent_esperado: String = "abc".to_string();
        let length: usize = user_agent.len();
        let length = CompactSize::new(length as u64);
        let start_height: i32 = 3;
        let relay: bool = false;

        let version_message = VersionMessage {
            magic_bytes,
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
        let mut stream: Vec<u8> = Vec::new();
        
        version_message.serialize(&mut stream)?;
        
        let mut stream_esperado: Vec<u8> = Vec::new();
        magic_bytes.serialize(&mut stream_esperado)?;
        VERSION_TYPE.serialize(&mut stream_esperado)?;
        
        let mut payload: Vec<u8> = Vec::new();
        version.serialize(&mut payload)?;
        services.serialize(&mut payload)?;
        timestamp.serialize(&mut payload)?;
        recv_services.serialize(&mut payload)?;
        
        recv_addr.serialize_big_endian(&mut payload)?;
        recv_port.serialize_big_endian(&mut payload)?;

        services.serialize(&mut payload)?;

        trans_addr.serialize_big_endian(&mut payload)?;
        trans_port.serialize_big_endian(&mut payload)?; 
        
        nonce.serialize(&mut payload)?;
        length.serialize(&mut payload)?; 
        user_agent_esperado.serialize(&mut payload)?;
        start_height.serialize(&mut payload)?; 
        relay.serialize(&mut payload)?;

        (payload.len() as u32).serialize(&mut stream_esperado)?;
        let hash_bytes: sha256d::Hash = sha256d::Hash::hash(&payload); 
        let checksum: [u8; 4] = match hash_bytes[0..4].try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorChecksum),
        };
        checksum.serialize(&mut stream_esperado)?;
        payload.serialize(&mut stream_esperado)?;
        
        assert_eq!(stream_esperado, stream);
        
        Ok(())
    }

    #[test]
    fn test02_deserializar() -> Result<(), ErrorMessage> {
        let magic_bytes = [0x55, 0x66, 0xee, 0xee];
        let version = ProtocolVersionP2P::V31402;
        let services = SupportedServices::NodeNetworkLimited;

        let naive = NaiveDateTime::from_timestamp_opt(1628, 0).unwrap();
        let timestamp: DateTime<Utc> = DateTime::<Utc>::from_utc(naive, Utc);

        let recv_services: SupportedServices = SupportedServices::NodeNetworkLimited;
        let recv_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let recv_port: u16 = 80;
        let trans_addr: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);
        let trans_port: u16 = 64;
        let nonce: u64 = 00001111;
        let user_agent: String = "abc".to_string();
        let user_agent_esperado: String = "abc".to_string();
        let length: usize = user_agent.len();
        let length = CompactSize::new(length as u64);
        let start_height: i32 = 3;
        let relay: bool = false;

        let mut stream: Vec<u8> = Vec::new();
    
        magic_bytes.serialize(&mut stream)?;
        VERSION_TYPE.serialize(&mut stream)?;
        
        let mut payload: Vec<u8> = Vec::new();
        version.serialize(&mut payload)?;
        services.serialize(&mut payload)?;
        timestamp.serialize(&mut payload)?;
        recv_services.serialize(&mut payload)?;
        
        recv_addr.serialize_big_endian(&mut payload)?;
        recv_port.serialize_big_endian(&mut payload)?;

        services.serialize(&mut payload)?;

        trans_addr.serialize_big_endian(&mut payload)?;
        trans_port.serialize_big_endian(&mut payload)?; 
        
        nonce.serialize(&mut payload)?;
        length.serialize(&mut payload)?; 
        user_agent_esperado.serialize(&mut payload)?;
        start_height.serialize(&mut payload)?; 
        relay.serialize(&mut payload)?;

        (payload.len() as u32).serialize(&mut stream)?;
        let hash_bytes: sha256d::Hash = sha256d::Hash::hash(&payload); 
        let checksum: [u8; 4] = match hash_bytes[0..4].try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorChecksum),
        };
        checksum.serialize(&mut stream)?;
        payload.serialize(&mut stream)?;
        
        let mut stream: &[u8] = &stream;
        
        let version_esperado = VersionMessage {
            magic_bytes,
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
