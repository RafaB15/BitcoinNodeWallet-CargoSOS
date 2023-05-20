use super::{
    block_header::BlockHeader, 
    transaction::Transaction,
    transaction_coinbase::TransactionCoinbase,
    error_block::ErrorBlock,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};

use crate::messages::{
    compact_size::CompactSize,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub tx_coinbase: Option<TransactionCoinbase>,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: BlockHeader) -> Self {
        Block {
            header,
            tx_coinbase: None,
            transactions: vec![],
        }
    }

    pub fn proof_of_inclusion(&self) -> bool {
        self.header.proof_of_inclusion(&self.transactions)
    }

    pub fn append_transaction_coinbase(
        &mut self, 
        transaction_coinbase: TransactionCoinbase
    ) -> Result<(), ErrorBlock> 
    {
        match self.tx_coinbase {
            Some(_) => return Err(ErrorBlock::TransactionAlreadyInBlock),
            None => self.tx_coinbase = Some(transaction_coinbase),
        }

        Ok(())
    }

    pub fn append_transaction(&mut self, transaction: Transaction) -> Result<(), ErrorBlock> {

        match self.transactions.iter().any(|this_transaction| *this_transaction == transaction) {
            true => return Err(ErrorBlock::TransactionAlreadyInBlock),
            false => self.transactions.push(transaction),
        }

        Ok(())
    }    
}

impl SerializableInternalOrder for Block {

    fn io_serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        self.header.io_serialize(stream)?;

        match &self.tx_coinbase {
            Some(tx_coinbase) => {
                CompactSize::new((self.transactions.len() + 1) as u64).le_serialize(stream)?;

                println!("We have {} transactions", self.transactions.len() + 1);

                tx_coinbase.io_serialize(stream)?;
                for transaction in self.transactions.iter() {
                    transaction.io_serialize(stream)?;
                }
                
            },
            None => {},
        };
        
        Ok(())
    }
}

impl DeserializableInternalOrder for Block {

    fn io_deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let header = BlockHeader::io_deserialize(stream)?;
        let length = CompactSize::le_deserialize(stream)?.value;

        if length == 0 {
            return Ok(Block::new(header));
        }
        
        let transaction_coinbase = TransactionCoinbase::io_deserialize(stream)?;

        println!("We have {} transactions from deserialization", length);

        let mut transactions: Vec<Transaction> = Vec::new();
        for _ in 1..length {
            let transaction = Transaction::io_deserialize(stream)?;
            transactions.push(transaction);
        }

        Ok(Block {
            header,
            tx_coinbase: Some(transaction_coinbase),
            transactions,
        })
    }
}
