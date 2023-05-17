use super::{
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
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

impl Serializable for PongMessage {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.nonce.serialize(stream)
    }
}

impl Deserializable for PongMessage {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(PongMessage {
            nonce: u64::deserialize(stream)?,
        })
    }
}