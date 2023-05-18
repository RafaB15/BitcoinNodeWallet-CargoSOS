use super::{
    block::Block, 
    block_header::BlockHeader, 
    node_chain::NodeChain,
    error_block::ErrorBlock,
};

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

    pub fn append_header(&mut self, header: BlockHeader) -> Result<(), ErrorBlock> {
        self.append_block(Block::new(header))
    }

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

    pub fn update_block(&mut self, block: Block) -> Result<(), ErrorBlock> {

        for current_block in self.blocks.iter_mut() {

            if current_block.is_equal(&block) {

                return current_block.update_block(block);
            }
        }

        Err(ErrorBlock::CouldNotUpdate)
    }

    pub fn get_blocks_after_timestamp(&self, timestamp: u32) -> Result<Vec<Block>, ErrorBlock> {
        
        let mut blocks_after_timestamp: Vec<Block> = Vec::new();

        for current_block in self.blocks.iter() {

            if current_block.block.header.time > timestamp {

                blocks_after_timestamp.push(current_block.block.clone());
            }
        }

        Ok(blocks_after_timestamp)
    }

    pub fn latest(&self) -> Vec<Block> {
        
        let mut latest: Vec<Block> = Vec::new();

        println!("Last blocks: {:?}", self.last_blocks);

        for index_last_block in self.last_blocks.iter() {

            let last_block = match self.get_block_at(*index_last_block) {
                Ok(block) => block,
                Err(_) => continue,
            };

            latest.push(last_block.block.clone());
        }

        latest
    }

    fn get_block_at(&self, index: usize) -> Result<NodeChain, ErrorBlock> {
        match self.blocks.get(index) {
            Some(block) => Ok(block.clone()),
            None => Err(ErrorBlock::NodeChainReferenceNotFound),
        }
    }

    fn get_block_at_mut(&mut self, index: usize) -> Result<NodeChain, ErrorBlock> {
        match self.blocks.get(index) {
            Some(block) => Ok(block.clone()),
            None => Err(ErrorBlock::NodeChainReferenceNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::block_structure::{
        compact256::Compact256,
        block_version,
        transaction_input::TransactionInput,
        transaction_output::TransactionOutput,
        transaction::Transaction,
        outpoint::Outpoint,
    };
    use crate::messages::compact_size::CompactSize;
    use super::*;

    #[test]
    fn test_01_correct_append_header() {
        let block = Block::new(
            BlockHeader::new(
                block_version::BlockVersion::V1,
                [0; 32],
                [0; 32],
                0,
                Compact256::from(10),
                0,
                CompactSize::new(0),
            )
        );

        let hash_of_first_block_header = block.header.get_hash256d().unwrap();

        let mut blockchain = BlockChain::new(block).unwrap();

        let header_to_append = BlockHeader::new(
            block_version::BlockVersion::V1,
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
            Outpoint { hash: [1;32], index: 23 },
            String::from("Prueba in"),
            24
        );

        let transaction_output = TransactionOutput{
            value: 10, 
            pk_script: String::from("Prueba out")
        };

        let transaction = Transaction {
            version : 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        let empty_block = Block::new(
            BlockHeader::new(
                block_version::BlockVersion::V1,
                [0; 32],
                [0; 32],
                0,
                Compact256::from(10),
                0,
                CompactSize::new(0),
            )
        );

        let mut block_with_transactions = empty_block.clone();
        block_with_transactions.append_transaction(transaction.clone()).unwrap();

        let mut blockchain = BlockChain::new(empty_block).unwrap();

        blockchain.update_block(block_with_transactions).unwrap();

        assert_eq!(blockchain.blocks[0].block.transactions[0], transaction);

    } 

    #[test]
    fn test_03_correct_get_block_after_timestamp() {

        let block = Block::new(
            BlockHeader::new(
                block_version::BlockVersion::V1,
                [0; 32],
                [0; 32],
                0,
                Compact256::from(10),
                0,
                CompactSize::new(0),
            )
        );

        let hash_of_first_block_header = block.header.get_hash256d().unwrap();

        let mut blockchain = BlockChain::new(block).unwrap();

        let header_to_append = BlockHeader::new(
            block_version::BlockVersion::V1,
            hash_of_first_block_header.clone(),
            [3; 32],
            5,
            Compact256::from(10),
            21,
            CompactSize::new(0),
        );

        blockchain.append_header(header_to_append.clone()).unwrap();

        let block_after_timestamp = blockchain.get_blocks_after_timestamp(3).unwrap();
        assert_eq!(block_after_timestamp[0].header, header_to_append);
    }

    #[test]
    fn test_04_correct_get_latest() {

        let block = Block::new(
            BlockHeader::new(
                block_version::BlockVersion::V1,
                [0; 32],
                [0; 32],
                0,
                Compact256::from(10),
                0,
                CompactSize::new(0),
            )
        );

        let hash_of_first_block_header = block.header.get_hash256d().unwrap();
    
        let mut blockchain = BlockChain::new(block).unwrap();

        let header_to_append = BlockHeader::new(
            block_version::BlockVersion::V1,
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