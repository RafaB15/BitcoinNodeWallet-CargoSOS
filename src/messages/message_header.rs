use super::command_name::CommandName;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::block_structure::hash::HashTypeReduced;

use std::io::{Read, Write};

const MAGIC_BYTES_SIZE: usize = 4;
const MASSAGE_TYPE_SIZE: usize = 12;
const PAYLOAD_SIZE: usize = 4;
const CHECKSUM_SIZE: usize = 4;

const HEADER_SIZE: usize = MAGIC_BYTES_SIZE + MASSAGE_TYPE_SIZE + PAYLOAD_SIZE + CHECKSUM_SIZE;

pub type MagicType = [u8; 4];

/// It's the header of any message
#[derive(Debug, std::cmp::PartialEq)]
pub struct MessageHeader {
    pub magic_numbers: MagicType,
    pub command_name: CommandName,
    pub payload_size: u32,
    pub checksum: HashTypeReduced,
}

impl MessageHeader {
    /// Reads the header from the stream
    ///
    /// ### Error
    ///  * `ErrorSerialization::ErrorWhileReading`: It will appear when there is an error in the reading from a stream
    pub fn deserialize_header(stream: &mut dyn Read) -> Result<MessageHeader, ErrorSerialization> {
        let mut buffer: Vec<u8> = vec![0; HEADER_SIZE];

        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorWhileReading);
        }

        let mut buffer: &[u8] = &buffer[..];

        MessageHeader::io_deserialize(&mut buffer)
    }
}

impl SerializableInternalOrder for MessageHeader {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.magic_numbers.io_serialize(stream)?;
        self.command_name.io_serialize(stream)?;
        self.payload_size.le_serialize(stream)?;
        self.checksum.io_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for MessageHeader {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(MessageHeader {
            magic_numbers: MagicType::io_deserialize(stream)?,
            command_name: CommandName::io_deserialize(stream)?,
            payload_size: u32::le_deserialize(stream)?,
            checksum: HashTypeReduced::io_deserialize(stream)?,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test01_correct_message_header_serialization() {
        let mut serialized_fields = vec![];
        let magic_numbers: [u8; 4] = [0x55, 0x66, 0xee, 0xee];
        magic_numbers.io_serialize(&mut serialized_fields).unwrap();

        let command_name: CommandName = CommandName::Headers;
        command_name.io_serialize(&mut serialized_fields).unwrap();

        let payload_size: u32 = 1;
        payload_size.le_serialize(&mut serialized_fields).unwrap();

        let checksum = [0xC7, 0xF1, 0x8F, 0xE8];
        checksum.io_serialize(&mut serialized_fields).unwrap();

        let message_header = MessageHeader {
            magic_numbers,
            command_name,
            payload_size,
            checksum,
        };

        let mut serialized_message_header = vec![];
        message_header
            .io_serialize(&mut serialized_message_header)
            .unwrap();

        assert_eq!(serialized_fields, serialized_message_header);
    }

    #[test]
    fn test02_correct_message_header_deserialization() {
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];
        let checksum = [0xC7, 0xF1, 0x8F, 0xE8];

        let header = MessageHeader {
            magic_numbers: magic_bytes,
            command_name: CommandName::Headers,
            payload_size: 0,
            checksum,
        };

        let mut serialized_headers_message = vec![];
        header
            .io_serialize(&mut serialized_headers_message)
            .unwrap();

        let expected_message_header =
            MessageHeader::io_deserialize(&mut serialized_headers_message.as_slice()).unwrap();

        assert_eq!(header, expected_message_header);
    }
}
