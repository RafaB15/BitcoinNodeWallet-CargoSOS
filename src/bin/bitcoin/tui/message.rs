use cargosos_bitcoin::block_structure::{block::Block, transaction::Transaction};

#[derive(Debug, Clone)]
pub enum Message {
    Block(Block),
    Transaction(Transaction),
    Exit,
}
