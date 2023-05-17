use super::{
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    deserializable::Deserializable, error_serialization::ErrorSerialization,
    serializable::Serializable,
};

use std::io::{
    Read, 
    Write
};

#[derive(Debug, std::cmp::PartialEq)]
pub struct SendCmpctMessage {
    pub announce: bool,
    pub version: u64,
}

impl Message for SendCmpctMessage {

    fn get_command_name() -> CommandName {
        CommandName::SendCmpct
    }
}

impl Serializable for SendCmpctMessage {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.announce.serialize(stream)?;
        self.version.serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for SendCmpctMessage {
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(SendCmpctMessage{
            announce: bool::deserialize(stream)?,
            version: u64::deserialize(stream)?,
        })
    }
}