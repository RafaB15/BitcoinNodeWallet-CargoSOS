/// This enum represents the signal that the back sends to the front.
pub enum SignalToFront {
    /// Signal to add an account to the list of accounts.
    RegisterAccount(String),

    /// Signal to update the available balance.
    LoadAvailableBalance((f64, f64)),

    /// Signal to notify that the blockchain is ready.
    NotifyBlockchainIsReady,

    /// Signal to notify that an error involving an account occurred.
    ErrorInTransaction(String),

    /// Signal to notify that an error ocurred while creating an account.
    ErrorInAccountCreation(String),

    /// Signal to notify that we received a transaction from one of our accounts.
    TransactionOfAccountReceived(String),

    /// Signal to notify that we received a transaction from one of our accounts in a block.
    BlockWithUnconfirmedTransactionReceived,

    /// Signal to transmit the information of the transactions of an account.
    AccountTransactions(Vec<(u32, [u8; 32], i64)>),

    /// Signal to notify that we have to update the front.
    Update,
}
