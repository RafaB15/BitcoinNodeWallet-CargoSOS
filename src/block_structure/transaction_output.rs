use crate::messages::compact_size::CompactSize;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use std::io::{Read, Write};

use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionOutput {
    pub value: i64,
    pub pk_script: Vec<u8>,
}

impl TransactionOutput {
    pub fn new(value: i64, pk_script: Vec<u8>) -> TransactionOutput {
        TransactionOutput { value, pk_script }
    }
}

impl SerializableInternalOrder for TransactionOutput {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.value.le_serialize(stream)?;

        CompactSize::new(self.pk_script.len() as u64).le_serialize(stream)?;
        self.pk_script.io_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for TransactionOutput {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let value = i64::le_deserialize(stream)?;
        let length_pk_script = CompactSize::le_deserialize(stream)?.value;

        let mut pk_script: Vec<u8> = Vec::new();
        for _ in 0..length_pk_script {
            pk_script.push(u8::le_deserialize(stream)?);
        }

        Ok(TransactionOutput { value, pk_script })
    }
}
