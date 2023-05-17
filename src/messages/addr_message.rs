use super::{
    compact_size::CompactSize,
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable, 
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

impl Serializable for AddrMessage {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        CompactSize::new(self.ip_addresses.len() as u64).serialize(stream)?;
        for ip_address in self.ip_addresses.iter() {
            ip_address.serialize(stream)?;
        }

        Ok(())
    }
}

impl Deserializable for AddrMessage {
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {    

        let ip_address_count = CompactSize::deserialize(stream)?.value;
        let mut ip_addresses: Vec<NetworkIpAddres> = Vec::new();

        for _ in 0..ip_address_count {
            ip_addresses.push(NetworkIpAddres::deserialize(stream)?);
        }

        Ok(AddrMessage{
            ip_addresses,
        })
    }
}