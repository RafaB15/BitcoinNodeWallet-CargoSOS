use super::{
    error_block::ErrorBlock,
    hash::{hash256d, HashType},
    transaction::Transaction,
};

pub struct MerkleTree {
    pub root: HashType,
    pub hashes: Vec<Vec<HashType>>,
    pub initial_count: usize,
}

impl MerkleTree {
    /// Creates a new Merkle Tree from a list of transactions
    ///
    /// ### Errors
    ///  * `ErrorBlock::CouldNotWriteTxId`: It will appear when the transaction id could not be written
    ///  * `ErrorBlock::CouldNotGetVecTxIds`: It will appear when the transaction id could not be created
    pub fn new(transactions: &[Transaction]) -> Result<MerkleTree, ErrorBlock> {

        let mut current_level: Vec<HashType> = Transaction::get_vec_txids(transactions)?;
        MerkleTree::make_valid_level(&mut current_level);

        let mut current_lenght = current_level.len();

        let mut levels: Vec<Vec<HashType>> = vec![current_level.clone()];

        while current_lenght != 1 {
            let mut next_level: Vec<HashType> = Vec::new();
            for (i, combined) in current_level.iter().enumerate().step_by(2) {
                // Concatenar dos hashes
                let mut combined = combined.to_vec();
                match current_level.get(i + 1) {
                    Some(combined_next) => combined.extend_from_slice(combined_next),
                    None => return Err(ErrorBlock::CouldNotWriteTxId("Could not get next tx".to_string())),
                };

                // Calcular el hash combinado
                let combined_hash = match hash256d(&combined) {
                    Ok(combined_hash) => combined_hash,
                    Err(_) => return Err(ErrorBlock::CouldNotWriteTxId("Could not get combined hash".to_string())),
                };

                next_level.push(combined_hash);
            }
            MerkleTree::make_valid_level(&mut next_level);
            current_lenght = next_level.len();
            current_level = next_level;
            levels.push(current_level.clone());
        }

        let root = match current_level.first() {
            Some(root) => root.clone(),
            None => return Err(ErrorBlock::CouldNotWriteTxId("Could not get root".to_string())),
        };

        Ok(MerkleTree {
            root,
            hashes: levels,
            initial_count: 0,
        })
    }

    fn make_valid_level(transaction_hashes: &mut Vec<HashType>) {
        if (transaction_hashes.len() % 2 != 0) && (transaction_hashes.len() != 1)  {
            let last_element = match transaction_hashes.last() {
                Some(last_element) => last_element.clone(),
                None => return,
            };
            transaction_hashes.push(last_element);
        }
    }

    /// Returns the root of the Merkle Tree
    /// It will be at the first position of the vector
    ///
    /// ### Errors
    ///    * `ErrorBlock::RootHashNotFound`: It will appear when the root hash in merkle tree could not be found
    pub fn get_root(&self) -> HashType {
        self.root
    }
    /*
    /// Returns the hash at the given index
    ///
    /// ### Errors
    ///   * `ErrorBlock::NoHashFound`: It will appear when the hash in merkle tree could not be found at given index
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
    ///  * `ErrorBlock::TransactionNotFound`: It will appear when the transaction could not be found
    ///  * `ErrorBlock::NoHashFound`: It will appear when the hash in merkle tree could not be found at given index
    ///  * `ErrorBlock::CouldNotWriteTxId`: It will appear when the transaction id could not be written (while creating the merkle tree)
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
    }*/
}
