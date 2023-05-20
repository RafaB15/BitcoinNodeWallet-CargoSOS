use crate::messages::compact_size::CompactSize;

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
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
    pub pk_script: Vec<u8>,
}

impl SerializableInternalOrder for TransactionOutput {

    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.value.le_serialize(stream)?;

        CompactSize::new(self.pk_script.len() as u64).le_serialize(stream)?;
        self.pk_script.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for TransactionOutput {
    
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let value = i64::le_deserialize(stream)?;
        let length_pk_script = CompactSize::le_deserialize(stream)?.value;

        let mut pk_script: Vec<u8> = Vec::new();
        for i in 0..length_pk_script {
            let value = match u8::le_deserialize(stream) {
                Ok(value) => value,
                Err(error) => return Err(ErrorSerialization::ErrorInDeserialization(format!(
                    "In transaction output: No se pudo conseguir pk script, tira: {:?}, at {} with {}",
                    error,
                    i,
                    length_pk_script,
                ))),
            };
            pk_script.push(value);
        }

        Ok(TransactionOutput { 
            value, 
            pk_script
        })
    }
}