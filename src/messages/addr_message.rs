use super::{
    compact_size::CompactSize,
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian, 
    error_serialization::ErrorSerialization,
};

use crate::connections::{
    network_ip_addres::NetworkIpAddres,
};

use std::io::{
    Read, 
    Write
};

#[derive(Debug)]
pub struct AddrMessage {
    pub ip_addresses: Vec<NetworkIpAddres>,
}

impl Message for AddrMessage {

    fn get_command_name() -> CommandName {
        CommandName::Addr
    }
}

impl SerializableLittleEndian for AddrMessage {

    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        CompactSize::new(self.ip_addresses.len() as u64).le_serialize(stream)?;
        for ip_address in self.ip_addresses.iter() {
            ip_address.le_serialize(stream)?;
        }

        Ok(())
    }
}

impl DeserializableLittleEndian for AddrMessage {
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {    

        let ip_address_count = CompactSize::le_deserialize(stream)?.value;
        let mut ip_addresses: Vec<NetworkIpAddres> = Vec::new();

        for _ in 0..ip_address_count {
            ip_addresses.push(NetworkIpAddres::le_deserialize(stream)?);
        }

        Ok(AddrMessage{
            ip_addresses,
        })
    }
}