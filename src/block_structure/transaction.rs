use super::{
    hash::{
        hash256, 
        HashType,
    },
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_internal_order::SerializableInternalOrder, deserializable_internal_order::DeserializableInternalOrder,
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

impl SerializableInternalOrder for Transaction {

    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.le_serialize(stream)?;

        CompactSize::new(self.tx_in.len() as u64).le_serialize(stream)?;
        for tx_in in self.tx_in.iter() {
            tx_in.io_serialize(stream)?;
        }

        CompactSize::new(self.tx_out.len() as u64).le_serialize(stream)?;
        for tx_out in &self.tx_out {
            tx_out.io_serialize(stream)?;
        }

        self.time.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for Transaction {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version = i32::le_deserialize(stream)?;
        
        let length_tx_in = CompactSize::le_deserialize(stream)?.value;
        let mut tx_in: Vec<TransactionInput> = Vec::new();
        for _ in 0..length_tx_in {
            tx_in.push(TransactionInput::io_deserialize(stream)?);
        }

        let length_tx_out = CompactSize::le_deserialize(stream)?.value;
        let mut tx_out: Vec<TransactionOutput> = Vec::new();
        for _ in 0..length_tx_out {
            tx_out.push(TransactionOutput::io_deserialize(stream)?);
        }

        println!("We have: {} tx_in y {} tx_out", tx_in.len(), tx_out.len());

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
        if self.io_serialize(&mut buffer).is_err() {
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