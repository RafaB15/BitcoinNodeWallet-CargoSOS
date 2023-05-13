use super::{
    block_header::BlockHeader, 
    transaction::Transaction,
};

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>
}