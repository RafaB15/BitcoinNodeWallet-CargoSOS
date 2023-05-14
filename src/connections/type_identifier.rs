use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
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
    Transaction_id,
    Block,
    Filtered_block,
    Compact_block,
    Witness_transaction,
    Witness_block,
    Filtered_witness_block,
}

impl Serializable for TypeIdentifier {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let value: u32 = match self {
            TypeIdentifier::Transaction_id => TRANSACTION_ID_VALUE,
            TypeIdentifier::Block => BLOCK_VALUE,
            TypeIdentifier::Filtered_block => FILTERED_BLOCK_VALUE,
            TypeIdentifier::Compact_block => COMPACT_BLOCK_VALUE,
            TypeIdentifier::Witness_transaction => WITNESS_TRANSACTION_VALUE,
            TypeIdentifier::Witness_block => WITNESS_BLOCK_VALUE,
            TypeIdentifier::Filtered_witness_block => FILTERED_WITNESS_BLOCK_VALUE,
        };

        match value.serialize(stream) {
            Err(_) => Err(ErrorSerialization::ErrorInSerialization(format!("While serializing the type identifier {:?}", self))),
            _ => Ok(()),
        }
    }
}

impl Deserializable for TypeIdentifier {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let value = u32::deserialize(stream)?;

        match value {
            TRANSACTION_ID_VALUE => Ok(TypeIdentifier::Transaction_id),
            BLOCK_VALUE => Ok(TypeIdentifier::Block),
            FILTERED_BLOCK_VALUE => Ok(TypeIdentifier::Filtered_block),
            COMPACT_BLOCK_VALUE => Ok(TypeIdentifier::Compact_block),
            WITNESS_TRANSACTION_VALUE => Ok(TypeIdentifier::Witness_transaction),
            WITNESS_BLOCK_VALUE => Ok(TypeIdentifier::Witness_block),
            FILTERED_WITNESS_BLOCK_VALUE => Ok(TypeIdentifier::Filtered_witness_block),
            _ => Err(ErrorSerialization::ErrorInDeserialization("While deserializing the type identifier".to_string())),
        }
    }
}