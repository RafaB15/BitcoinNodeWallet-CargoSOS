use super::hash::HashType;

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, 
};
use std::io::{
    Read,
    Write,
};

use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq)]
pub struct Outpoint {
    pub hash: HashType,
    pub index: u32,
}

impl SerializableLittleEndian for Outpoint {
    
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.hash.le_serialize(stream)?;
        self.index.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableLittleEndian for Outpoint {
    
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let hash = HashType::le_deserialize(stream)?;
        let index = u32::le_deserialize(stream)?;

        Ok(Outpoint { 
            hash, 
            index
        })
    }
}
