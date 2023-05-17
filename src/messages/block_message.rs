use super::{
    message::Message,
    command_name::CommandName,
};

use crate::block_structure::{
    block::Block,
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

pub struct BlockMessage {
    pub block: Block,
}

impl Message for BlockMessage {
    
    fn get_command_name() -> CommandName {
        CommandName::Block
    }
}

impl Serializable for BlockMessage {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.block.serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for BlockMessage {
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(BlockMessage { 
            block: Block::deserialize(stream)?,
        })
    }
}