use super::{
    block::Block,
    hash::HashType,
    error_block::ErrorBlock,
};

#[derive(Debug, Clone)]
pub struct NodeChain {

    pub block: Block,
    pub header_hash: HashType,

    pub index_previous_node: Option<usize>,
}

impl NodeChain {

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

    pub fn is_previous_of(&self, block: &Block) -> bool {
        
        let previous_hash = match block.header.get_hash256d() {
            Ok(hash) => hash,
            _ => return false,
        };

        self.header_hash.eq(&previous_hash)
    }

    pub fn is_equal(&self, block: &Block) -> bool {

        let (given_hash, hash) = match (
            block.header.get_hash256d(), 
            self.block.header.get_hash256d()
        ) {
            (Ok(given_hash), Ok(hash)) => (given_hash, hash),
            _ => return false,
        };

        given_hash.eq(&hash)
    }

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