use super::{
    error_block::ErrorBlock,
    hash::{hash256d, HashType},
    transaction::Transaction,
};

use std::f64;

pub struct MerkleTree {
    pub hashes: Vec<HashType>,
    pub initial_count: usize,
}

impl MerkleTree {
    ///Creates a new Merkle Tree from a list of transactions
    ///
    ///
    /// ### Errors
    ///  * `CouldNotWriteTxId` - If the transaction id could not be written
    ///  * `CouldNotGetVecTxIds` - If the transaction id vector could not be obtained
    pub fn new(transactions: &[Transaction]) -> Result<MerkleTree, ErrorBlock> {
        //chequeo que sea base de 2, si lo es no hago nada, sino -> aplico 2 ^ log_2(transactions.len) = initial_count

        let log_result = (transactions.len() as f64).log2();
        let levels = log_result.ceil() as u32;
        let initial_count = (2_usize).pow(levels);

        //println!("Initial len {}\nInitial count: {}", transactions.len(), initial_count);

        let mut tx_ids: Vec<HashType> = Transaction::get_vec_txids(transactions)?;
        let last_tx: HashType = match tx_ids.last() {
            Some(last_tx) => *last_tx,
            None => return Err(ErrorBlock::CouldNotWriteTxId),
        };

        while tx_ids.len() < initial_count {
            tx_ids.push(last_tx);
        }

        let mut hashes: Vec<HashType> = tx_ids.clone();

        for _ in 0..levels {
            let mut tx_ids_combined: Vec<HashType> = Vec::new();
            for (i, combined) in tx_ids.iter().enumerate().step_by(2) {
                // Concatenar dos hashes
                let mut combined = combined.to_vec();
                match tx_ids.get(i + 1) {
                    Some(combined_next) => combined.extend_from_slice(combined_next),
                    None => return Err(ErrorBlock::CouldNotWriteTxId),
                };

                // Calcular el hash combinado
                let combined_hash = match hash256d(&combined) {
                    Ok(combined_hash) => combined_hash,
                    Err(_) => return Err(ErrorBlock::CouldNotWriteTxId),
                };

                tx_ids_combined.push(combined_hash);
            }

            tx_ids = tx_ids_combined;
            hashes.extend_from_slice(&tx_ids);
        }

        Ok(MerkleTree {
            hashes,
            initial_count,
        })
    }

    /// Returns the root of the Merkle Tree
    /// It will be at the first position of the vector
    ///
    /// ### Errors
    ///    * `RootHashNotFound` - If the root hash could not be found
    pub fn get_root(&self) -> Result<HashType, ErrorBlock> {
        let hashes: Vec<HashType> = self.hashes.clone();
        let root: HashType = match hashes.last() {
            Some(root) => *root,
            None => return Err(ErrorBlock::RootHashNotFound),
        };
        Ok(root)
    }

    /// Returns the hash at the given index
    ///
    /// ### Errors
    ///   * `NoHashFound` - If the hash could not be found
    pub fn get_hash_at(&self, index: usize) -> Result<HashType, ErrorBlock> {
        let hashes: Vec<HashType> = self.hashes.clone();
        let hash: HashType = match hashes.get(index) {
            Some(hash) => *hash,
            None => return Err(ErrorBlock::NoHashFound),
        };
        Ok(hash)
    }

    /// Returns the merkle path of the given transaction
    ///
    /// ### Errors
    ///  * `TransactionNotFound` - If the transaction could not be found
    ///  * `NoHashFound` - If the hash could not be found
    ///  * `CouldNotWriteTxId` - If the transaction id could not be written (while creating the merkle tree)
    pub fn get_merkle_path(
        transactions: &[Transaction],
        target_transaction: Transaction,
    ) -> Result<Vec<HashType>, ErrorBlock> {
        let merkle_tree = MerkleTree::new(transactions)?;
        let mut size = merkle_tree.initial_count;

        // Find the target transaction index in the block
        let mut target_index = match transactions
            .iter()
            .position(|transaction| *transaction == target_transaction)
        {
            Some(target_index) => target_index,
            None => return Err(ErrorBlock::TransactionNotFound),
        };

        let mut merkle_path: Vec<HashType> = Vec::new();

        while size > 1 {
            let sibling_index = match target_index % 2 == 0 {
                true => target_index + 1,
                false => target_index - 1,
            };

            merkle_path.push(merkle_tree.get_hash_at(sibling_index)?);

            target_index = size + target_index / 2;
            size /= 2;
        }

        Ok(merkle_path)
    }
}
