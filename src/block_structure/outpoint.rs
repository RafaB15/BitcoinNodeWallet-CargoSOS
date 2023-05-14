use super::hash::HashType;
use crate::serialization::{
    serializable::Serializable,
    error_serialization::ErrorSerialization,
};

use std::io::Write;

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