/// This enum represents the signal that the front sends to the back.
pub enum SignalToBack {
    /// Signal to obtain the balance of an account.
    GetAccountBalance,

    /// Signal to create a transaction.
    CreateTransaction(String, f64, f64),

    /// Signal to change the selected account.
    ChangeSelectedAccount(String),

    /// Signal to create an account.
    CreateAccount(String, String, String),

    /// Signal to get the transactions of an account.
    GetAccountTransactions,

    /// Signal requesting the merkle proof of a transaction.
    RequestMerkleProof(String, String),

    /// Signal to exit the program.
    ExitProgram,
}
