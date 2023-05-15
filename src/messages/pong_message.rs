use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};

pub struct Pong {
    pub nonce: u64,
}

impl Serializable for Pong {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.nonce.serialize(stream)
    }
}

impl Deserializable for Pong {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(Pong {
            nonce: u64::deserialize(stream)?,
        })
    }
}