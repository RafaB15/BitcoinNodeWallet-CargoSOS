use super::outpoint::Outpoint;

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

impl Serializable for TransactionInput {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.previous_output.serialize(stream)?;

        CompactSize::new(self.signature_script.len() as u64).serialize(stream)?;
        self.signature_script.serialize(stream)?;

        self.sequence.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for TransactionInput {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let previous_output = Outpoint::deserialize(stream)?;
        let length_sginature = CompactSize::deserialize(stream)?;
        let signature_script = String::deserialize_fix_size(stream, length_sginature.value as usize)?;
        let sequence = u32::deserialize(stream)?;

        Ok(TransactionInput { 
            previous_output, 
            signature_script, 
            sequence
        })
    }
}
