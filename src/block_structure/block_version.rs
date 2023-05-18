use crate::connections::error_connection::ErrorConnection;
use std::io::Write;
use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
};


#[derive(Debug, std::cmp::PartialEq, Copy, Clone)]
pub enum BlockVersion {
    V1,
    V2,
    V3,
    V4,
}

impl std::convert::TryFrom<i32> for BlockVersion {
    type Error = ErrorConnection;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(BlockVersion::V1),
            2 => Ok(BlockVersion::V2),
            3 => Ok(BlockVersion::V3),
            4 => Ok(BlockVersion::V4),
            _ => Err(ErrorConnection::ErrorInvalidInputParse),
        }
    }
}

impl std::convert::TryInto<i32> for BlockVersion {
    type Error = ErrorConnection;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            BlockVersion::V1 => Ok(1),
            BlockVersion::V2 => Ok(2),
            BlockVersion::V3 => Ok(3),
            BlockVersion::V4 => Ok(4),
        }
    }
}

impl SerializableLittleEndian for BlockVersion {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let version: i32 = match (*self).try_into() {
            Ok(version) => version,
            Err(_) => {
                return Err(ErrorSerialization::ErrorInSerialization(format!(
                    "While serializing {:?}",
                    self
                )))
            }
        };
        match stream.write(&version.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorWhileWriting),
        }
    }
}

impl DeserializableLittleEndian for BlockVersion {

    fn le_deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let version_int = i32::le_deserialize(stream)?;
        match version_int.try_into() {
            Ok(version) => Ok(version),
            Err(_) => Err(ErrorSerialization::ErrorInDeserialization(format!("While deserializing block version {:?}", version_int))),
        }
    }
}
