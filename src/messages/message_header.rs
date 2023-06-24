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

/// It;s the header of any message
#[derive(Debug)]
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
