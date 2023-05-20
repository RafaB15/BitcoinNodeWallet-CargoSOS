use super::{
    error_block::ErrorBlock,
    hash::{hash256, HashType, hash256d},
    //transaction_input::TransactionInput,
    //transaction_output::TransactionOutput, transaction,
    transaction::Transaction,
};

use std::f64;

pub struct MerkleTree {
    hashes: Vec<HashType>,
    initial_count: usize,
}

impl MerkleTree {

    fn is_power_of_two(number: u32) -> bool {
        let log_result = (number as f64).log2();
        log_result.fract() == 0.0
    }

    pub fn new(transactions: &[Transaction]) -> Result<MerkleTree, ErrorBlock> {
        //chequeo que sea base de 2, si lo es no hago nada, sino -> aplico log_2(transactions.len) ^ 2 = initial_count
        let mut initial_count = match Self::is_power_of_two(transactions.len() as u32) {
            true => transactions.len(),
            false => {
                let log_result = (transactions.len() as f64).log2();
                (log_result as usize).pow(2)
            },
        };

        let mut tx_ids: Vec<HashType> = Transaction::get_vec_txids(transactions)?;

        let mut hashes: Vec<HashType> = Vec::new();

        while initial_count > 1 {
            if tx_ids.len() % 2 == 1 {
                if let Some(last_hash) = tx_ids.last() {
                    tx_ids.push(*last_hash);
                }
            }

            for (i, combined) in tx_ids.iter().enumerate().step_by(2) {
                // Concatenar dos hashes
                let mut combined = combined.to_vec();
                match hashes.get(i + 1) {
                    Some(combined_next) => combined.extend_from_slice(combined_next),
                    None => return Err(ErrorBlock::CouldNotWriteTxId),
                };

                // Calcular el hash combinado
                let combined_hash = match hash256d(&combined) {
                    Ok(combined_hash) => combined_hash,
                    Err(_) => return Err(ErrorBlock::CouldNotWriteTxId),
                };
                tx_ids.push(combined_hash);
            }
        }

        Ok(MerkleTree {
            hashes: tx_ids,
            initial_count: initial_count,
        })
    }

    pub fn get_root(&self) -> Result<HashType, ErrorBlock> {

        let mut hashes: Vec<HashType> = self.hashes.clone();
        let root: HashType = match hashes.last() {
            Some(root) => *root,
            None => return Err(ErrorBlock::NoTransactions),
        };
        Ok(root)
    }


    pub fn get_merkle_path(transactions: &[Transaction], target_transaction: Transaction) -> Result<Vec<HashType>,ErrorBlock> {
        let mut merkle_path: Vec<HashType> = Vec::new();
        let merkle_tree = MerkleTree::new(&transactions)?;


        // Find the target transaction index in the block
        let target_index = transactions.iter().position(|tx| *tx == target_transaction);
        if target_index.is_none() {
            return Err(ErrorBlock::TransactionNotFound);
        } else {};
        todo!()
    }
}





