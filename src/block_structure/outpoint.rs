use super::hash::HashType;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};
use std::{
    cmp::PartialEq,
    hash::{Hash, Hasher},
    io::{Read, Write},
};

/// It represents the outpoint of a transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outpoint {
    hash: HashType,
    index: u32,
}

impl Outpoint {
    pub fn new(hash: HashType, index: u32) -> Self {
        Outpoint { hash, index }
    }
}

impl SerializableInternalOrder for Outpoint {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.hash.io_serialize(stream)?;
        self.index.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for Outpoint {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let hash = HashType::io_deserialize(stream)?;
        let index = u32::le_deserialize(stream)?;

        Ok(Outpoint { hash, index })
    }
}

impl Hash for Outpoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
        self.index.hash(state);
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_01_correct_outpoint_serialization() {
        let outpoint = Outpoint {
            hash: [
                0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80, 
                0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03, 
                0x98, 0xa1, 0x4f, 0x3f
                ],
            index: 1,
        };
        let mut stream: Vec<u8> = Vec::new();
        outpoint.io_serialize(&mut stream).unwrap();
        assert_eq!(
            stream,
            vec![
                0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80, 
                0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03, 
                0x98, 0xa1, 0x4f, 0x3f, 0x01, 0x00, 0x00, 0x00 
            ]
        );
    }

    #[test]
    fn test_02_correct_outpoint_deserialization(){
        let outpoint = Outpoint {
            hash: [
                0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80, 
                0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03, 
                0x98, 0xa1, 0x4f, 0x3f
                ],
            index: 1,
        };
        let mut stream: &[u8] = &[
            0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80, 
            0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03, 
            0x98, 0xa1, 0x4f, 0x3f, 0x01, 0x00, 0x00, 0x00 
        ];
        let outpoint_deserialized = Outpoint::io_deserialize(&mut stream).unwrap();
        assert_eq!(outpoint, outpoint_deserialized);
    }
}