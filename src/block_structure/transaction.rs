use super::{
    error_block::ErrorBlock,
    hash::{hash256, HashType},
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use crate::messages::{
    compact_size::CompactSize, 
};

use std::io::{
    Read,
    Write,
};

use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub version: i32,
    pub tx_in: Vec<TransactionInput>,
    pub tx_out: Vec<TransactionOutput>,
    pub time: u32,
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

impl Deserializable for Transaction {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version = i32::deserialize(stream)?;
        let length_tx_in = CompactSize::deserialize(stream)?;
        let mut tx_in: Vec<TransactionInput> = Vec::new();
        for _ in 0..length_tx_in.value {
            tx_in.push(TransactionInput::deserialize(stream)?);
        }

        let length_tx_out = CompactSize::deserialize(stream)?;
        let mut tx_out: Vec<TransactionOutput> = Vec::new();
        for _ in 0..length_tx_out.value {
            tx_out.push(TransactionOutput::deserialize(stream)?);
        }

        let time = u32::deserialize(stream)?;
        
        Ok(Transaction { 
            version,
            tx_in, 
            tx_out, 
            time
        })
    }
}

impl Transaction {
    pub fn get_tx_id(&self, stream: &mut dyn Write) -> Result<HashType, ErrorBlock> {
        let mut buffer = vec![];
        if self.serialize(&mut buffer).is_err() {
            return Err(ErrorBlock::CouldNotGetTxId);
        }

        // Hash the buffer to get the transaction ID
        let txid = match hash256(&buffer) {
            Ok(txid) => txid,
            Err(_) => return Err(ErrorBlock::CouldNotGetTxId),
        };

        // Write the buffer to the stream
        if stream.write_all(&buffer).is_err() {
            return Err(ErrorBlock::CouldNotWriteTxId);
        }

        Ok(txid)
    }
}