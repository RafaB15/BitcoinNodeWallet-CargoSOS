use super::{
    error_block::ErrorBlock,
    hash::{hash256, HashType, hash256d},
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
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

impl SerializableLittleEndian for Transaction {

    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.le_serialize(stream)?;

        CompactSize::new(self.tx_in.len() as u64).le_serialize(stream)?;
        for tx_in in &self.tx_in {
            tx_in.le_serialize(stream)?;
        }

        CompactSize::new(self.tx_out.len() as u64).le_serialize(stream)?;

        for tx_out in &self.tx_out {
            tx_out.le_serialize(stream)?;
        }

        self.time.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableLittleEndian for Transaction {

    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version = i32::le_deserialize(stream)?;
        let length_tx_in = CompactSize::le_deserialize(stream)?;
        let mut tx_in: Vec<TransactionInput> = Vec::new();
        for _ in 0..length_tx_in.value {
            tx_in.push(TransactionInput::le_deserialize(stream)?);
        }

        let length_tx_out = CompactSize::le_deserialize(stream)?;
        let mut tx_out: Vec<TransactionOutput> = Vec::new();
        for _ in 0..length_tx_out.value {
            tx_out.push(TransactionOutput::le_deserialize(stream)?);
        }

        let time = u32::le_deserialize(stream)?;
        
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
        if self.le_serialize(&mut buffer).is_err() {
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

    pub fn get_vec_txids(transactions: &[Transaction]) -> Result<Vec<HashType>, ErrorBlock> {
        let mut tx_ids = Vec::new();
        for tx in transactions {
            let mut vec_tx = Vec::new();
            match tx.get_tx_id(&mut vec_tx) {
                Ok(txid) => tx_ids.push(txid),
                Err(_) => return Err(ErrorBlock::CouldNotGetTxId),
            };
        }
        Ok(tx_ids)
    }        
}
       

