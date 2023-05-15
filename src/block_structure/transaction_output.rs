#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub value: i64,
    pub public_key_script: String,
}


impl TransactionOutput {
    pub fn get_public_key_hash(&self) -> String {
        todo!()
    }
}