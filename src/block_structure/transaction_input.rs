use super::outpoint::Outpoint;

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
pub struct TransactionInput {
    pub previous_output: Outpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(
        previous_output: Outpoint,
        signature_script: Vec<u8>,
        sequence: u32,
    ) -> TransactionInput {
        TransactionInput {
            previous_output,
            signature_script,
            sequence,
        }
    }
}

impl SerializableInternalOrder for TransactionInput {

    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.previous_output.io_serialize(stream)?;

        CompactSize::new(self.signature_script.len() as u64).le_serialize(stream)?;
        self.signature_script.io_serialize(stream)?;

        self.sequence.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for TransactionInput {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let previous_output = Outpoint::io_deserialize(stream)?;
        let length_sginature = CompactSize::le_deserialize(stream)?.value;

        let mut signature_script: Vec<u8> = Vec::new();
        for i in 0..length_sginature {
            let value = match u8::le_deserialize(stream) {
                Ok(value) => value,
                Err(error) => return Err(ErrorSerialization::ErrorInDeserialization(format!(
                    "En transaction input: No se pudo conseguir pk script, tira: {:?}, at {} with {}",
                    error,
                    i,
                    length_sginature,
                ))),
            };
            signature_script.push(value);
        }
        let sequence = u32::le_deserialize(stream)?;

        Ok(TransactionInput { 
            previous_output, 
            signature_script, 
            sequence
        })
    }
}
