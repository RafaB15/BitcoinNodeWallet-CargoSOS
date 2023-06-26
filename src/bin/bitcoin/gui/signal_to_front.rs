pub enum SignalToFront {
    RegisterWallet(String),
    LoadAvailableBalance((f64, f64)),
    NotifyBlockchainIsReady,
    ErrorInTransaction(String),
    ErrorInAccountCreation(String),
    TransactionOfAccountReceived(String),
    BlockWithUnconfirmedTransactionReceived,
    AccountTransactions(Vec<(u32, [u8; 32], i64)>),
    Update,
}
