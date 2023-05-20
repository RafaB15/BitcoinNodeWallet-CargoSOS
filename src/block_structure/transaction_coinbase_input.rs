use super::hash::HashType;

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
    pub hash: HashType,       // should be null [32-byte null]
    pub index: u32,           // should be UINT32_MAX [0xffffffff]
    pub height: String,          // should be script [Varies (4)]
    pub coinbase_script: Vec<u8>, // should be None
    pub sequence: u32,        // should be uint32_t [4]
}

impl SerializableInternalOrder for TransactionCoinbaseInput {

    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        self.hash.le_serialize(stream)?;
        self.index.le_serialize(stream)?;
        CompactSize::new(self.coinbase_script.len() as u64).le_serialize(stream)?;
        //self.script_bytes.le_serialize(stream)?;
        self.height.le_serialize(stream)?;
        self.coinbase_script.le_serialize(stream)?;
        self.sequence.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for TransactionCoinbaseInput {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        todo!()
    }
}
