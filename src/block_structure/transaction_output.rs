use crate::serialization::{
    serializable::Serializable,
    error_serialization::ErrorSerialization,
};
use crate::messages::compact_size::CompactSize;
use std::io::Write;

pub struct TransactionOutput {
    pub value: i64,
    pub public_key: String,
}

impl Serializable for TransactionOutput {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.value.serialize(stream)?;

        CompactSize::new(self.public_key.len() as u64).serialize(stream)?;
        self.public_key.serialize(stream)?;

        Ok(()) 
    }
}