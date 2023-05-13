use crate::connections::error_connection::ErrorConnection;
use std::io::Write;
use crate::serialization::{
    serializable::Serializable,
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

impl Serializable for BlockVersion {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let version: i32 = match (*self).try_into() {
            Ok(version) => version,
            Err(_) => return Err(ErrorSerialization::ErrorInSerialization(format!("While serializing {:?}", self))),
        };
        match stream.write(&version.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorWhileWriting),
        }
    }
}