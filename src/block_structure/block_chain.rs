use super::{
    block::Block,
    block_header::BlockHeader,
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
};

pub struct BlockChain {
    next_block: Vec<BlockChain>,
    block: Block,   
}

impl BlockChain {

    pub fn new(block: Block) -> Self {
        BlockChain { 
            next_block: vec![],
            block,
        }
    }

    pub fn append_header(&self, header: BlockHeader) -> Result<(), ErrorBlock> {
        self.append_block(Block::new(header))
    }

    pub fn append_block(&self, block: Block) -> Result<(), ErrorBlock> {
        todo!()
    }

    pub fn update_block(&mut self, block: Block) -> Result<(), ErrorBlock> {
        todo!()
    }

    pub fn last<'b>(&self) -> &'b Block {
        todo!()
    }

    pub fn get_utxo_from_address(&self, address: &str) -> Vec<TransactionOutput> {
        let mut utxo_from_address = self.block.get_utxo_from_address(address);
        if !self.next_block.is_empty() {
            for block in self.next_block.iter() {
                utxo_from_address.extend(block.get_utxo_from_address(address));
            }
        }
        return utxo_from_address;
        
    }
}