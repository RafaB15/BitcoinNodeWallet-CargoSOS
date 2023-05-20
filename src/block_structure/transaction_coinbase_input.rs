use super::hash::HashType;

use crate::messages::compact_size::CompactSize;

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_fix_size::DeserializableFixSize,
    error_serialization::ErrorSerialization, 

};

use std::io::{
    Write,
    Read,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionCoinbaseInput {
    pub hash: HashType,       // should be null [32-byte null]
    pub index: u32,           // should be UINT32_MAX [0xffffffff]
    pub height: Vec<u8>,          // should be script [Varies (4)]
    pub coinbase_script: Vec<u8>, // should be None
    pub sequence: u32,        // should be uint32_t [4]
}

impl SerializableInternalOrder for TransactionCoinbaseInput {

    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        self.hash.le_serialize(stream)?;
        self.index.le_serialize(stream)?;
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
        
        let hash = HashType::le_deserialize(stream)?;
        let index = u32::le_deserialize(stream)?;
        let coinbase_script_length = CompactSize::le_deserialize(stream)?.value;

        let height_length = match u8::le_deserialize(stream){
            Ok(value) => value,
            Err(error) => return Err(ErrorSerialization::ErrorInDeserialization(format!(
                "In transaction coinbase: No se pudo conseguir height lenght, tira: {:?}",
                error,
            ))),
        };
        let mut height: Vec<u8> = Vec::new();
        for _ in 0..height_length {
            let value = match u8::le_deserialize(stream) {
                Ok(value) => value,
                Err(error) => return Err(ErrorSerialization::ErrorInDeserialization(format!(
                    "In transaction coinbase: No se pudo conseguir height, tira: {:?}",
                    error,
                ))),
            };
            height.push(value);
        }

        let mut coinbase_script: Vec<u8> = Vec::new();

        for _ in 0..coinbase_script_length {
            let value = match u8::le_deserialize(stream) {
                Ok(value) => value,
                Err(error) => return Err(ErrorSerialization::ErrorInDeserialization(format!(
                    "In transaction coinbase: No se pudo conseguir coinbase script length, tira: {:?}",
                    error,
                ))),
            };
            coinbase_script.push(value);
        }

        let sequence = u32::le_deserialize(stream)?;
        
        Ok(TransactionCoinbaseInput {
            hash,
            index,
            height,
            coinbase_script,
            sequence,
        })
    }
}
