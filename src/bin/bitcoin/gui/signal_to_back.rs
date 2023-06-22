pub enum SignalToBack{
    GetAccountBalance(String),
    GetRecentTransactions(String),
    CreateTransaction(String, String),
}