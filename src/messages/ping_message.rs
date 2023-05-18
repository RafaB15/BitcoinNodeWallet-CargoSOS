use super::{
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};

pub struct PingMessage {
    pub nonce: u64,
}

impl Message for PingMessage {
    
    fn get_command_name() -> CommandName {
        CommandName::Ping
    }
}

impl SerializableInternalOrder for PingMessage {
    
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.nonce.le_serialize(stream)
    }
}

impl DeserializableInternalOrder for PingMessage {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(PingMessage {
            nonce: u64::le_deserialize(stream)?,
        })
    }
}