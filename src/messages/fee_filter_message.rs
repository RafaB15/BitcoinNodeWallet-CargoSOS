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
    Write
};

#[derive(Debug)]
pub struct FeeFilterMessage {
    pub feerate: u64,
}

impl Message for FeeFilterMessage {

    fn get_command_name() -> CommandName {
        CommandName::FeeFilter
    }
}

impl SerializableLittleEndian for FeeFilterMessage {

    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        self.feerate.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableLittleEndian for FeeFilterMessage {
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {    

        Ok(FeeFilterMessage{
            feerate: u64::le_deserialize(stream)?,
        })
    }
}