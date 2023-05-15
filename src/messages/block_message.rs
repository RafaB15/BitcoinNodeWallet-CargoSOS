use super::{
    message,
    message_header::MessageHeader,
};

use crate::block_structure::{
    block::Block,
    hash::hash256d_reduce,
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

impl BlockMessage {

    pub fn deserialize_message(
        stream: &mut dyn Read, 
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> 
    {
        let mut buffer: &[u8] = message::read_exact(stream, message_header.payload_size as usize)?;

        let message = Self::deserialize(&mut buffer)?;

        let mut serialized_message: Vec<u8> = Vec::new();
        message.serialize(&mut serialized_message)?;
        
        let checksum = hash256d_reduce(&serialized_message)?;
        if !checksum.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(
                format!("Checksum isn't the same: {:?} != {:?}", checksum, message_header.checksum)
            ));
        }

        Ok(message)        
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