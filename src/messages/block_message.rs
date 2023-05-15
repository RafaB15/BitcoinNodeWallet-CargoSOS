use super::{
    message_header::MessageHeader,
};

use crate::block_structure::{
    block::Block,
};

use crate::serialization::{
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use std::io::Read;

pub struct BlockMessage {
    pub block: Block,
}

impl BlockMessage {

    pub fn deserialize_message(
        stream: &mut dyn Read, 
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> 
    {
        let mut buffer: Vec<u8> = vec![0; message_header.payload_size as usize];

        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorWhileReading);
        }

        let mut buffer: &[u8] = &buffer[..];

        Ok(BlockMessage::deserialize(&mut buffer)?)
    }

}

impl Deserializable for BlockMessage {
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(BlockMessage { 
            block: Block::deserialize(stream)?,
        })
    }
}