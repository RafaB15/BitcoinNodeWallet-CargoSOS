use super::{
    message::Message,
    command_name::CommandName,
};

use crate::block_structure::{
    block::Block,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
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

impl SerializableInternalOrder for BlockMessage {
    
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.block.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for BlockMessage {
    
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(BlockMessage { 
            block: Block::le_deserialize(stream)?,
        })
    }
}