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

/// It's the representation of a block in the block chain
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

    /// Verifies that the merkle root hash is correct
    pub fn proof_of_inclusion(&self) -> bool {
        self.header.proof_of_inclusion(&self.transactions)
    }

    /// Appends the transaction to the block if it's not already in the block
    ///
    /// ### Error
    ///  * `ErrorBlock::TransactionAlreadyInBlock`: It will appear when the Transaction is already in the block
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

    /// Calculate the merkle path for a transaction in the block
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotCalculateMerklePath`: It will appear when the merkle path could not be calculated
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

#[cfg(test)]

mod tests {
    use super::*;

    use crate::block_structure::{
        block_version, compact256::Compact256, outpoint::Outpoint,
        transaction_input::TransactionInput, transaction_output::TransactionOutput,
    };

    use crate::messages::compact_size::CompactSize;

    #[test]
    fn test_01_correct_block_serialization() {
        let block_header = BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        );

        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction_1 = Transaction {
            version: 1,
            tx_in: vec![transaction_input],
            tx_out: vec![transaction_output],
            time: 0,
        };

        let transaction_2 = transaction_1.clone();

        let transactions = vec![transaction_1, transaction_2];

        let block = Block {
            header: block_header.clone(),
            transactions: transactions.clone(),
        };

        let mut serialized_fields = Vec::new();
        block_header.io_serialize(&mut serialized_fields).unwrap();
        for transaction in transactions.iter() {
            transaction.io_serialize(&mut serialized_fields).unwrap();
        }

        let mut serialized_block = Vec::new();
        block.io_serialize(&mut serialized_block).unwrap();

        assert_eq!(serialized_fields, serialized_block);
    }

    #[test]
    fn test_02_correct_block_deserialization() {
        let block_header = BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        );

        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction_1 = Transaction {
            version: 1,
            tx_in: vec![transaction_input],
            tx_out: vec![transaction_output],
            time: 0,
        };

        let transaction_2 = transaction_1.clone();

        let transactions = vec![transaction_1, transaction_2];

        let block = Block {
            header: block_header.clone(),
            transactions: transactions.clone(),
        };

        let mut serialized_block = Vec::new();
        block.io_serialize(&mut serialized_block).unwrap();

        let deserialized_block = Block::io_deserialize(&mut serialized_block.as_slice()).unwrap();

        assert_eq!(block, deserialized_block);
    }

    #[test]
    fn test_03_correct_append_transaction() {
        let block_header = BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(1),
        );

        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction_1 = Transaction {
            version: 1,
            tx_in: vec![transaction_input],
            tx_out: vec![transaction_output],
            time: 0,
        };

        let transaction_input =
            TransactionInput::new(Outpoint::new([2; 32], 26), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction_2 = Transaction {
            version: 1,
            tx_in: vec![transaction_input],
            tx_out: vec![transaction_output],
            time: 0,
        };

        let transactions = vec![transaction_1];

        let mut block = Block {
            header: block_header.clone(),
            transactions: transactions.clone(),
        };

        assert!(block.transactions.len() == 1);
        block.append_transaction(transaction_2).unwrap();
        assert!(block.transactions.len() == 2);
    }

    #[test]
    fn test_04_cannot_append_a_transaction_already_in_block() {
        let block_header = BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(1),
        );

        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction_1 = Transaction {
            version: 1,
            tx_in: vec![transaction_input],
            tx_out: vec![transaction_output],
            time: 0,
        };

        let transaction_2 = transaction_1.clone();

        let transactions = vec![transaction_1];

        let mut block = Block {
            header: block_header.clone(),
            transactions: transactions.clone(),
        };

        assert!(block.transactions.len() == 1);
        assert!(block.append_transaction(transaction_2).is_err());
    }
}
