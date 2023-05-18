use super::{
    block::Block, 
    block_header::BlockHeader, 
    node_chain::NodeChain,
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
};

use std::sync::{
    Arc,
    Mutex, 
    MutexGuard,
};

pub type NodeChainRef = Arc<Mutex<NodeChain>>;

pub struct BlockChain {
    
    blocks: Vec<NodeChainRef>,

    last_blocks: Vec<NodeChainRef>,
}

impl BlockChain {

    pub fn new(block: Block) -> Result<Self, ErrorBlock> {

        let first_node: NodeChainRef = Arc::new(Mutex::new(NodeChain::first(block)?));
        
        let blocks: Vec<NodeChainRef> = vec![first_node.clone()];
        let last_blocks: Vec<NodeChainRef> = vec![first_node];
        
        Ok(BlockChain { 
            blocks,
            last_blocks,
        })
    }

    pub fn append_header(&mut self, header: BlockHeader) -> Result<(), ErrorBlock> {
        self.append_block(Block::new(header))
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), ErrorBlock> {

        for last_block_ref in self.last_blocks.iter_mut() {

            let last_clone_ref = last_block_ref.clone();
            let last_block = Self::get_mut_block(&last_clone_ref)?;

            if (*last_block).is_equal(&block) {
                return Err(ErrorBlock::TransactionAlreadyInBlock);
            }

            if last_block.is_previous_of(&block) {

                let node = NodeChain::new(block, last_block_ref.clone())?;
                let nodo_reference = Arc::new(Mutex::new(node));

                self.blocks.push(nodo_reference.clone());

                *last_block_ref = nodo_reference;

                return Ok(());
            }

            let mut previous_node_option = &last_block.previous_node;

            while let Some(previous_node_ref) = previous_node_option {

                let previous_node = Self::get_mut_block(previous_node_ref)?;

                if previous_node.is_equal(&block) {
                    return Err(ErrorBlock::TransactionAlreadyInBlock);
                }

                if previous_node.is_previous_of(&block) {

                    let node = NodeChain::new(block, previous_node_ref.clone())?;
                    let node_reference = Arc::new(Mutex::new(node));
                    self.blocks.push(node_reference.clone());

                    self.last_blocks.push(node_reference);

                    return Ok(());
                }

                *previous_node_option = previous_node.previous_node;

                //let last_block_ref = previous_node_ref;
                //last_block = Self::get_mut_block(last_block_ref)?;
                
            }
        }

        Err(ErrorBlock::CouldNotAppendBlock)
    }

    pub fn update_block(&self, block: Block) -> Result<(), ErrorBlock> {

        for current_block in self.blocks.iter() {

            let mut current_block = Self::get_mut_block(current_block)?;

            if current_block.is_equal(&block) {

                current_block.update_block(block)?;
                
                return Ok(())
            }
        }

        Err(ErrorBlock::CouldNotUpdate)
    }

    pub fn get_block_after_timestamp(&self, timestamp: u32) -> Result<NodeChainRef, ErrorBlock> {
        
        todo!()
    }

    pub fn last(&self) -> Vec<Block> {
        todo!()
    }

    pub fn get_utxo(&self) -> Vec<TransactionOutput> {
        todo!()
    }

    fn get_mut_block(block: &NodeChainRef) -> Result<MutexGuard<NodeChain>, ErrorBlock> {
        match block.lock() {
            Ok(block) => Ok(block),
            _ => Err(ErrorBlock::CouldNotModify),
        }
    }
}
