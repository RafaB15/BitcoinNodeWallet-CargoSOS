use super::{
    block::Block,
    block_header::BlockHeader,
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
};

pub struct BloqueChain {
    next_block: Vec<BloqueChain>,
    block: Block,
}

impl BloqueChain {

    pub fn new(block: Block) -> Self {
        BloqueChain { 
            next_block: vec![],
            block,
        }
    }

    pub fn append_header(&self, header: BlockHeader) -> Result<(), ErrorBlock> {
        todo!()
    }

    pub fn append_block(&self, block: Block) -> Result<(), ErrorBlock> {
        todo!()
    }

    pub fn update_block(&mut self, block: Block) -> Result<(), ErrorBlock> {
        todo!()
    }

    pub fn get_utxo(&self) -> Vec<TransactionOutput> {
        todo!()
    }
}