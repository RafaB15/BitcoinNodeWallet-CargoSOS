use super::{command_name::CommandName, compact_size::CompactSize, message::Message};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::block_structure::block_header::BlockHeader;

use std::io::{Read, Write};

/// It's the headers message
#[derive(Debug, std::cmp::PartialEq)]
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
            header.io_serialize(stream)?;
        }
        Ok(())
    }
}

impl DeserializableInternalOrder for HeadersMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let count = CompactSize::le_deserialize(stream)?.value;
        let mut headers = Vec::new();
        for _ in 0..count {
            headers.push(BlockHeader::io_deserialize(stream)?);
        }
        Ok(HeadersMessage { headers })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_headers_message_serialization() {
        let genesis_block_header = BlockHeader::generate_genesis_block_header();
        let field = vec![genesis_block_header.clone()];
        let header_message = HeadersMessage {
            headers: vec![genesis_block_header.clone()],
        };

        let mut serialized_block_header_fields = Vec::new();
        CompactSize::new(field.len() as u64)
            .le_serialize(&mut serialized_block_header_fields)
            .unwrap();
        genesis_block_header
            .io_serialize(&mut serialized_block_header_fields)
            .expect("Error while serializing");

        let mut serialized_header_message = Vec::new();
        header_message
            .io_serialize(&mut serialized_header_message)
            .unwrap();
        assert_eq!(serialized_block_header_fields, serialized_header_message);
    }

    #[test]
    fn test_02_correct_headers_message_deserialization() {
        let block_header_bytes = vec![
            0x00, 0xe0, 0xf8, 0x2c, 0x3a, 0x41, 0xed, 0xdf, 0xb7, 0x7e, 0x88, 0x5d, 0x5a, 0x15,
            0x47, 0x6e, 0xbe, 0x14, 0xe4, 0x11, 0x58, 0x81, 0xed, 0xf8, 0xfc, 0x64, 0x30, 0x3e,
            0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0xc4, 0xf8, 0x11, 0x4e, 0x74,
            0xc8, 0x6f, 0xec, 0xe0, 0xe8, 0xba, 0xfe, 0xff, 0x77, 0x3f, 0xc7, 0x3e, 0xa1, 0x8c,
            0x62, 0xad, 0x08, 0x54, 0xe5, 0xf8, 0xb0, 0xc5, 0x2f, 0x68, 0x3a, 0xb5, 0x41, 0xa9,
            0x95, 0x64, 0x7f, 0x5d, 0x21, 0x19, 0x4d, 0x58, 0xb1, 0x0f, 0x00, 0x00, 0x00, 0x00,
        ];

        let header = BlockHeader::io_deserialize(&mut block_header_bytes.as_slice());

        let header_message = HeadersMessage {
            headers: vec![header.unwrap()],
        };

        let mut serialized_header_message = Vec::new();
        header_message
            .io_serialize(&mut serialized_header_message)
            .unwrap();

        let deserialized_header_message =
            HeadersMessage::io_deserialize(&mut serialized_header_message.as_slice()).unwrap();

        assert_eq!(header_message, deserialized_header_message);
    }
}
