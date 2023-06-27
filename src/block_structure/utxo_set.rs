use super::{
    block::Block, block_chain::BlockChain, hash::hash256d, outpoint::Outpoint,
    transaction::Transaction, transaction_output::TransactionOutput,
};

use crate::{
    serialization::serializable_internal_order::SerializableInternalOrder,
    wallet_structure::address::Address,
};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UTXOSet {
    utxo: HashMap<Outpoint, TransactionOutput>,
    pending: Vec<Transaction>,
}

impl UTXOSet {
    /// Creates a new UTXOSet from a vector of blocks
    pub fn new(blocks: Vec<Block>) -> UTXOSet {
        let mut utxo_set = UTXOSet {
            utxo: HashMap::new(),
            pending: Vec::new(),
        };

        blocks
            .iter()
            .for_each(|block| utxo_set.update_utxo_with_block(block));

        utxo_set
    }

    /// Creates a new UTXOSet from a blockchain
    pub fn from_blockchain(blockchain: &BlockChain) -> UTXOSet {
        Self::new(blockchain.get_all_blocks())
    }

    /// Returns a list of the utxo that have not been spent yet
    pub fn get_utxo_list(&self, possible_address: Option<&Address>) -> Vec<TransactionOutput> {
        self.get_utxo_list_with_outpoints(possible_address)
            .iter()
            .map(|(_, transaction_output)| transaction_output.clone())
            .collect()
    }

    /// Get the list of transaction outputs of the given address. In case of not given an address it will get all of them
    pub fn get_utxo_list_with_outpoints(
        &self,
        possible_address: Option<&Address>,
    ) -> Vec<(Outpoint, TransactionOutput)> {
        let mut utxo_without_pending = self.utxo.clone();

        for transaction in self.pending.iter() {
            for input in &transaction.tx_in {
                utxo_without_pending.remove(&input.previous_output);
            }
        }

        utxo_without_pending
            .iter()
            .filter_map(|(outpoint, output)| {
                if let Some(address) = possible_address {
                    match address.verify_transaction_ownership(output) {
                        true => Some((outpoint.clone(), output.clone())),
                        false => None,
                    }
                } else {
                    Some((outpoint.clone(), output.clone()))
                }
            })
            .collect()
    }

    /// Updates the UTXOSet with the transaction outputs of a new block
    fn update_utxo_with_transaction_output(&mut self, transactions: &Vec<Transaction>) {
        for transaction in transactions {
            let mut serialized_transaction: Vec<u8> = Vec::new();
            match transaction.io_serialize(&mut serialized_transaction) {
                Ok(_) => (),
                Err(_) => continue,
            }
            let hashed_transaction = match hash256d(&serialized_transaction) {
                Ok(hashed_transaction) => hashed_transaction,
                Err(_) => continue,
            };

            for (index_utxo, output) in transaction.tx_out.iter().enumerate() {
                let outpoint = Outpoint::new(hashed_transaction, index_utxo as u32);
                self.utxo.insert(outpoint, output.clone());
            }
        }
    }

    /// Updates the UTXOSet with the transaction inputs of a new block
    fn update_utxo_with_transaction_input(&mut self, transactions: &Vec<Transaction>) {
        for transaction in transactions {
            for input in &transaction.tx_in {
                self.utxo.remove(&input.previous_output);
            }
        }
    }

    /// Updates de UTXOSet with the information of a block
    pub fn update_utxo_with_block(&mut self, block: &Block) {
        self.update_utxo_with_transaction_output(&block.transactions);
        self.update_utxo_with_transaction_input(&block.transactions);
        self.pending.clear();
    }

    /// Add a new transaction to the pending transactions removing its influence in the balance
    pub fn append_pending_transaction(&mut self, transaction: Transaction) {
        self.pending.push(transaction);
    }

    /// Returns the balance of the UTXOSet in Satoshis.
    pub fn get_balance_in_satoshis(&self, address: &Address) -> i64 {
        let mut balance: i64 = 0;
        self.get_utxo_list(Some(address))
            .iter()
            .for_each(|output| balance += output.value);
        balance
    }

    /// Returns the balance of the UTXOSet in TBTC.
    pub fn get_balance_in_tbtc(&self, address: &Address) -> f64 {
        self.get_balance_in_satoshis(address) as f64 / 100_000_000.0
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    use crate::block_structure::{
        block::Block, block_header::BlockHeader, block_version, compact256::Compact256,
        outpoint::Outpoint, transaction::Transaction, transaction_input::TransactionInput,
        transaction_output::TransactionOutput,
    };
    
    use crate::messages::compact_size::CompactSize;

    #[test]
    fn test_01_correct_utxo_set_creation_from_utxo_set_with_account_transactions() {
        let mut block = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(1),
        ));

