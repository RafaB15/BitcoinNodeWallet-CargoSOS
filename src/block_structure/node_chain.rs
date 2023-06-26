use super::{
    block::Block, block_header::BlockHeader, error_block::ErrorBlock, hash::HashType,
    transaction::Transaction,
};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use std::io::{Read, Write};

pub(super) const NONE_INDEX: u64 = u64::MAX;

/// It's the representation of a node in the block chain
#[derive(Debug, Clone, PartialEq)]
pub(super) struct NodeChain {
    pub block: Block,
    pub header_hash: HashType,

    pub index_previous_node: Option<usize>,
}

impl NodeChain {
    /// It creates a node chain without a previous one
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotHash`: It will appear when a header could not be hash correctly
    pub fn first(block: Block) -> Result<Self, ErrorBlock> {
        let header_hash = match block.header.get_hash256d() {
            Ok(hash) => hash,
            _ => return Err(ErrorBlock::CouldNotHash),
        };

        Ok(NodeChain {
            index_previous_node: None,
            header_hash,
            block,
        })
    }

    /// It creates a node chain with a previous one
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotHash`: It will appear when a header could not be hash correctly
    pub fn new(block: Block, index_previous_node: usize) -> Result<Self, ErrorBlock> {
        let header_hash = match block.header.get_hash256d() {
            Ok(hash) => hash,
            _ => return Err(ErrorBlock::CouldNotHash),
        };

        Ok(NodeChain {
            index_previous_node: Some(index_previous_node),
            header_hash,
            block,
        })
    }

    /// Returns if a given block is the previous of the current one
    pub fn is_previous_of(&self, block: &Block) -> bool {
        self.header_hash
            .eq(&block.header.previous_block_header_hash)
    }

    /// Returns if a given block is the same as the current one
    pub fn is_equal(&self, block: &Block) -> bool {
        let (given_hash, hash) = match (block.header.get_hash256d(), self.header_hash) {
            (Ok(given_hash), hash) => (given_hash, hash),
            _ => return false,
        };

        given_hash.eq(&hash)
    }

