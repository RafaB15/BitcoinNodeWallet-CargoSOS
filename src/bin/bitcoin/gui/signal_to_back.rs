pub enum SignalToBack{
    GetAccountBalance,
    GetRecentTransactions(String),
    CreateTransaction(String, String),
    ExitProgram,
    ChangeSelectedAccount(String),
}