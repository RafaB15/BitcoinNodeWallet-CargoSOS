use super::{command_name::CommandName, compact_size::CompactSize, message::Message};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::block_structure::hash::HashType;

use crate::connections::p2p_protocol::ProtocolVersionP2P;

use std::io::{Read, Write};

/// It's the get headers message
#[derive(Debug, std::cmp::PartialEq)]
pub struct GetHeadersMessage {
    pub version: ProtocolVersionP2P,
    pub header_locator_hashes: Vec<HashType>,
    pub stop_hash: HashType,
}

impl GetHeadersMessage {
    pub fn new(
        version: ProtocolVersionP2P,
        header_locator_hashes: Vec<HashType>,
        stop_hash: HashType,
    ) -> Self {
        GetHeadersMessage {
            version,
            header_locator_hashes,
            stop_hash,
        }
    }
}

impl Message for GetHeadersMessage {
    fn get_command_name() -> CommandName {
        CommandName::GetHeaders
    }
}

impl SerializableInternalOrder for GetHeadersMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.le_serialize(stream)?;
        CompactSize::new(self.header_locator_hashes.len() as u64).le_serialize(stream)?;

        for hash in self.header_locator_hashes.iter() {
            hash.le_serialize(stream)?;
        }

        self.stop_hash.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for GetHeadersMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version = ProtocolVersionP2P::le_deserialize(stream)?;
        let size = CompactSize::le_deserialize(stream)?;

        let mut header_locator_hashes: Vec<HashType> = Vec::new();
        for _ in 0..size.value {
            let header_locator_hash = HashType::le_deserialize(stream)?;
            header_locator_hashes.push(header_locator_hash);
        }

        let stop_hash = HashType::le_deserialize(stream)?;

        Ok(GetHeadersMessage {
            version,
            header_locator_hashes,
            stop_hash,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test01_serialize() -> Result<(), ErrorSerialization> {
        let version = ProtocolVersionP2P::V70015;
        let header_locator_hash: Vec<HashType> = vec![[1; 32], [2; 32], [0; 32]];
        let length = CompactSize::new(header_locator_hash.len() as u64);
        let stop_hash: HashType = [1; 32];

        let mut expected_stream: Vec<u8> = Vec::new();

        version.le_serialize(&mut expected_stream)?;
        length.le_serialize(&mut expected_stream)?;
        for header_hash in header_locator_hash.iter() {
            let _ = header_hash.le_serialize(&mut expected_stream)?;
        }
        stop_hash.le_serialize(&mut expected_stream)?;

        let get_headers_message = GetHeadersMessage::new(version, header_locator_hash, stop_hash);

        let mut stream: Vec<u8> = Vec::new();
        get_headers_message.io_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize() {
        let version = ProtocolVersionP2P::V70015;
        let header_locator_hash: Vec<HashType> = vec![[1; 32], [2; 32], [0; 32]];
        let stop_hash: HashType = [1; 32];

        let mut serialized_stream: Vec<u8> = Vec::new();
        let get_headers_message = GetHeadersMessage {
            version,
            header_locator_hashes: header_locator_hash.clone(),
            stop_hash,
        };

        get_headers_message
            .io_serialize(&mut serialized_stream)
            .unwrap();
        let deserialized_message =
            GetHeadersMessage::io_deserialize(&mut serialized_stream.as_slice()).unwrap();

        assert_eq!(deserialized_message, get_headers_message);
    }
}
