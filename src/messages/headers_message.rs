use super::{
    compact_size::CompactSize,
    message::Message,
    command_name::CommandName,
};

use crate::block_structure::{
    block_header::BlockHeader, 
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

pub struct HeadersMessage {
    pub headers: Vec<BlockHeader>,
}

impl Message for HeadersMessage {

    fn get_command_name() -> CommandName {
        CommandName::Headers
    }
}

impl SerializableInternalOrder for HeadersMessage {
        
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        CompactSize::new(self.headers.len() as u64).le_serialize(stream)?;
        for header in &self.headers {
            header.le_serialize(stream)?;
        }
        Ok(())
    }
}

impl DeserializableInternalOrder for HeadersMessage {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let count = CompactSize::le_deserialize(stream)?.value;
        let mut headers = Vec::new();
        for _ in 0..count {
            headers.push(BlockHeader::le_deserialize(stream)?);
        }
        Ok(HeadersMessage{
            headers,
        })
    }
}



