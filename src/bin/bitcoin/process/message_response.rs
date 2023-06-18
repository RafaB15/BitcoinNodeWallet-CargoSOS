use cargosos_bitcoin::block_structure::{block::Block, transaction::Transaction};

#[derive(Debug, Clone)]
pub enum MessageResponse {
    Block(Block),
    Transaction(Transaction),
    CreateAccount,
    ChangeAccount,
    SendTransaction,
    ShowAccounts,
    ShowBalance,
    LastTransactions,
    Exit,
}
