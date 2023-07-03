use super::{
    block::Block, block_header::BlockHeader, error_block::ErrorBlock, hash::HashType,
    node_chain::NodeChain,
};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::configurations::try_default::TryDefault;

use std::{
    cmp,
    io::{Read, Write},
};

/// It's the internal representation of the block chain
#[derive(Debug, Clone, PartialEq)]
pub struct BlockChain {
    blocks: Vec<NodeChain>,
    last_blocks: Vec<usize>,
}

impl BlockChain {
    pub fn new(block: Block) -> Result<Self, ErrorBlock> {
        let first_node: NodeChain = NodeChain::first(block)?;

        let blocks: Vec<NodeChain> = vec![first_node];
        let last_blocks: Vec<usize> = vec![0];

        Ok(BlockChain {
            blocks,
            last_blocks,
        })
    }

    /// Appends a block header to the block chain
    ///
    /// ### Error
    ///  * `ErrorBlock::`
    pub fn append_header(&mut self, header: BlockHeader) -> Result<(), ErrorBlock> {
        if header.proof_of_work() {
            self.append_block(Block::new(header))
        } else {
            Err(ErrorBlock::ErrorWithProofOfWork)
        }
    }

    /// Appends a vector of block headers to the block chain
    ///
    /// ### Error
    ///  * `ErrorBlock::`
    pub fn append_headers(&mut self, headers: Vec<BlockHeader>) -> Result<u32, ErrorBlock> {
        let mut added_headers = 0;
        for header in headers.iter() {
            match self.append_header(*header) {
                Ok(_) => added_headers += 1,
                Err(ErrorBlock::ErrorWithProofOfWork) => {
                    return Err(ErrorBlock::ErrorWithProofOfWork)
                }
                _ => break,
            }
        }

        Ok(added_headers)
    }

    /// Appends a block to the block chain
    ///
    /// ### Error
    ///  * `ErrorBlock::TransactionAlreadyInBlock`: It will appear when the Transaction is already in the block
    pub fn append_block(&mut self, block: Block) -> Result<(), ErrorBlock> {
        for (i, index_last_block) in self.last_blocks.clone().iter().enumerate() {
            let mut last_block = self.get_block_at(*index_last_block)?;

            if last_block.is_equal(&block) {
                return Err(ErrorBlock::TransactionAlreadyInBlock);
            }

            if last_block.is_previous_of(&block) {
                let node = NodeChain::new(block, *index_last_block, last_block.height)?;
                self.blocks.push(node);

                self.last_blocks[i] = self.blocks.len() - 1;
                return Ok(());
            }

            while let Some(index_previous_node) = last_block.index_previous_node {
                last_block = self.get_block_at_mut(index_previous_node)?;

                if last_block.is_equal(&block) {
                    return Err(ErrorBlock::TransactionAlreadyInBlock);
                }

                if last_block.is_previous_of(&block) {
                    let node = NodeChain::new(block, index_previous_node, last_block.height)?;
                    self.blocks.push(node);

                    self.last_blocks.push(self.blocks.len() - 1);
                    return Ok(());
                }
            }
        }

        Err(ErrorBlock::CouldNotAppendBlock)
    }

