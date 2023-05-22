use super::{command_name::CommandName, message::Message};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use std::io::{Read, Write};

pub struct PongMessage {
    pub nonce: u64,
}

impl Message for PongMessage {
    fn get_command_name() -> CommandName {
        CommandName::Pong
    }
}

impl SerializableInternalOrder for PongMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.nonce.le_serialize(stream)
    }
}

impl DeserializableInternalOrder for PongMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(PongMessage {
            nonce: u64::le_deserialize(stream)?,
        })
    }
}
