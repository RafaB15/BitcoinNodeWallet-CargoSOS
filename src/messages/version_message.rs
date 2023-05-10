use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
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

        let version: i32 = match self.version.try_into() {
            Ok(version) => version,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };

        let services: u64 = match self.services.try_into() {
            Ok(services) => services,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };
        
        let recv_services: u64 = match self.recv_services.try_into() {
            Ok(recv_services) => recv_services,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };

        let relay_value: u8 = match self.relay {
            true => 0x01,
            false => 0x00,
        };

        serialized_message.extend_from_slice(&self.magic_bytes);
        serialized_message.extend_from_slice(VERSION_TYPE);


        payload.extend_from_slice(&version.to_le_bytes());
        payload.extend_from_slice(&services.to_le_bytes());
        payload.extend_from_slice(&self.timestamp.timestamp().to_le_bytes());        
        payload.extend_from_slice(&recv_services.to_le_bytes());
        payload.extend_from_slice(&self.recv_addr.octets());
        payload.extend_from_slice(&self.recv_port.to_be_bytes());
        payload.extend_from_slice(&services.to_le_bytes());
        payload.extend_from_slice(&self.trans_addr.octets());
        payload.extend_from_slice(&self.trans_port.to_be_bytes());
        payload.extend_from_slice(&self.nonce.to_le_bytes());
        payload.extend_from_slice(&(self.user_agent.len() as u32).to_le_bytes());
        payload.extend_from_slice(self.user_agent.as_bytes());
        payload.extend_from_slice(&self.start_height.to_le_bytes());        
        payload.extend_from_slice(&[relay_value]);
        
        let payload_size: u32 = payload.len() as u32;
        serialized_message.extend_from_slice(&payload_size.to_le_bytes());


        let hash_of_bytes = sha256d::Hash::hash(&payload);

        let hash_bytes: &[u8] = hash_of_bytes.as_ref();
        let checksum: &[u8; 4] = match (&hash_bytes[0..4]).try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorInSerialization),
        };

        
        serialized_message.extend_from_slice(checksum);
        serialized_message.extend_from_slice(&payload);
        
        match stream.write(&serialized_message) {
            Ok(_) => Ok(())
            _ => Err(ErrorMessage::ErrorWhileWriting),
        }
    }
}

impl Deserializable for VersionMessage {
    type Value = Self;
    fn deserialize(stream: &mut dyn Read) ->  Result<Self::Value, ErrorMessage> {
        //version
        let mut version_bytes = [0u8; 4];
        if stream.read_exact(&mut version_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let version_int = i32::from_le_bytes(version_bytes);
        let version: ProtocolVersionP2P = match version_int.try_into() {
            Ok(version) => version,
            _ => return Err(ErrorMessage::ErrorInDeserialization),
        };
        
        //services
        let mut services_bytes = [0u8; 8];
        if stream.read_exact(&mut services_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let services_int: u64 = u64::from_le_bytes(services_bytes);
        let services: SupportedServices = match services_int.try_into() {
            Ok(services) => services,
            _ => return Err(ErrorMessage::ErrorInDeserialization),
        };
        
        //timestamp
        let mut timestamp_bytes = [0u8; 8];
        if stream.read_exact(&mut timestamp_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let timestamp_int = i64::from_le_bytes(timestamp_bytes);
        let timestamp_utc = NaiveDateTime::from_timestamp_opt(timestamp_int, 0).ok_or(ErrorMessage::ErrorInDeserialization)?;
        let timestamp = DateTime::<Utc>::from_utc(timestamp_utc, Utc);

        //recv_services: SupportedServices
        let mut recv_services_bytes = [0u8; 8];
        if stream.read_exact(&mut recv_services_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        };
        let recv_services_int: u64 = u64::from_le_bytes(recv_services_bytes);
        let recv_services: SupportedServices = match recv_services_int.try_into() {
            Ok(recv_services) => recv_services,
            _ => return Err(ErrorMessage::ErrorInDeserialization),
        };

        //recv_addr: Ipv6Addr
        let mut recv_bytes = [0u8; 16];
        if stream.read_exact(&mut recv_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let recv_addr = Ipv6Addr::from(recv_bytes);

        //recv_port: u16
        let mut recv_port_bytes = [0u8; 2];
        if stream.read_exact(&mut recv_port_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let recv_port = u16::from_le_bytes(recv_port_bytes);

        //addr trans services
        let mut addr_services_bytes = [0u8; 8];
        if stream.read_exact(&mut addr_services_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let addr_services_int: u64 = u64::from_le_bytes(addr_services_bytes);
        let _: SupportedServices = match addr_services_int.try_into() {
            Ok(addr_services) => match addr_services == services {
                true => addr_services,
                false => return Err(ErrorMessage::ErrorInDeserialization),
            }
            _ => return Err(ErrorMessage::ErrorInDeserialization),
        };

        //trans_addr: Ipv6Addr
        let mut trans_addr_bytes = [0u8; 16];
        if stream.read_exact(&mut trans_addr_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let trans_addr = Ipv6Addr::from(trans_addr_bytes);

        //trans_port: u16
        let mut trans_port_bytes = [0u8; 2];
        if stream.read_exact(&mut trans_port_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let trans_port = u16::from_be_bytes(trans_port_bytes);

        //nonce: u64
        let mut nonce_bytes = [0u8; 8];
        if stream.read_exact(&mut nonce_bytes).is_err() {
        return Err(ErrorMessage::ErrorInDeserialization);
        }
        let nonce = u64::from_le_bytes(nonce_bytes);

        //user_agent: String
        let mut user_agent_len_buf = [0u8; 1];
        if stream.read_exact(&mut user_agent_len_buf).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        };
        let user_agent_len = user_agent_len_buf[0] as usize;
        let mut user_agent_buf = vec![0u8; user_agent_len];
        if stream.read_exact(&mut user_agent_buf).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        };
        let user_agent = match String::from_utf8(user_agent_buf) {
            Ok(user_agent) => user_agent,
            Err(_) => return Err(ErrorMessage::ErrorInDeserialization),
        };
        
        //start_height: i32
        let mut height_bytes = [0u8; 4];
        if stream.read_exact(&mut height_bytes).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let start_height = i32::from_le_bytes(height_bytes);

        //relay: bool
        let mut relay_value = [0u8; 1];
        if stream.read_exact(&mut relay_value).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let relay = match relay_value[0] {
            0x00 => false,
            0x01 => true,
            _ => return Err(ErrorMessage::ErrorInDeserialization),
        };

        Ok(VersionMessage::new(
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

        ))
    }
}

        