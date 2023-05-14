use super::{
    block_header::BlockHeader, 
    transaction::Transaction,
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
};

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>
}

impl Block {

    pub fn new(header: BlockHeader) -> Self {
        Block { 
            header, 
            transactions: vec![] 
        }
    }

    pub fn proof_of_inclusion(&self) -> bool {
        self.header.proof_of_inclusion(&self.transactions)
    }

    pub fn agregar_transaccion(self, transaction: Transaction) {
        todo!()
    }

    pub fn get_utxo_from_address(&self, address: &str) -> Vec<TransactionOutput> {
        todo!()
    }
}