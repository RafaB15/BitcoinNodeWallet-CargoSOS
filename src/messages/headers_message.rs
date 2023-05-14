use super::compact_size::CompactSize;
use crate::block_structure::block_header::BlockHeader;
use crate::serialization::deserializable::Deserializable;

use std::io::Read;
use crate::serialization::error_serialization::ErrorSerialization;

pub struct HeadersMessage {
    pub count: CompactSize,
    pub headers: Vec<BlockHeader>,
}

impl HeadersMessage {
    pub fn new(count: CompactSize, headers: Vec<BlockHeader>) -> Self {
        HeadersMessage { 
            count, 
            headers, 
        }
    }
}

impl Deserializable for HeadersMessage {
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let count = CompactSize::deserialize(stream)?;
        let mut headers = Vec::new();
        for _ in 0..count.value {
            headers.push(BlockHeader::deserialize(stream)?);
        }
        Ok(HeadersMessage::new(count, headers))
    }
}



