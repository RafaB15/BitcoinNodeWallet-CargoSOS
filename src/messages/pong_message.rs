use super::{
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};

pub struct PongMessage {
    pub nonce: u64,
}

impl Message for PongMessage {

    fn get_command_name() -> CommandName {
        CommandName::Pong
    }
}

impl SerializableLittleEndian for PongMessage {
    
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.nonce.le_serialize(stream)
    }
}

impl DeserializableLittleEndian for PongMessage {

    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(PongMessage {
            nonce: u64::le_deserialize(stream)?,
        })
    }
}