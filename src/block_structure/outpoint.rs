use super::hash::HashType;

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
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

impl Serializable for Outpoint {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.hash.serialize(stream)?;
        self.index.serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for Outpoint {
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let hash = HashType::deserialize(stream)?;
        let index = u32::deserialize(stream)?;

        Ok(Outpoint { 
            hash, 
            index
        })
    }
}
