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
