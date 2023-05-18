use super::{
    block::Block,
    block_chain::NodeChainRef, 
    hash::HashType,
    error_block::ErrorBlock,
};



use std::{sync::Arc};

#[derive(Debug, Clone)]
pub struct NodeChain {

    pub previous_node: Option<NodeChainRef>,

    pub block: Block,
    pub header_hash: HashType,
}

impl NodeChain {

    pub fn first(block: Block) -> Result<Self, ErrorBlock> {

        let header_hash = match block.header.get_hash256d() {
            Ok(hash) => hash,
            _ => return Err(ErrorBlock::CouldNotHash),
        };

        Ok(NodeChain { 
            previous_node: None, 
            header_hash, 
            block, 
        })
    }

    pub fn new(block: Block, previous_node: NodeChainRef) -> Result<Self, ErrorBlock> {

        let header_hash = match block.header.get_hash256d() {
            Ok(hash) => hash,
            _ => return Err(ErrorBlock::CouldNotHash),
        };

        Ok(NodeChain { 
            previous_node: Some(previous_node), 
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

        let given_hash = match block.header.get_hash256d() {
            Ok(hash) => hash,
            _ => return false,
        };

        let hash = match self.block.header.get_hash256d() {
            Ok(hash) => hash,
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