use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use crate::block_structure::{
    hash::HashTypeReduced,
};

use super::command_name::CommandName;

use std::io::{
    Read,
    Write,
};

pub type MagicType = [u8; 4];

pub struct MessageHeader {
    pub magic_numbers: MagicType,
    pub command_name: CommandName,
    pub payload_size: u32,
    pub checksum: HashTypeReduced,
}

impl Serializable for MessageHeader {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        self.magic_numbers.serialize(stream)?;
        self.command_name.serialize(stream)?;
        self.payload_size.serialize(stream)?;
        self.checksum.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for MessageHeader {
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        
        Ok(MessageHeader {
            magic_numbers: MagicType::deserialize(stream)?,
            command_name: CommandName::deserialize(stream)?,
            payload_size: u32::deserialize(stream)?,
            checksum: HashTypeReduced::deserialize(stream)?,
        })
    }
}