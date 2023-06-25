pub enum SignalToBack{
    GetAccountBalance,
    CreateTransaction(String, f64, f64),
    ExitProgram,
    ChangeSelectedAccount(String),
    CreateAccount(String, String, String),
    GetAccountTransactions,
}