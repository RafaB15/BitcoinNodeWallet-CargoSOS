use crate::block_structure::{
    block::Block,
};

use crate::serialization::{
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

pub struct BlockMessage {
    pub block: Block,
}

impl Deserializable for BlockMessage {
    
    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        Ok(BlockMessage { 
            block: Block::deserialize(stream)?,
        })
    }
}