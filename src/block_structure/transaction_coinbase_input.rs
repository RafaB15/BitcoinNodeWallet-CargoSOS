use super::{
    outpoint::Outpoint,
};

use crate::messages::compact_size::CompactSize;

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization, 
};

use std::io::{
    Write,
    Read,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionCoinbaseInput {
    pub previous_output: Outpoint,
    pub height: Vec<u8>,          // should be script [Varies (4)]
    pub coinbase_script: Vec<u8>, // should be None
    pub sequence: u32,        // should be uint32_t [4]
}

impl SerializableInternalOrder for TransactionCoinbaseInput {

    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        self.previous_output.io_serialize(stream)?;
        CompactSize::new(self.coinbase_script.len() as u64).le_serialize(stream)?;
        (self.height.len() as u8).le_serialize(stream)?;
        self.height.le_serialize(stream)?;
        self.coinbase_script.le_serialize(stream)?;
        self.sequence.le_serialize(stream)?;
        
        Ok(())
    }
}

impl DeserializableInternalOrder for TransactionCoinbaseInput {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        
        let previous_output = Outpoint::io_deserialize(stream)?;
        let coinbase_script_length = CompactSize::le_deserialize(stream)?.value;

        let height_length = u8::le_deserialize(stream)?;
        let mut height: Vec<u8> = Vec::new();
        for _ in 0..height_length {
            height.push(u8::le_deserialize(stream)?);
        }

        let mut coinbase_script: Vec<u8> = Vec::new();

        for _ in 0..coinbase_script_length {
            coinbase_script.push(u8::le_deserialize(stream)?);
        }

        let sequence = u32::le_deserialize(stream)?;
        
        Ok(TransactionCoinbaseInput {
            previous_output,
            height,
            coinbase_script,
            sequence,
        })
    }
}
