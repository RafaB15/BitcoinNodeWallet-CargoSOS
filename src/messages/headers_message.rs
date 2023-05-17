use super::{
    compact_size::CompactSize,
    message::Message,
    command_name::CommandName,
};

use crate::block_structure::{
    block_header::BlockHeader, 
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

pub struct HeadersMessage {
    pub headers: Vec<BlockHeader>,
}

impl Message for HeadersMessage {

    fn get_command_name() -> CommandName {
        CommandName::Headers
    }
}

impl Serializable for HeadersMessage {
        
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        CompactSize::new(self.headers.len() as u64).serialize(stream)?;
        for header in &self.headers {
            header.serialize(stream)?;
        }
        Ok(())
    }
}

impl Deserializable for HeadersMessage {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let count = CompactSize::deserialize(stream)?.value;
        let mut headers = Vec::new();
        for _ in 0..count {
            headers.push(BlockHeader::deserialize(stream)?);
        }
        Ok(HeadersMessage{
            headers,
        })
    }
}



