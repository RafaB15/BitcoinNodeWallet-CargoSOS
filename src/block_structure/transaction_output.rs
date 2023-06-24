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

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_01_correct_transaction_output_serialization () {
        let value: i64 = 10;
        let pk_script: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let transaction_output = TransactionOutput::new(value, pk_script.clone());
        
        let mut fields_serialized: Vec<u8> = Vec::new();
        value.le_serialize(&mut fields_serialized).unwrap();
        CompactSize::new(pk_script.len() as u64).le_serialize(&mut fields_serialized).unwrap();
        pk_script.io_serialize(&mut fields_serialized).unwrap();

        let mut output_serialized = Vec::new();
        transaction_output.io_serialize(&mut output_serialized).unwrap();

        assert_eq!(fields_serialized, output_serialized);
    }

    #[test]
    fn test_02_correct_transaction_output_deserialization () {
        let transaction_output_bytes = [10, 0, 0, 0, 0, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8];
        let actual_transaction_output = TransactionOutput::new(10, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let deserialized_transaction_output = TransactionOutput::io_deserialize(&mut &transaction_output_bytes[..]).unwrap();
        assert_eq!(actual_transaction_output, deserialized_transaction_output);
    }
}