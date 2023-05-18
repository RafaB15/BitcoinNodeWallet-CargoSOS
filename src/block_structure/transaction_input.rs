use super::outpoint::Outpoint;

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
pub struct TransactionInput {
    pub previous_output: Outpoint,
    pub signature_script: String,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(
        previous_output: Outpoint,
        signature_script: String,
        sequence: u32,
    ) -> TransactionInput {
        TransactionInput {
            previous_output,
            signature_script,
            sequence,
        }
    }
}

impl SerializableLittleEndian for TransactionInput {

    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.previous_output.le_serialize(stream)?;

        CompactSize::new(self.signature_script.len() as u64).le_serialize(stream)?;
        self.signature_script.le_serialize(stream)?;

        self.sequence.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableLittleEndian for TransactionInput {

    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let previous_output = Outpoint::le_deserialize(stream)?;
        let length_sginature = CompactSize::le_deserialize(stream)?;
        let signature_script = String::deserialize_fix_size(stream, length_sginature.value as usize)?;
        let sequence = u32::le_deserialize(stream)?;

        Ok(TransactionInput { 
            previous_output, 
            signature_script, 
            sequence
        })
    }
}
