use super::{command_name::CommandName, compact_size::CompactSize, message::Message};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::connections::network_ip_addres::NetworkIpAddres;

use std::io::{Read, Write};

/// It's the address message
#[derive(Debug)]
pub struct AddrMessage {
    pub ip_addresses: Vec<NetworkIpAddres>,
}

impl Message for AddrMessage {
    fn get_command_name() -> CommandName {
        CommandName::Addr
    }
}

impl SerializableInternalOrder for AddrMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        CompactSize::new(self.ip_addresses.len() as u64).le_serialize(stream)?;
        for ip_address in self.ip_addresses.iter() {
            ip_address.le_serialize(stream)?;
        }

        Ok(())
    }
}

impl DeserializableInternalOrder for AddrMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let ip_address_count = CompactSize::le_deserialize(stream)?.value;
        let mut ip_addresses: Vec<NetworkIpAddres> = Vec::new();

        for _ in 0..ip_address_count {
            ip_addresses.push(NetworkIpAddres::le_deserialize(stream)?);
        }

        Ok(AddrMessage { ip_addresses })
    }
}
