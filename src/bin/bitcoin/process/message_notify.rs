use cargosos_bitcoin::block_structure::{block::Block, transaction::Transaction};

#[derive(Debug, Clone)]
pub enum MessageNotify {
    Balance(f64),
    Transaction(Transaction),
    TransactionInBlock((Transaction, Block)),
}
