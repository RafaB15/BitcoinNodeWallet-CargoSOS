use crate::messages::compact_size::CompactSize;
use crate::serialization::{error_serialization::ErrorSerialization, serializable::Serializable};
use std::io::Write;

#[derive(Debug, Clone)]
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
