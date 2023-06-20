use super::{
    block_header::BlockHeader, error_block::ErrorBlock, hash::HashType, merkle_tree::MerkleTree,
    transaction::Transaction,
};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
};

use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: BlockHeader) -> Self {
        Block {
            header,
            transactions: vec![],
        }
    }

    pub fn proof_of_inclusion(&self) -> bool {
        self.header.proof_of_inclusion(&self.transactions)
    }

    pub fn append_transaction(&mut self, transaction: Transaction) -> Result<(), ErrorBlock> {
        match self
            .transactions
            .iter()
            .any(|this_transaction| *this_transaction == transaction)
        {
            true => return Err(ErrorBlock::TransactionAlreadyInBlock),
            false => self.transactions.push(transaction),
        }

        Ok(())
    }

    pub fn get_merkle_path(&self, transaction: &Transaction) -> Result<Vec<HashType>, ErrorBlock> {
        let path: Vec<HashType> =
            match MerkleTree::get_merkle_path(&self.transactions, transaction.clone()) {
                Ok(path) => path,
                Err(_) => return Err(ErrorBlock::CouldNotCalculateMerklePath),
            };

        Ok(path)
    }
}

impl SerializableInternalOrder for Block {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.header.io_serialize(stream)?;

        for transaction in self.transactions.iter() {
            transaction.io_serialize(stream)?;
        }

        Ok(())
    }
}

impl DeserializableInternalOrder for Block {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let header = BlockHeader::io_deserialize(stream)?;
        let length = header.transaction_count.value;

        let mut transactions: Vec<Transaction> = Vec::new();
        for _ in 0..length {
            let transaction = Transaction::io_deserialize(stream)?;
            transactions.push(transaction);
        }

        Ok(Block {
            header,
            transactions,
        })
    }
}
