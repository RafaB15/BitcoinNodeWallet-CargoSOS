use super::outpoint::Outpoint;

#[derive(Debug, Clone)]
pub struct TransactionInput {
    pub previos_output: Outpoint,
    pub signature_script: String,
    pub sequence: u32,
}