        let transaction_input = TransactionInput::new(
            Outpoint::new([1; 32], 23),
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![
                0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6,
                0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac,
            ],
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        block.append_transaction(transaction).unwrap();

        let blockchain = BlockChain::new(block).unwrap();

        let utxo_set_blockchain = UTXOSet::from_blockchain(&blockchain);
        let address = Address::new(&"mrhW6tcF2LDetj3kJvaDTvatrVxNK64NXk".to_string()).unwrap();
        assert_eq!(utxo_set_blockchain.utxo.len(), 1);
        assert_eq!(utxo_set_blockchain.get_balance_in_satoshis(&address), 10);
    }

    #[test]
    fn test_02_correct_utxo_set_creation_from_utxo_set_without_account_transactions() {
        let mut block = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(1),
        ));

        let transaction_input = TransactionInput::new(
            Outpoint::new([1; 32], 23),
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![
                0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6,
                0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac,
            ],
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        block.append_transaction(transaction).unwrap();

        let blockchain = BlockChain::new(block).unwrap();

        let utxo_set_blockchain = UTXOSet::from_blockchain(&blockchain);
        let address = Address::new(&"mnQLoVaZ3w1NLVmUhfG8hh6WoG3iu7cnNw".to_string()).unwrap();
        assert_eq!(utxo_set_blockchain.utxo.len(), 1);
        assert!(utxo_set_blockchain.get_balance_in_satoshis(&address) == 0);
    }

    #[test]
    fn test_03_correct_utxo_set_update_from_block() {
        let mut block_1 = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(1),
        ));

        let transaction_output_1 = TransactionOutput {
            value: 10,
            pk_script: vec![
                0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6,
                0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac,
            ],
        };

        let transaction_output_2 = TransactionOutput {
            value: 20,
            pk_script: vec![
                0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6,
                0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac,
            ],
        };

        let transaction_output = Transaction {
            version: 1,
            tx_in: vec![],
            tx_out: vec![transaction_output_1.clone(), transaction_output_2.clone()],
            time: 0,
        };

        block_1
            .append_transaction(transaction_output.clone())
            .unwrap();

        let blockchain = BlockChain::new(block_1).unwrap();

        let mut utxo_set = UTXOSet::from_blockchain(&blockchain);
        let address = Address::new(&"mrhW6tcF2LDetj3kJvaDTvatrVxNK64NXk".to_string()).unwrap();

        assert_eq!(utxo_set.utxo.len(), 2);
        assert_eq!(utxo_set.get_balance_in_satoshis(&address), 30);

        let mut serialized_transaction = Vec::new();
        transaction_output
            .io_serialize(&mut serialized_transaction)
            .unwrap();
        let hashed_transaction = hash256d(&serialized_transaction).unwrap();

        let transaction_input_1 = TransactionInput::new(
            Outpoint::new(hashed_transaction, 0),
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_input = Transaction {
            version: 1,
            tx_in: vec![transaction_input_1.clone()],
            tx_out: vec![],
            time: 0,
        };

        let mut block_transaction_input = Block::new(BlockHeader::new(
            block_version::BlockVersion::from(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(1),
        ));

        block_transaction_input
            .append_transaction(transaction_input)
            .unwrap();

        utxo_set.update_utxo_with_block(&block_transaction_input);

        assert_eq!(utxo_set.utxo.len(), 1);
        assert!(utxo_set.get_balance_in_satoshis(&address) == 20);
    }

    #[test]
    fn test_04_correct_balance_calculation_in_tbtc() {
        let mut block = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(0),
        ));

        let transaction_input = TransactionInput::new(
            Outpoint::new([1; 32], 23),
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![
                0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6,
                0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac,
            ],
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        block.append_transaction(transaction).unwrap();

        let blockchain = BlockChain::new(block).unwrap();

        let utxo_set_blockchain = UTXOSet::from_blockchain(&blockchain);
        let address = Address::new(&"mrhW6tcF2LDetj3kJvaDTvatrVxNK64NXk".to_string()).unwrap();
        assert_eq!(
            utxo_set_blockchain.get_balance_in_tbtc(&address),
            (10 as f64 / 100_000_000 as f64)
        );
    }
}
