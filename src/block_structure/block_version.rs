use crate::serialization::{
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_little_endian::SerializableLittleEndian,
};

use std::{
    convert::From,
    io::{Read, Write},
};

/// Represents the version of a block in the block chain
#[derive(Debug, std::cmp::PartialEq, Copy, Clone)]
pub struct BlockVersion {
    pub value: i32,
}

impl BlockVersion {
    pub const fn version(value: i32) -> Self {
        BlockVersion { value }
    }
}

impl From<i32> for BlockVersion {
    fn from(value: i32) -> Self {
        BlockVersion { value }
    }
}

impl From<BlockVersion> for i32 {
    fn from(block_version: BlockVersion) -> Self {
        block_version.value
    }
}

impl SerializableLittleEndian for BlockVersion {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let version: i32 = (*self).into();
        match stream.write(&version.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorWhileWriting),
        }
    }
}

impl DeserializableLittleEndian for BlockVersion {
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version_int = i32::le_deserialize(stream)?;
        Ok(version_int.into())
    }
}
