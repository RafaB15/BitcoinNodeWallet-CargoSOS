use crate::messages::compact_size::CompactSize;

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    deserializable_fix_size::DeserializableFixSize,
    error_serialization::ErrorSerialization, 
};

use std::io::{
    Read,
    Write,
};

use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionOutput {
    pub value: i64,
    pub pk_script: String,
}

impl Serializable for TransactionOutput {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.value.serialize(stream)?;

        CompactSize::new(self.pk_script.len() as u64).serialize(stream)?;
        self.pk_script.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for TransactionOutput {
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let value = i64::deserialize(stream)?;
        let length_pk_script = CompactSize::deserialize(stream)?;
        let pk_script = String::deserialize_fix_size(stream, length_pk_script.value as usize)?;

        Ok(TransactionOutput { 
            value, 
            pk_script
        })
    }
}