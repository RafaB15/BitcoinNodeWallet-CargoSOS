use super::{
    block::Block, 
    block_header::BlockHeader, 
    node_chain::NodeChain,
    transaction_output::TransactionOutput,
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

        for index_last_block in self.last_blocks.clone().iter_mut() {

            let last_block = self.get_block_at_mut(*index_last_block)?;

            if last_block.is_equal(&block) {
                return Err(ErrorBlock::TransactionAlreadyInBlock);
            }

            if last_block.is_previous_of(&block) {

                let node = NodeChain::new(block, *index_last_block)?;
                self.blocks.push(node);

                *index_last_block = self.blocks.len() - 1;

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
        
        let mut last_blocks: Vec<Block> = Vec::new();

        for index_last_block in self.last_blocks.iter() {

            let last_block = match self.get_block_at(*index_last_block) {
                Ok(block) => block,
                Err(_) => continue,
            };

            last_blocks.push(last_block.block.clone());
        }

        last_blocks

    }

    pub fn get_utxo(&self) -> Vec<TransactionOutput> {
        todo!()
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
