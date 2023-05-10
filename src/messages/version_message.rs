use super::{
    serializable_big_endian::SerializableBigEndian,
    serializable::Serializable,
    deserializable::{
        Deserializable,
        get_slice,
    },
    error_message::ErrorMessage, deserializable_big_endian::DeserializableBigEndian, compact_size::CompactSize, deserializable_fix_size::DeserializableFixSize, 
};

use std::net::{Ipv6Addr, SocketAddr};
use chrono::{
    DateTime,
    //Timelike,
    NaiveDateTime,
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

    pub fn serializar_payload(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        
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

    pub fn deserializar_payload(stream: &mut dyn Read, magic_bytes: [u8; 4], payload_size: u32) ->  Result<VersionMessage, ErrorMessage> {
        let mut buffer: Vec<u8> = vec![0; payload_size as usize];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let mut buffer: &[u8] = &buffer[..];

        let version = ProtocolVersionP2P::deserialize(&mut buffer)?;
        let services = SupportedServices::deserialize(&mut buffer)?;
        let timestamp = DateTime::<Utc>::deserialize(&mut buffer)?;
        let recv_services = SupportedServices::deserialize(&mut buffer)?;

        let recv_addr = Ipv6Addr::deserialize_big_endian(&mut buffer)?;
        let recv_port = u16::deserialize_big_endian(&mut buffer)?;

        let trans_services = SupportedServices::deserialize(&mut buffer)?;
        if trans_services != services {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        let trans_addr = Ipv6Addr::deserialize_big_endian(&mut buffer)?;
        let trans_port = u16::deserialize_big_endian(&mut buffer)?;

        let nonce = u64::deserialize(&mut buffer)?;
        let user_agent_len = CompactSize::deserialize(&mut buffer)?;
        let user_agent = String::deserialize_fix_size(&mut buffer, user_agent_len.value as usize)?;
        let start_height = i32::deserialize(&mut buffer)?;
        let relay = bool::deserialize(&mut buffer)?;

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
        let hash_bytes: sha256d::Hash = sha256d::Hash::hash(&payload);        
        let checksum: &[u8; 4] = match (&hash_bytes[0..4]).try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorInSerialization),
        };
        checksum.serialize(&mut serialized_message)?;

        // payload
        payload.serialize(&mut serialized_message)?;
        
        serialized_message.serialize(stream)
    }
}

impl Deserializable for VersionMessage {
    
    type Value = Self;

    fn deserialize(stream: &mut dyn Read) ->  Result<Self::Value, ErrorMessage> {

        let mut buffer: Vec<u8> = vec![0; HEADER_SIZE];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let mut buffer: &[u8] = &buffer[..];

        let magic_bytes = <[u8; MAGIC_BYTES_SIZE] as Deserializable>::deserialize(&mut buffer)?;

        let message_type = <[u8; MASSAGE_TYPE_SIZE] as Deserializable>::deserialize(&mut buffer)?;
        if !VERSION_TYPE.eq(&message_type) {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        let payload_size = <u32 as Deserializable>::deserialize(&mut buffer)?;        
        let receive_checksum = <[u8; CHECKSUM_SIZE] as Deserializable>::deserialize(&mut buffer)?;

        let version_message = Self::deserializar_payload(stream, magic_bytes, payload_size)?;

        let mut payload_bytes: Vec<u8> = Vec::new();
        version_message.serializar_payload(&mut payload_bytes)?;

        let hash_bytes: sha256d::Hash = sha256d::Hash::hash(&payload_bytes);        
        let checksum: &[u8; 4] = match (&hash_bytes[0..4]).try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorInSerialization),
        };

        if !checksum.eq(&receive_checksum) {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        Ok(version_message)        
    }

    
}