    /// Updating the information of a block with its header hash
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotUpdate`: It will appear when the block is not in the blockchain.
    pub fn update_block(&mut self, block: Block) -> Result<(), ErrorBlock> {
        for current_block in self.blocks.iter_mut().rev() {
            if current_block.is_equal(&block) {
                return current_block.update_block(block);
            }
        }

        Err(ErrorBlock::CouldNotUpdate)
    }

    /// Get all blocks after the given timestamp
    pub fn get_blocks_after_timestamp(&self, timestamp: u32) -> Vec<Block> {
        let mut blocks_after_timestamp: Vec<Block> = Vec::new();

        for current_block in self.blocks.iter() {
            if current_block.block.header.time > timestamp {
                blocks_after_timestamp.push(current_block.block.clone());
            }
        }

        blocks_after_timestamp
    }

    /// Get all completed blocks
    pub fn get_all_blocks(&self) -> Vec<Block> {
        self.blocks
            .iter()
            .filter_map(|node| match !node.block.transactions.is_empty() {
                true => Some(node.block.clone()),
                false => None,
            })
            .collect()
    }

    /// Get the block at the end of all forks
    pub fn latest(&self) -> Vec<Block> {
        let mut latest: Vec<Block> = Vec::new();

        for index_last_block in self.last_blocks.iter() {
            let last_block = match self.get_block_at(*index_last_block) {
                Ok(block) => block,
                Err(_) => continue,
            };

            latest.push(last_block.block.clone());
        }

        latest
    }

    pub fn headers_to_update(&self, go_back: usize) -> Vec<Block> {
        let mut latest: Vec<Block> = Vec::new();

        for index_last_block in self.last_blocks.iter() {
            match self.get_block_at(*index_last_block) {
                Ok(last_block) => latest.push(last_block.block),
                Err(_) => continue,
            };

            let previous_index = cmp::max(0, *index_last_block as i32 - go_back as i32) as usize;

            match self.get_block_at(previous_index) {
                Ok(last_block) => latest.push(last_block.block),
                Err(_) => continue,
            };
        }

        latest
    }

    /// Get the latests node chains from the blockchain
    fn get_latests_node_chains(&self) -> Vec<NodeChain> {
        let mut latest: Vec<NodeChain> = Vec::new();

        for index_last_block in self.last_blocks.iter() {
            let last_block = match self.get_block_at(*index_last_block) {
                Ok(block) => block,
                Err(_) => continue,
            };

            latest.push(last_block);
        }

        latest
    }

    /// Get the node at the given index
    ///
    /// ### Error
    ///  * `ErrorBlock::NodeChainReferenceNotFound`: It will appear when a node position it's not found in the block chain
    fn get_block_at(&self, index: usize) -> Result<NodeChain, ErrorBlock> {
        match self.blocks.get(index) {
            Some(block) => Ok(block.clone()),
            None => Err(ErrorBlock::NodeChainReferenceNotFound),
        }
    }

    /// Get the node at the given index
    ///
    /// ### Error
    ///  * `ErrorBlock::NodeChainReferenceNotFound`: It will appear when a node position it's not found in the block chain
    fn get_block_at_mut(&mut self, index: usize) -> Result<NodeChain, ErrorBlock> {
        match self.blocks.get(index) {
            Some(block) => Ok(block.clone()),
            None => Err(ErrorBlock::NodeChainReferenceNotFound),
        }
    }

    /// Rids the blockchain of forks
    ///  
    /// ### Error
    ///  * `ErrorBlock::NodeChainReferenceNotFound`: It will appear when a node position it's not found in the block chain
    fn cleanse_block_chain(&mut self) -> Result<(), ErrorBlock> {
        let mut latest_nodes = self.get_latests_node_chains();
        if latest_nodes.len() == 1 {
            return Ok(());
        }
        let mut main_node = match latest_nodes
            .iter()
            .enumerate()
            .max_by_key(|(_, node)| node.height)
        {
            Some((index, _)) => latest_nodes.remove(index),
            None => return Err(ErrorBlock::NodeChainReferenceNotFound),
        };
        let mut main_chain_values: Vec<NodeChain> = Vec::new();

        while !latest_nodes.is_empty() {
            let current_biggest_index = match latest_nodes
                .iter()
                .enumerate()
                .max_by_key(|(_, node)| node.height)
            {
                Some((index, _)) => index,
                None => break,
            };

            let current_biggest_node = latest_nodes[current_biggest_index].clone();

            if current_biggest_node == main_node {
                latest_nodes.remove(current_biggest_index);
                continue;
            }

            let current_biggest_height = current_biggest_node.height;
            let main_height = main_node.height;

            if main_height > current_biggest_height {
                main_chain_values.insert(0, main_node.clone());
                main_node = match main_node.index_previous_node {
                    Some(index) => self.get_block_at_mut(index)?,
                    None => return Err(ErrorBlock::NodeChainReferenceNotFound),
                };
            } else {
                latest_nodes[current_biggest_index] =
                    match latest_nodes[current_biggest_index].index_previous_node {
                        Some(index) => self.get_block_at_mut(index)?,
                        None => return Err(ErrorBlock::NodeChainReferenceNotFound),
                    };
            }
        }
        main_chain_values.insert(0, main_node.clone());

        let main_chain_update_index = match main_node.index_previous_node {
            Some(index) => index,
            None => return Err(ErrorBlock::NodeChainReferenceNotFound),
        };

        let mut index_temp = main_chain_update_index;
        for node in main_chain_values.iter_mut() {
            node.index_previous_node = Some(index_temp);
            index_temp += 1;
        }
        main_chain_values.insert(0, self.get_block_at_mut(main_chain_update_index)?);
        self.blocks
            .splice(main_chain_update_index.., main_chain_values);
        self.last_blocks = vec![index_temp];
        println!("{:?}", self.blocks);
        Ok(())
    }

    /// Returns the header that matches the given hash
    fn get_node_chain_with_hash(&self, header_hash: &HashType) -> Option<NodeChain> {
        for node_chain in self.blocks.iter() {
            if node_chain.header_hash == *header_hash {
                return Some(node_chain.clone());
            }
        }
        None
    }

    /// Returns the most reacents out of the headers that match the given hashes
    pub fn get_most_recent_hash(&self, hashes: Vec<HashType>) -> Result<HashType, ErrorBlock> {
        let mut nodes: Vec<NodeChain> = Vec::new();
        for hash in hashes.iter() {
            match self.get_node_chain_with_hash(hash) {
                Some(node) => {
                    nodes.push(node);
                }
                None => continue,
            };
        }
        match nodes.iter().max_by_key(|node| node.height) {
            Some(node) => Ok(node.header_hash),
            None => match BlockHeader::generate_genesis_block_header().get_hash256d() {
                Ok(hash) => Ok(hash),
                Err(_) => Err(ErrorBlock::ErrorHashingBlockHeader),
            },
        }
    }

    /// Gets a maximum of 2000 headers from the given hash
    pub fn get_headers_from_header_hash(
        &mut self,
        header_hash: &HashType,
        stop_hash: &HashType,
    ) -> Result<Vec<BlockHeader>, ErrorBlock> {
        let mut headers: Vec<BlockHeader> = Vec::new();
        let mut save = false;

        if self.cleanse_block_chain().is_err() {
            return Err(ErrorBlock::ErrorCleansingBlockChain);
        }

        for node in self.blocks.iter() {
            if node.header_hash == *header_hash {
                save = true;
                continue;
            }
            if save {
                headers.push(node.block.header);
                if (node.header_hash == *stop_hash) || (headers.len() >= 2000) {
                    break;
                }
            }
        }
        Ok(headers)
    }

    /// Gets a block with the given hash
    pub fn get_block_with_hash(&self, header_hash: &HashType) -> Option<Block> {
        if let Some(node) = self.get_node_chain_with_hash(header_hash) {
            return Some(node.block);
        };
        None
    }
}

impl TryDefault for BlockChain {
    type Error = ErrorBlock;

    fn try_default() -> Result<Self, Self::Error> {
        let genesis_header: BlockHeader = BlockHeader::generate_genesis_block_header();
        let genesis_block: Block = Block::new(genesis_header);

        BlockChain::new(genesis_block)
    }
}

impl SerializableInternalOrder for BlockChain {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let mut block_chain: Vec<u8> = Vec::new();
        for node_chain in self.blocks.iter() {
            node_chain.io_serialize(&mut block_chain)?;
        }

        for index_last_block in self.last_blocks.iter() {
            (*index_last_block as u64).le_serialize(&mut block_chain)?;
        }

        let mut header: Vec<u8> = Vec::new();

        (self.last_blocks.len() as u64).le_serialize(&mut header)?;
        (self.blocks.len() as u64).le_serialize(&mut header)?;

        header.io_serialize(stream)?;
        block_chain.io_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for BlockChain {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let last_blocks_count = u64::le_deserialize(stream)?;
        let headers_count = u64::le_deserialize(stream)?;

        let mut node_chains: Vec<NodeChain> = Vec::new();
        for _ in 0..headers_count {
            node_chains.push(NodeChain::io_deserialize(stream)?);
        }

        let mut last_blocks: Vec<usize> = Vec::new();
        for _ in 0..last_blocks_count {
            last_blocks.push(u64::le_deserialize(stream)? as usize);
        }

        Ok(BlockChain {
            blocks: node_chains,
            last_blocks,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::block_structure::{
        block_version, compact256::Compact256, hash::HashType, outpoint::Outpoint,
        transaction::Transaction, transaction_input::TransactionInput,
        transaction_output::TransactionOutput,
    };

    use super::*;
    use crate::messages::compact_size::CompactSize;

    fn create_transaction(time: u32, index: u32) -> Transaction {
        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], index), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time,
        }
    }

    fn create_block(previous_header: HashType, transaction_count: u64, time: u32) -> Block {
        Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            previous_header,
            [0; 32],
            time,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(transaction_count),
        ))
    }

    #[test]
    fn test_01_correct_append_header() {
        let block = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(0),
        ));

        let hash_of_first_block_header = block.header.get_hash256d().unwrap();

        let mut blockchain = BlockChain::new(block).unwrap();

        let header_to_append = BlockHeader::new(
            block_version::BlockVersion::version(1),
            hash_of_first_block_header.clone(),
            [0; 32],
            0,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(0),
        );

        blockchain.append_header(header_to_append.clone()).unwrap();
        assert_eq!(blockchain.blocks[1].block.header, header_to_append);
    }

    #[test]
    fn test_02_correct_block_update() {
        let transaction_input = TransactionInput::new(
            Outpoint::new([1; 32], 23),
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: "Prueba out".as_bytes().to_vec(),
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        let empty_block = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(0),
        ));

        let mut block_with_transactions = empty_block.clone();
        block_with_transactions
            .append_transaction(transaction.clone())
            .unwrap();

        let mut blockchain = BlockChain::new(empty_block).unwrap();

        blockchain.update_block(block_with_transactions).unwrap();

        assert_eq!(blockchain.blocks[0].block.transactions[0], transaction);
    }

    #[test]
    fn test_03_correct_get_block_after_timestamp() {
        let block = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(0),
        ));

        let hash_of_first_block_header = block.header.get_hash256d().unwrap();

        let mut blockchain = BlockChain::new(block).unwrap();

        let header_to_append = BlockHeader::new(
            block_version::BlockVersion::version(1),
            hash_of_first_block_header.clone(),
            [3; 32],
            5,
            Compact256::from(u32::MAX),
            21,
            CompactSize::new(0),
        );

        blockchain.append_header(header_to_append.clone()).unwrap();

        let block_after_timestamp = blockchain.get_blocks_after_timestamp(3);
        assert_eq!(block_after_timestamp[0].header, header_to_append);
    }

    #[test]
    fn test_04_correct_get_latest() {
        let block = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(0),
        ));

        let hash_of_first_block_header = block.header.get_hash256d().unwrap();

        let mut blockchain = BlockChain::new(block).unwrap();

        let header_to_append = BlockHeader::new(
            block_version::BlockVersion::version(1),
            hash_of_first_block_header.clone(),
            [3; 32],
            5,
            Compact256::from(u32::MAX),
            21,
            CompactSize::new(0),
        );

        blockchain.append_header(header_to_append.clone()).unwrap();

        let last_blocks = blockchain.latest();
        assert_eq!(last_blocks[0].header, header_to_append);
    }

    #[test]
    fn test_05_correct_blockchain_cleansing() {
        let transaction_1 = create_transaction(10, 0);
        let transaction_2 = create_transaction(20, 1);
        let transaction_3 = create_transaction(30, 2);
        let transaction_4 = create_transaction(40, 3);
        let transaction_5 = create_transaction(50, 4);
        let transaction_6 = create_transaction(60, 5);

        let mut block_1 = create_block([0; 32], 1, 1);
        block_1.append_transaction(transaction_1).unwrap();

        let mut block_2 = create_block(block_1.header.get_hash256d().unwrap(), 1, 2);
        block_2.append_transaction(transaction_2).unwrap();

        let mut block_3 = create_block(block_2.header.get_hash256d().unwrap(), 1, 3);
        block_3.append_transaction(transaction_3).unwrap();

        let mut block_4 = create_block(block_2.header.get_hash256d().unwrap(), 1, 4);
        block_4.append_transaction(transaction_4).unwrap();

        let mut block_5 = create_block(block_4.header.get_hash256d().unwrap(), 1, 5);
        block_5.append_transaction(transaction_5).unwrap();

        let mut block_6 = create_block(block_5.header.get_hash256d().unwrap(), 1, 6);
        block_6.append_transaction(transaction_6).unwrap();

        let mut blockchain = BlockChain::new(block_1).unwrap();
        blockchain.append_block(block_2).unwrap();
        blockchain.append_block(block_3).unwrap();
        blockchain.append_block(block_4).unwrap();
        blockchain.append_block(block_5).unwrap();
        blockchain.append_block(block_6).unwrap();

        assert_eq!(blockchain.last_blocks.len(), 2);
        assert_eq!(blockchain.last_blocks, vec![2, 5]);
        assert!(blockchain.cleanse_block_chain().is_ok());
        assert_eq!(blockchain.last_blocks.len(), 1);
        assert_eq!(blockchain.last_blocks, vec![4]);
        assert_eq!(blockchain.blocks.len(), 5);
    }

    #[test]
    fn test_06_correct_headers_from_hash() {
        let block_1 = create_block([0; 32], 1, 1);
        let block_2 = create_block(block_1.header.get_hash256d().unwrap(), 2, 2);
        let block_3 = create_block(block_2.header.get_hash256d().unwrap(), 3, 3);
        let block_4 = create_block(block_3.header.get_hash256d().unwrap(), 4, 4);
        let block_5 = create_block(block_4.header.get_hash256d().unwrap(), 5, 5);
        let block_6 = create_block(block_5.header.get_hash256d().unwrap(), 6, 6);

        let mut blockchain = BlockChain::new(block_1).unwrap();
        blockchain.append_block(block_2).unwrap();
        blockchain.append_block(block_3.clone()).unwrap();
        blockchain.append_block(block_4.clone()).unwrap();
        blockchain.append_block(block_5.clone()).unwrap();
        blockchain.append_block(block_6.clone()).unwrap();

        let headers = blockchain
            .get_headers_from_header_hash(
                &block_3.header.get_hash256d().unwrap(),
                &block_6.header.get_hash256d().unwrap(),
            )
            .unwrap();
        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0], block_4.header);
        assert_eq!(headers[1], block_5.header);
        assert_eq!(headers[2], block_6.header);
    }

    #[test]
    fn test_07_correct_get_most_recent_hash() {
        let block_1 = create_block([0; 32], 1, 1);
        let block_2 = create_block(block_1.header.get_hash256d().unwrap(), 2, 2);
        let block_3 = create_block(block_2.header.get_hash256d().unwrap(), 3, 3);
        let block_4 = create_block(block_3.header.get_hash256d().unwrap(), 4, 4);
        let block_5 = create_block(block_4.header.get_hash256d().unwrap(), 5, 5);
        let block_6 = create_block(block_5.header.get_hash256d().unwrap(), 6, 6);

        let mut blockchain = BlockChain::new(block_1).unwrap();
        blockchain.append_block(block_2.clone()).unwrap();
        blockchain.append_block(block_3.clone()).unwrap();
        blockchain.append_block(block_4).unwrap();
        blockchain.append_block(block_5).unwrap();
        blockchain.append_block(block_6.clone()).unwrap();

        let hashes = vec![
            block_2.header.get_hash256d().unwrap(),
            block_3.header.get_hash256d().unwrap(),
            block_6.header.get_hash256d().unwrap(),
        ];

        let most_recent_hash = blockchain.get_most_recent_hash(hashes).unwrap();
        assert_eq!(most_recent_hash, block_6.header.get_hash256d().unwrap());
    }
}
