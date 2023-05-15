use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};

pub struct Ping {
    pub nonce: u64,
}

impl Serializable for Ping {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.nonce.serialize(stream)
    }
}

impl Deserializable for Ping {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(Ping {
            nonce: u64::deserialize(stream)?,
        })
    }
}