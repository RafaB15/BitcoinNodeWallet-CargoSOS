use super::{
    compact_size::CompactSize,
    message_header::MessageHeader,
};

use crate::block_structure::{
    block_header::BlockHeader, 
    hash::hash256d_reduce
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

impl HeadersMessage {
    pub fn new(headers: Vec<BlockHeader>) -> Self {
        HeadersMessage { 
            headers, 
        }
    }

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

        let message = Self::deserialize(&mut buffer)?;

        let mut serialized_message: Vec<u8> = Vec::new();
        message.serialize(&mut serialized_message)?;
        
        let checksum = hash256d_reduce(&serialized_message)?;
        if !checksum.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(
                format!("Checksum in headers isn't the same: {:?} != {:?}", checksum, message_header.checksum)
            ));
        }

        Ok(message)        
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
        Ok(HeadersMessage::new(
            headers)
        )
    }
}



