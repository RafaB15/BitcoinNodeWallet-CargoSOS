use super::{
    block_header::BlockHeader, 
    transaction::Transaction,
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
    hash::{
        HashType,
        hash256d,
    },
    merkle_tree::MerkleTree,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};


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

        match self.transactions.iter().any(|this_transaction| *this_transaction == transaction) {
            true => return Err(ErrorBlock::TransactionAlreadyInBlock),
            false => self.transactions.push(transaction),
        }

        Ok(())
    }

    pub fn get_merkle_path(&self, transaction: &Transaction) -> Result<Vec<HashType>, ErrorBlock> {
        let path: Vec<HashType> = match MerkleTree::get_merkle_path(
            &self.transactions, 
            transaction.clone()
        ){
            Ok(path) => path,
            Err(_) => return Err(ErrorBlock::CouldNotCalculateMerklePath),
        };
        Ok(path)
    }    

    pub fn remove_spent_transactions_in_list(&self, utxo: &mut Vec<(TransactionOutput, HashType, u32)>) {
        for transaction in &self.transactions {
            for input in &transaction.tx_in {
                for (output, transaction_hash, index) in utxo.iter_mut() {
                    if input.previous_output.hash.eq(transaction_hash)  && input.previous_output.index == *index{
                        output.value = -1;
                    }
                }
            }
        } 
    }

    pub fn add_utxo_to_list(&self, utxo: &mut Vec<(TransactionOutput, HashType, u32)>) {
        for transaction in &self.transactions {
            let mut serialized_transaction: Vec<u8> = Vec::new();
            match transaction.io_serialize(&mut serialized_transaction) {
                Ok(_) => (),
                Err(_) => continue,
            }
            let hashed_transaction = match hash256d(&serialized_transaction) {
                Ok(hashed_transaction) => hashed_transaction,
                Err(_) => continue,
            };

            let mut index_utxo = 0;

            for output in &transaction.tx_out {
                utxo.push((output.clone(), hashed_transaction, index_utxo));
                index_utxo += 1;
            }
        }
    }

    pub fn update_utxo_list(&self, utxo: &mut Vec<(TransactionOutput, HashType, u32)>) {
        self.add_utxo_to_list(utxo);
        self.remove_spent_transactions_in_list(utxo);
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
