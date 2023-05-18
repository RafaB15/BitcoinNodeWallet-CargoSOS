use crate::messages::compact_size::CompactSize;

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
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

impl SerializableLittleEndian for TransactionOutput {

    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.value.le_serialize(stream)?;

        CompactSize::new(self.pk_script.len() as u64).le_serialize(stream)?;
        self.pk_script.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableLittleEndian for TransactionOutput {
    
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let value = i64::le_deserialize(stream)?;
        let length_pk_script = CompactSize::le_deserialize(stream)?;
        let pk_script = String::deserialize_fix_size(stream, length_pk_script.value as usize)?;

        Ok(TransactionOutput { 
            value, 
            pk_script
        })
    }
}