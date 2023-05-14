use super::{
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub version: i32,
    pub tx_in: Vec<TransactionInput>,
    pub tx_out: Vec<TransactionOutput>,
    pub time: u32,
}