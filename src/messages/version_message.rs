use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::net::Ipv6Addr;
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
};

use bitcoin_hashes::{
    sha256d,
    Hash,
};

pub const VERSION_TYPE: [u8; 12] = [118, 101, 114, 115, 105, 111, 110, 0, 0, 0, 0, 0];

pub struct VersionMessage {
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

impl Serializable for VersionMessage {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage>{
    
        //message_type

        if stream.write(&VERSION_TYPE).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        //payload_size: u32
        let payload_size: u32 = 86; //Está algo hardcodeado, pues asume que mandamos un cero como user agent bytes
        if stream.write(&payload_size.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        //checksum
        //Since for the checksum we need to hash the payload, we will first serialize the payload without writing it to the stream

        //version serialization
        let version: i32 = match self.version.try_into() {
            Ok(version) => version,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };
        let version_bytes = version.to_le_bytes();

        //services serialization
        let services: u64 = match self.services.try_into() {
            Ok(services) => services,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };
        let services_bytes = services.to_le_bytes();

        //timestamp serialization
        let timestamp_bytes = self.timestamp.timestamp().to_le_bytes();

        //recv_services serialization
        let recv_services: u64 = match self.recv_services.try_into() {
            Ok(recv_services) => recv_services,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };
        let recv_services_bytes = recv_services.to_le_bytes();

        //recv_addr serialization
        let recv_addr_bytes = self.recv_addr.octets();

        //recv_port serialization
        let recv_port_bytes = self.recv_port.to_be_bytes();

        //trans services serialization = es el mismo que services_bytes

        //trans addrs serialization
        let trans_addr_bytes = self.trans_addr.octets();

        //trans port serialization
        let trans_port_bytes = self.trans_port.to_be_bytes();

        //nonce serialization
        let nonce_bytes = self.nonce.to_le_bytes();

        //user_agent serialization
        let user_agent_bytes = (self.user_agent.len() as u32).to_le_bytes();

        //start_height serialization
        let start_height_bytes = self.start_height.to_le_bytes();

        //relay serialization
        let relay_value: u8 = match self.relay {
            true => 0x01,
            false => 0x00,
        };
        let relay_bytes = [relay_value];
        
        //We can now create a payload variable and write the bytes to it

        let mut payload = Vec::new();
        payload.extend_from_slice(&version_bytes);
        payload.extend_from_slice(&services_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&recv_services_bytes);
        payload.extend_from_slice(&recv_addr_bytes);
        payload.extend_from_slice(&recv_port_bytes);
        payload.extend_from_slice(&services_bytes);
        payload.extend_from_slice(&trans_addr_bytes);
        payload.extend_from_slice(&trans_port_bytes);
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&user_agent_bytes);
        payload.extend_from_slice(&start_height_bytes);
        payload.extend_from_slice(&relay_bytes);

        //We can now calculate the checksum
        let hash_of_bytes = sha256d::Hash::hash(&payload);

        let hash_bytes: &[u8] = hash_of_bytes.as_ref();
        let checksum: &[u8; 4] = match (&hash_bytes[0..4]).try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorInSerialization),
        };

        //We write the checksum to the stream
        if stream.write(checksum).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        //We write the payload to the stream
        if stream.write(&payload).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        Ok(())

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

        