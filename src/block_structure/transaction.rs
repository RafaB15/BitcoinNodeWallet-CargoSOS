use super::{
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub version: i32,
    pub tx_in: Vec<TransactionInput>,
    pub tx_out: Vec<TransactionOutput>,
    pub time: u32,
}

impl Serializable for Transaction {
    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        todo!()
    }
}

impl Deserializable for Transaction {
    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        todo!()
    }
}