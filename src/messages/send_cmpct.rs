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

impl SerializableLittleEndian for SendCmpctMessage {

    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.announce.le_serialize(stream)?;
        self.version.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableLittleEndian for SendCmpctMessage {
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(SendCmpctMessage{
            announce: bool::le_deserialize(stream)?,
            version: u64::le_deserialize(stream)?,
        })
    }
}