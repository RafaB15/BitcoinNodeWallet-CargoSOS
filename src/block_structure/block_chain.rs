use super::{
    block::Block, block_header::BlockHeader, error_block::ErrorBlock, node_chain::NodeChain,
};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::configurations::try_default::TryDefault;

use std::io::{Read, Write};

/// It's the internal representation of the block chain
#[derive(Debug, Clone)]
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
        self.append_block(Block::new(header))
    }

    /// Appends a vector of block headers to the block chain
    ///
    /// ### Error
    ///  * `ErrorBlock::`
    pub fn append_headers(&mut self, headers: Vec<BlockHeader>) -> Result<u32, ErrorBlock> {
        let mut added_headers = 0;
        for header in headers.iter() {
            if !header.proof_of_work() {
                return Err(ErrorBlock::ErrorWithProofOfWork);
            }

            match self.append_header(*header) {
                Ok(_) => added_headers += 1,
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
            let last_block = self.get_block_at(*index_last_block)?;

            if last_block.is_equal(&block) {
                return Err(ErrorBlock::TransactionAlreadyInBlock);
            }

            if last_block.is_previous_of(&block) {
                let node = NodeChain::new(block, *index_last_block)?;
                self.blocks.push(node);

                self.last_blocks[i] = self.blocks.len() - 1;

                return Ok(());
            }

            while let Some(index_previous_node) = last_block.index_previous_node {
                let last_block = self.get_block_at_mut(index_previous_node)?;

                if last_block.is_equal(&block) {
                    return Err(ErrorBlock::TransactionAlreadyInBlock);
                }

                if last_block.is_previous_of(&block) {
                    let node = NodeChain::new(block, *index_last_block)?;
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
            .filter_map(|node| match node.block.transactions.len() > 0 {
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
        block_version, compact256::Compact256, outpoint::Outpoint, transaction::Transaction,
        transaction_input::TransactionInput, transaction_output::TransactionOutput,
    };

    use super::*;
    use crate::messages::compact_size::CompactSize;

    #[test]
    fn test_01_correct_append_header() {
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
            [0; 32],
            0,
            Compact256::from(10),
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
            Compact256::from(10),
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
            Compact256::from(10),
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
            Compact256::from(10),
            21,
            CompactSize::new(0),
        );

        blockchain.append_header(header_to_append.clone()).unwrap();

        let last_blocks = blockchain.latest();
        assert_eq!(last_blocks[0].header, header_to_append);
    }
}
