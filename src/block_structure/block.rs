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

    pub fn remove_spent_transactions(&self, utxo_from_address: &mut Vec<TransactionOutput>) {
        todo!();
    }

    pub fn update_utxo_from_address(&self, address: &str, utxo_from_address: &mut Vec<TransactionOutput>) {
        //Vamos a chequear toas los output transactions y las que tengan el address las agregamos, luego chequeamos 
        //si en los inputs alguna gasta alguno de los outputs.
        for transaction in &self.transactions {
            self.remove_spent_transactions(utxo_from_address);
            for output in &transaction.tx_out {
                if output.get_public_key_hash() == address {
                    utxo_from_address.push(output.clone());
                }
            }
        }
    }
}