use super::{
    message::{
        Message,
        CHECKSUM_EMPTY_PAYLOAD,
    },
    command_name::CommandName,
};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder, 
    serializable_internal_order::SerializableInternalOrder,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read, 
    Write
};

#[derive(Debug, std::cmp::PartialEq)]
pub struct SendHeadersMessage;

impl Message for SendHeadersMessage {

    fn calculate_checksum(
        _: &[u8],
    ) -> Result<[u8; 4], ErrorSerialization> 
    {
        Ok(CHECKSUM_EMPTY_PAYLOAD)
    }

    fn get_command_name() -> CommandName {
        CommandName::SendHeaders
    }
}

impl SerializableInternalOrder for SendHeadersMessage {

    fn io_serialize(&self, _: &mut dyn Write) -> Result<(), ErrorSerialization> {
        Ok(())
    }
}

impl DeserializableInternalOrder for SendHeadersMessage {
    fn io_deserialize(_: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(SendHeadersMessage)
    }
}