use super::{
    block::Block,
    hash::HashType,
    error_block::ErrorBlock,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};

const NONE_INDEX: u64 = u64::MAX;

#[derive(Debug, Clone)]
pub(super) struct NodeChain {

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
        self.header_hash.eq(&block.header.previous_block_header_hash)
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

impl SerializableInternalOrder for NodeChain {

    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        
        self.block.io_serialize(stream)?;
        self.header_hash.io_serialize(stream)?;
        
        match self.index_previous_node {
            Some(index) => (index as u64).le_serialize(stream)?,
            None => NONE_INDEX.le_serialize(stream)?,
        };
        
        Ok(())
    }
}

impl DeserializableInternalOrder for NodeChain {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        
        Ok(NodeChain {
            block: Block::io_deserialize(stream)?,
            header_hash: HashType::io_deserialize(stream)?,
            index_previous_node: match u64::le_deserialize(stream)? {
                NONE_INDEX => None,
                index => Some(index as usize),
            },
        })

    }
}