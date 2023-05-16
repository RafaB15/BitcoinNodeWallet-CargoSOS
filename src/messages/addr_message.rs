use super::{
    compact_size::CompactSize,
    message_header::MessageHeader,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable, 
    error_serialization::ErrorSerialization,
};

use crate::connections::{
    network_ip_addres::NetworkIpAddres,
};

use crate::block_structure::{
    hash::hash256d_reduce,
};

use std::io::{
    Read, 
    Write
};

#[derive(Debug)]
pub struct AddrMessage {
    pub ip_addresses: Vec<NetworkIpAddres>,
}

impl AddrMessage {
  
    pub fn deserialize_message(
        stream: &mut dyn Read, 
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> 
    {
        let mut buffer: Vec<u8> = vec![0; message_header.payload_size as usize];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorWhileReading);
        }
        let mut buffer: &[u8] = &buffer[..];

        let message = Self::deserialize(&mut buffer)?;

        let mut serialized_message: Vec<u8> = Vec::new();
        message.serialize(&mut serialized_message)?;

        let checksum = hash256d_reduce(&serialized_message)?;
        if !checksum.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(
                format!("Checksum in addr isn't the same: {:?} != {:?}", checksum, message_header.checksum)
            ));
        }

        Ok(message)
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