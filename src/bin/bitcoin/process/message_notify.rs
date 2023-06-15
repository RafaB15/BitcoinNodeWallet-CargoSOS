use cargosos_bitcoin::block_structure::transaction::Transaction;

#[derive(Debug, Clone)]
pub enum MessageNotify {
    Balance(u64),
    Transaction(Transaction),
}
