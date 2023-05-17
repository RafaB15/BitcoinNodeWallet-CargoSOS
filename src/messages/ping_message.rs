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

pub struct PingMessage {
    pub nonce: u64,
}

impl Message for PingMessage {
    
    fn get_command_name() -> CommandName {
        CommandName::Ping
    }
}

impl Serializable for PingMessage {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.nonce.serialize(stream)
    }
}

impl Deserializable for PingMessage {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(PingMessage {
            nonce: u64::deserialize(stream)?,
        })
    }
}