    /// It updates the information of the current node chain
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotHash`: It will appear when a header could not be hash correctly
    pub(super) fn update_block(&mut self, block: Block) -> Result<(), ErrorBlock> {
        let header_hash = match block.header.get_hash256d() {
            Ok(hash) => hash,
            _ => return Err(ErrorBlock::CouldNotHash),
        };

        self.header_hash = header_hash;
        self.block = block;

        Ok(())
    }
}

impl SerializableInternalOrder for NodeChain {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.block.header.io_serialize(stream)?;

        (self.block.transactions.len() as u64).le_serialize(stream)?;
        for transaction in &self.block.transactions {
            transaction.io_serialize(stream)?;
        }

        self.header_hash.io_serialize(stream)?;

        match self.index_previous_node {
            Some(index) => (index as u64).le_serialize(stream)?,
            None => NONE_INDEX.le_serialize(stream)?,
        };

        Ok(())
    }
}

impl DeserializableInternalOrder for NodeChain {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut block = Block::new(BlockHeader::io_deserialize(stream)?);

        let transaction_count = u64::le_deserialize(stream)?;
        for _ in 0..transaction_count {
            let transaction = Transaction::io_deserialize(stream)?;
            if block.append_transaction(transaction).is_err() {
                return Err(ErrorSerialization::ErrorWhileReading);
            }
        }

        Ok(NodeChain {
            block,
            header_hash: HashType::io_deserialize(stream)?,
            index_previous_node: match u64::le_deserialize(stream)? {
                NONE_INDEX => None,
                index => Some(index as usize),
            },
        })
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::block_structure::{
        block_version::BlockVersion,
        compact256::Compact256,
        outpoint::Outpoint,
        transaction_output::TransactionOutput,
        transaction_input::TransactionInput,
        transaction::Transaction,
    };
    use crate::messages::compact_size::CompactSize;

    #[test]
    pub fn test_01_correct_first_block() {
        let block = Block::new(BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        ));
        let node_chain = NodeChain::first(block).unwrap();
        assert_eq!(node_chain.index_previous_node, None);
    }

    #[test]
    pub fn test_02_correct_new_nodechain_with_previous_node_chain() {
        let block = Block::new(BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        ));
        let node_chain = NodeChain::new(block, 23).unwrap();
        assert_eq!(node_chain.index_previous_node, Some(23));
    }

    #[test]
    pub fn test_03_correct_is_previous_of() {
        let block = Block::new(BlockHeader::generate_genesis_block_header());
        
        let node_chain = NodeChain::new(block, 23).unwrap();
        let block = Block::new(BlockHeader::new(
            BlockVersion::version(1),
            [
                0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xEA, 0x01, 0xAD, 0x0E, 0xE9, 0x84, 
                0x20, 0x97, 0x79, 0xBA, 0xAE, 0xC3, 0xCE, 0xD9, 0x0F, 0xA3, 0xF4, 0x08, 
                0x71, 0x95, 0x26, 0xF8, 0xD7, 0x7F, 0x49, 0x43
            ],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        ));
        assert!(node_chain.is_previous_of(&block));
    }

    #[test]
    pub fn test_04_correct_is_equal() {
        let block = Block::new(BlockHeader::generate_genesis_block_header());
        let node_chain = NodeChain::new(block.clone(), 23).unwrap();
        assert!(node_chain.is_equal(&block));
    }

    #[test]
    pub fn test_05_correct_node_chain_update() {
        let block_1 = Block::new(BlockHeader::generate_genesis_block_header());
        let mut node_chain = NodeChain::new(block_1, 23).unwrap();
        let block_2 = Block::new(BlockHeader::new(
            BlockVersion::version(1),
            [
                0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xEA, 0x01, 0xAD, 0x0E, 0xE9, 0x84, 
                0x20, 0x97, 0x79, 0xBA, 0xAE, 0xC3, 0xCE, 0xD9, 0x0F, 0xA3, 0xF4, 0x08, 
                0x71, 0x95, 0x26, 0xF8, 0xD7, 0x7F, 0x49, 0x43
            ],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        ));
        assert!(node_chain.update_block(block_2.clone()).is_ok());
        assert!(node_chain.is_equal(&block_2));
    }

    #[test]
    pub fn test_06_correct_node_chain_serialization() {
        let block_header = BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        );

        let transaction_input = TransactionInput::new(
            Outpoint::new(
                [1; 32],
                23,
        ),
            vec![1, 2, 3],
            24,
        );

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

        let node_chain = NodeChain::new(block.clone(), 23).unwrap();

        let mut serialized_fields = Vec::new();
        block_header.io_serialize(&mut serialized_fields).unwrap();
        (block.transactions.len() as u64).le_serialize(&mut serialized_fields).unwrap();
        for transaction in &block.transactions {
            transaction.io_serialize(&mut serialized_fields).unwrap();
        }
        block.header.get_hash256d().unwrap().io_serialize(&mut serialized_fields).unwrap();
        (23 as u64).le_serialize(&mut serialized_fields).unwrap();  

        let mut serialized_node_chain = Vec::new();
        node_chain.io_serialize(&mut serialized_node_chain).unwrap();

        assert_eq!(serialized_fields, serialized_node_chain);
    }

    #[test]
    pub fn test_07_correct_node_chain_deserialization() {
        let block_header = BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(1),
        );

        let transaction_input = TransactionInput::new(
            Outpoint::new(
                [1; 32],
                23,
            ),
            vec![1, 2, 3],
            24,
        );

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

        let transaction_input = TransactionInput::new(
            Outpoint::new(
                [2; 32],
                26,
            ),
            vec![1, 2, 3],
            24,
        );

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

        let transactions = vec![transaction_1, transaction_2];

        let block = Block {
            header: block_header.clone(),
            transactions: transactions.clone(),
        };

        let node_chain = NodeChain::new(block.clone(), 23).unwrap();

        let mut serialized_node_chain = Vec::new();
        node_chain.io_serialize(&mut serialized_node_chain).unwrap();

        let deserialized_node_chain = NodeChain::io_deserialize(&mut serialized_node_chain.as_slice()).unwrap();

        assert!(deserialized_node_chain.is_equal(&block));
    }
}
