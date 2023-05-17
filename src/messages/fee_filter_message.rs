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

impl Serializable for FeeFilterMessage {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        self.feerate.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for FeeFilterMessage {
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {    

        Ok(FeeFilterMessage{
            feerate: u64::deserialize(stream)?,
        })
    }
}