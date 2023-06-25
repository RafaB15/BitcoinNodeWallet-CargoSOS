pub enum SignalToBack{
    GetAccountBalance,
    GetRecentTransactions(String),
    CreateTransaction(String, f64, f64),
    ExitProgram,
    ChangeSelectedAccount(String),
    CreateAccount(String, String, String),
}