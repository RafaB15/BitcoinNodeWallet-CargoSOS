use super::{
    message::Message,
    command_name::CommandName,
    message_header::MessageHeader,
};

use crate::block_structure::{
    block::Block,
};

use crate::serialization::{
    serializable_internal_order::SerializableInternalOrder,
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
        self.block.io_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for BlockMessage {
    
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(BlockMessage { 
            block: Block::io_deserialize(stream)?,
        })
    }
}