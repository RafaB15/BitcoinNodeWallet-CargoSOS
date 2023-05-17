use super::{
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    deserializable_little_endian::DeserializableLittleEndian, error_serialization::ErrorSerialization,
    serializable_little_endian::SerializableLittleEndian,
};

use std::io::{
    Read, 
    Write
};

pub const SEND_HEADERS_CHECKSUM: [u8; 4] = [0x5d, 0xf6, 0xe0, 0xe2];

#[derive(Debug, std::cmp::PartialEq)]
pub struct SendHeadersMessage;

impl Message for SendHeadersMessage {

    fn calculate_checksum(
        _: &[u8],
    ) -> Result<[u8; 4], ErrorSerialization> 
    {
        Ok(SEND_HEADERS_CHECKSUM)
    }

    fn get_command_name() -> CommandName {
        CommandName::SendHeaders
    }
}

impl SerializableLittleEndian for SendHeadersMessage {

    fn le_serialize(&self, _: &mut dyn Write) -> Result<(), ErrorSerialization> {
        Ok(())
    }
}

impl DeserializableLittleEndian for SendHeadersMessage {
    fn le_deserialize(_: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(SendHeadersMessage)
    }
}