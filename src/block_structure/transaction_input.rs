use super::outpoint::Outpoint;

#[derive(Debug)]
pub struct TransactionInput {
    pub previos_output: Outpoint,
    pub signature_script: String,
    pub sequence: u32,
}