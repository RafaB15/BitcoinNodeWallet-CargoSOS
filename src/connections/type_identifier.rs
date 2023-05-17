use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};

const TRANSACTION_ID_VALUE: u32 = 0x01;
const BLOCK_VALUE: u32 = 0x02;
const FILTERED_BLOCK_VALUE: u32 = 0x03;
const COMPACT_BLOCK_VALUE: u32 = 0x04;
const WITNESS_TRANSACTION_VALUE: u32 = 0x40000001;
const WITNESS_BLOCK_VALUE: u32 = 0x40000002;
const FILTERED_WITNESS_BLOCK_VALUE: u32 = 0x40000003;

#[derive(Debug)]
pub enum TypeIdentifier {
    TransactionId,
    Block,
    FilteredBlock,
    CompactBlock,
    WitnessTransaction,
    WitnessBlock,
    FilteredWitnessBlock,
}

impl SerializableLittleEndian for TypeIdentifier {

    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let value: u32 = match self {
            TypeIdentifier::TransactionId => TRANSACTION_ID_VALUE,
            TypeIdentifier::Block => BLOCK_VALUE,
            TypeIdentifier::FilteredBlock => FILTERED_BLOCK_VALUE,
            TypeIdentifier::CompactBlock => COMPACT_BLOCK_VALUE,
            TypeIdentifier::WitnessTransaction => WITNESS_TRANSACTION_VALUE,
            TypeIdentifier::WitnessBlock => WITNESS_BLOCK_VALUE,
            TypeIdentifier::FilteredWitnessBlock => FILTERED_WITNESS_BLOCK_VALUE,
        };

        match value.le_serialize(stream) {
            Err(_) => Err(ErrorSerialization::ErrorInSerialization(format!("While serializing the type identifier {:?}", self))),
            _ => Ok(()),
        }
    }
}

impl DeserializableLittleEndian for TypeIdentifier {

    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let value = u32::le_deserialize(stream)?;

        match value {
            TRANSACTION_ID_VALUE => Ok(TypeIdentifier::TransactionId),
            BLOCK_VALUE => Ok(TypeIdentifier::Block),
            FILTERED_BLOCK_VALUE => Ok(TypeIdentifier::FilteredBlock),
            COMPACT_BLOCK_VALUE => Ok(TypeIdentifier::CompactBlock),
            WITNESS_TRANSACTION_VALUE => Ok(TypeIdentifier::WitnessTransaction),
            WITNESS_BLOCK_VALUE => Ok(TypeIdentifier::WitnessBlock),
            FILTERED_WITNESS_BLOCK_VALUE => Ok(TypeIdentifier::FilteredWitnessBlock),
            _ => Err(ErrorSerialization::ErrorInDeserialization("While deserializing the type identifier".to_string())),
        }
    }
}