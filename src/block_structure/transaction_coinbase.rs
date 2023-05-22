use super::{
    error_block::ErrorBlock,
    hash::{hash256, HashType},
    transaction_coinbase_input::TransactionCoinbaseInput,
    transaction_output::TransactionOutput,
};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::messages::compact_size::CompactSize;

use std::io::{Read, Write};

use std::cmp::PartialEq;

const TRANSACTION_INPUT_COUNT: u64 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionCoinbase {
    pub version: i32,
    pub tx_in_coinbase: TransactionCoinbaseInput,
    pub tx_out: Vec<TransactionOutput>,
    pub time: u32,
}

impl SerializableInternalOrder for TransactionCoinbase {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.le_serialize(stream)?;

        CompactSize::new(TRANSACTION_INPUT_COUNT).le_serialize(stream)?;
        self.tx_in_coinbase.io_serialize(stream)?;

        CompactSize::new(self.tx_out.len() as u64).le_serialize(stream)?;
        for tx_out in &self.tx_out {
            tx_out.io_serialize(stream)?;
        }

        self.time.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for TransactionCoinbase {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version = i32::le_deserialize(stream)?;
        let tx_in_coinbase = TransactionCoinbaseInput::io_deserialize(stream)?;

        let length_tx_out = CompactSize::le_deserialize(stream)?.value;

        let mut tx_out: Vec<TransactionOutput> = Vec::new();
        for _ in 0..length_tx_out {
            tx_out.push(TransactionOutput::io_deserialize(stream)?);
        }

        let time = u32::le_deserialize(stream)?;

        let transaction = TransactionCoinbase {
            version,
            tx_in_coinbase,
            tx_out,
            time,
        };

        Ok(transaction)
    }
}

impl TransactionCoinbase {
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
