use cargosos_bitcoin::block_structure::{block::Block, transaction::Transaction};

#[derive(Debug, Clone)]
pub enum MessageNotify {
    TransactionFromAccount(Transaction),
    Transaction(Transaction),
    Block(Block),
}
