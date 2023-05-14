use super::{
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
};

use crate::serialization::{
    serializable::Serializable,
    error_serialization::ErrorSerialization,
};

use crate::messages::compact_size::CompactSize;

use std::io::Write;

pub struct Transaction {
    pub version: i32,
    pub tx_in: Vec<TransactionInput>,
    pub tx_out: Vec<TransactionOutput>,
    pub time: u32,
}

impl Transaction {
    pub fn new(version: i32, tx_in: Vec<TransactionInput>, tx_out: Vec<TransactionOutput>, time: u32) -> Transaction {
        Transaction {
            version,
            tx_in,
            tx_out,
            time,
        }
    }
}

impl Serializable for Transaction {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.serialize(stream)?;

        CompactSize::new(self.tx_in.len() as u64).serialize(stream)?;
        for tx_in in &self.tx_in {
            tx_in.serialize(stream)?;
        }

        CompactSize::new(self.tx_out.len() as u64).serialize(stream)?;

        for tx_out in &self.tx_out {
            tx_out.serialize(stream)?;
        }

        self.time.serialize(stream)?;
        Ok(())
    }
}