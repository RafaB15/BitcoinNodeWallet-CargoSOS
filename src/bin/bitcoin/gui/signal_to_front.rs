use cargosos_bitcoin::{
    block_structure::block_chain::BlockChain,
    wallet_structure::wallet::Wallet,
    block_structure::transaction::Transaction,
};

pub enum SignalToFront {
    RegisterWallet(String),
    LoadAvailableBalance((f64, f64)),
    NotifyBlockchainIsReady,
    LoadRecentTransactions(Vec<Transaction>, Wallet),
    ErrorInTransaction(String),
    ErrorInAccountCreation(String),
    TransactionOfAccountReceived(String),
    TransactionInBlock,
    Update,
}