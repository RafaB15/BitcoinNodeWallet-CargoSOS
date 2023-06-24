use crate::block_structure::{block::Block, transaction::Transaction};

use std::cmp::PartialEq;

/// It represents the posible responses from a peer
#[derive(Debug, Clone, PartialEq)]
pub enum MessageResponse {
    Block(Block),
    Transaction(Transaction),
}