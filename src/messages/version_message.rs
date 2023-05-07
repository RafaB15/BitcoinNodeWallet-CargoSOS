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
    
        //version
        let version: i32 = match self.version.try_into() {
            Ok(version) => version,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };

        if stream.write(&version.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        //serializar 2 veces services
        let services: i64 = match self.services.try_into() {
            Ok(services) => services,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };

        if stream.write(&services.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        //timestamp
        let timestamp_bytes = self.timestamp.timestamp().to_le_bytes();
        if stream.write(&timestamp_bytes).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        //recv_services
        let recv_services: i64 = match self.recv_services.try_into() {
            Ok(recv_services) => recv_services,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };

        if stream.write(&recv_services.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }
        
        //recv_addr
        let recv_bytes = self.recv_addr.octets();
        if stream.write(&recv_bytes).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        //recv_port
        if stream.write(&self.recv_port.to_be_bytes()).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }
        
        //addr trans services
        if stream.write(&services.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        //trans addr
        let trans_addr_bytes = self.trans_addr.octets();
        if stream.write(&trans_addr_bytes).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        //trans port
        if stream.write(&self.trans_port.to_be_bytes()).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        //nonce
        if stream.write(&self.nonce.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        //user_agent VER ESTO
        if stream.write(&self.user_agent.as_bytes()).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        //start_height
        if stream.write(&self.start_height.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        //relay
        let relay_value = match self.relay {
            true => 0x01,
            false => 0x00,
        };
        if stream.write(&[relay_value]).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
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
        let services_int: i64 = i64::from_le_bytes(services_bytes);
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
        let recv_services_int: SupportedServices = recv_services_bytes.try_into();

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
        let addr_services_int: i64 = i64::from_le_bytes(services_bytes);
        let addr_services: SupportedServices = match addr_services_int.try_into() {
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

        //trans_port: u16, // tal vez es el mismo que el recv_port
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

        //user_agent: String -> no user agent (vacio)

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