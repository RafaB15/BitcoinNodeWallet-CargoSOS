use cargosos_bitcoin::block_structure::{block::Block, transaction::Transaction};

use std::cmp::PartialEq;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageResponse {
    Block(Block),
    Transaction(Transaction),
    CreateAccount,
    ChangeAccount,
    RemoveAccount,
    SendTransaction,
    ShowAccounts,
    ShowBalance,
    LastTransactions,
    Exit,
}
