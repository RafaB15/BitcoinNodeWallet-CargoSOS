use super::{
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
    hash::{HashType, hash256},
    error_block::ErrorBlock,
};

use crate::messages::get_headers_message::GetHeadersMessage;
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

impl Transaction {
    pub fn get_tx_id(&self, stream: &mut dyn Write) -> Result<HashType, ErrorBlock> {
        let mut buffer = vec![];
        if self.serialize(&mut buffer).is_err() {
            return Err(ErrorBlock::ErrorCouldNotGetTxId);
        }

        // Hash the buffer to get the transaction ID
        let txid = match hash256(&buffer) {
            Ok(txid) => txid,
            Err(_) => return Err(ErrorBlock::ErrorCouldNotGetTxId),
        };

        // Write the buffer to the stream
        if stream.write_all(&buffer).is_err(){
            return Err(ErrorBlock::ErrorCouldNotWriteTxId);
        }

        Ok(txid)
    }
}
