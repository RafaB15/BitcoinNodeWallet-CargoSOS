use cargosos_bitcoin::{
    block_structure::{block::Block, transaction::Transaction},
    wallet_structure::account::Account,
};

#[derive(Debug, Clone)]
pub enum MessageBroadcasting {
    Block(Block),
    Transaction(Transaction),
    ChangeAccount(Account),
    Exit,
}
