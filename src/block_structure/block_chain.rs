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

    pub fn update_utxo_from_address_in_block(&self, address: &str, utxo_from_address: &mut Vec<TransactionOutput>) {
        self.block.update_utxo_from_address(address, utxo_from_address);
        match self.next_block.iter().next() {
            Some(next_block) => next_block.update_utxo_from_address_in_block(address, utxo_from_address),
            None => (),
        }
    }

    pub fn get_utxo_from_address(&self, address: &str) -> Vec<TransactionOutput> {
        let mut utxo_from_address = vec![];
        self.update_utxo_from_address_in_block(address, &mut utxo_from_address);
        utxo_from_address
    }
}