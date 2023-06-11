use super::{
    block_chain::BlockChain,
    transaction_output::TransactionOutput,
    block::Block,
    hash::{
        HashType,
        hash256d,
    }, transaction::Transaction
};

use crate::serialization::{
    serializable_internal_order::SerializableInternalOrder,
};

use crate::wallet_structure::{
    address::Address,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UTXOSet {
    pub utxo: Vec<(TransactionOutput, HashType, u32)>,
    pub address: Option<Address>,
}

impl UTXOSet {

    /// Creates a new UTXOSet that can optionally be tied to an account.
    pub fn new(possible_address: Option<Address>) -> UTXOSet {
        UTXOSet {
            utxo: vec![],
            address: possible_address,
        }
    }

    /// Creates a new UTXOSet from a blockchain. If an account is provided, the UTXOSet 
    /// will only contain transactions that belong to the account.
    pub fn from_blockchain(blockchain: &BlockChain, possible_address: Option<Address>) -> UTXOSet {
        let mut utxo_set = UTXOSet::new(possible_address);
        let blocks = blockchain.get_all_blocks();
        for block in blocks {
            utxo_set.update_utxo_with_block(&block);
        }
        utxo_set
    }

    /// Creates a new UTXOSet from an already existing UTXOSet, keeping only the transactions
    /// belonging to the account provided.
    /// The utxo set provided must be up to date.
    pub fn from_utxo_set(utxo_set: &UTXOSet, address: &Address) -> UTXOSet {
        let mut new_utxo_set_list = Vec::new();
        for (output, transaction_hash, index) in utxo_set.utxo.iter() {
            if address.verify_transaction_ownership(output) {
                new_utxo_set_list.push((output.clone(), transaction_hash.clone(), index.clone()));
            }
        }
        UTXOSet {
            utxo: new_utxo_set_list,
            address: Some(address.clone()),
        }
    }

    /// Returns a list of the utxo that have not been spent yet.
    pub fn get_utxo_list(&self) -> Vec<TransactionOutput> {
        self.utxo.iter().map(|(output, _, _)| output.clone()).collect()
    }

    /// Updates the UTXOSet with the transaction outputs of a new block.
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
                if let Some(address) = &self.address {
                    if address.verify_transaction_ownership(output) {
                        self.utxo.push((output.clone(), hashed_transaction, index_utxo as u32));
                        continue;
                    }
                }
                self.utxo.push((output.clone(), hashed_transaction, index_utxo as u32));
            }
        }
    }

    /// Updates the UTXOSet with the transaction inputs of a new block.
    fn update_utxo_with_transaction_input(&mut self, transactions: &Vec<Transaction>) {
        for transaction in transactions {
            for input in &transaction.tx_in {
                for (output, transaction_hash, index) in self.utxo.iter_mut() {
                    if input.previous_output.hash.eq(transaction_hash)
                        && input.previous_output.index == *index
                    {
                        output.value = -1;
                    }
                }
            }
        }
        self.utxo.retain(|(output, _, _)| output.value != -1);
    }

    /// Updates de UTXOSet with the information of a block
    fn update_utxo_with_block(&mut self, block: &Block) {
        self.update_utxo_with_transaction_output(&block.transactions);
        self.update_utxo_with_transaction_input(&block.transactions);
    }

    /// Returns the balance of the UTXOSet in Satoshis.
    pub fn get_balance_in_satoshis(&self) -> i64 {
        let mut balance: i64 = 0;
        for (output, _, _) in self.utxo.iter() {
            balance += output.value;
        }
        balance
    }

    /// Returns the balance of the UTXOSet in TBTC.
    pub fn get_balance_in_tbtc(&self) -> f64 {
        self.get_balance_in_satoshis() as f64 / 100_000_000.0
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::block_structure::{
        block::Block,
        block_header::BlockHeader,
        block_version,
        compact256::Compact256,
        transaction::Transaction,
        transaction_input::TransactionInput,
        transaction_output::TransactionOutput,
        outpoint::Outpoint,
    };
    use crate::messages::compact_size::CompactSize;
    
    #[test]
    fn test_01_correct_utxo_set_creation_with_no_adress() {
        let utxo_set = UTXOSet::new(None);
        assert!(utxo_set.utxo.is_empty());
        assert!(utxo_set.address.is_none());
    }

    #[test]
    fn test_02_correct_utxo_set_creation_with_adress() {
        let address = "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs".to_string();
        let address = Address::new(&address).unwrap();
        let utxo_set = UTXOSet::new(Some(address));
        assert!(utxo_set.utxo.is_empty());
        assert!(utxo_set.address.is_some());
    }

    #[test]
    fn test_03_correct_utxo_set_creation_from_blockchain_with_no_adress() {
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
            Outpoint {
                hash: [1; 32],
                index: 23,
            },
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: "Prueba out".as_bytes().to_vec(),
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        block.append_transaction(transaction).unwrap();

        let blockchain = BlockChain::new(block).unwrap();

        let utxo_set = UTXOSet::from_blockchain(&blockchain, None);
        assert_eq!(utxo_set.utxo.len(), 1);
        assert!(utxo_set.address.is_none());
        assert!(utxo_set.get_balance_in_satoshis() == 10);
    }

    #[test]
    fn test_04_correct_utxo_set_creation_from_utxo_set_with_account_transactions() {
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
            Outpoint {
                hash: [1; 32],
                index: 23,
            },
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6, 0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac]            ,
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        block.append_transaction(transaction).unwrap();

        let blockchain = BlockChain::new(block).unwrap();

        let utxo_set_blockchain = UTXOSet::from_blockchain(&blockchain, None);
        let utxo_set_account = UTXOSet::from_utxo_set(&utxo_set_blockchain, &Address::new(&"mrhW6tcF2LDetj3kJvaDTvatrVxNK64NXk".to_string()).unwrap());
        assert_eq!(utxo_set_account.utxo.len(), 1);
        assert!(utxo_set_account.address.is_some());
        assert!(utxo_set_account.get_balance_in_satoshis() == 10);
    }

    #[test]
    fn test_05_correct_utxo_set_creation_from_utxo_set_without_account_transactions() {
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
            Outpoint {
                hash: [1; 32],
                index: 23,
            },
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6, 0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac],
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        block.append_transaction(transaction).unwrap();

        let blockchain = BlockChain::new(block).unwrap();

        let utxo_set_blockchain = UTXOSet::from_blockchain(&blockchain, None);
        let utxo_set_account = UTXOSet::from_utxo_set(&utxo_set_blockchain, &Address::new(&"mnQLoVaZ3w1NLVmUhfG8hh6WoG3iu7cnNw".to_string()).unwrap());
        assert_eq!(utxo_set_account.utxo.len(), 0);
        assert!(utxo_set_account.address.is_some());
        assert!(utxo_set_account.get_balance_in_satoshis() == 0);
    }

    #[test]
    fn test_06_correct_utxo_set_update_from_block() {

        let mut block_1 = Block::new(BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(0),
        ));

        let transaction_output_1 = TransactionOutput {
            value: 10,
            pk_script: vec![0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6, 0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac],
        };

        let transaction_output_2 = TransactionOutput {
            value: 20,
            pk_script: vec![0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6, 0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac],
        };

        let transaction_output = Transaction {
            version: 1,
            tx_in: vec![],
            tx_out: vec![transaction_output_1.clone(), transaction_output_2.clone()],
            time: 0,
        };

        block_1.append_transaction(transaction_output.clone()).unwrap();

        let blockchain = BlockChain::new(block_1).unwrap();

        let mut utxo_set = UTXOSet::from_blockchain(&blockchain, Some(Address::new(&"mnQLoVaZ3w1NLVmUhfG8hh6WoG3iu7cnNw".to_string()).unwrap()));

        assert_eq!(utxo_set.utxo.len(), 2);
        assert!(utxo_set.address.is_some());
        assert!(utxo_set.get_balance_in_satoshis() == 30);

        let mut serialized_transaction = Vec::new();
        transaction_output
            .io_serialize(&mut serialized_transaction)
            .unwrap();
        let hashed_transaction = hash256d(&serialized_transaction).unwrap();

        let transaction_input_1 = TransactionInput::new(
            Outpoint {
                hash: hashed_transaction,
                index: 0,
            },
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
            [0;32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(0),
        ));

        block_transaction_input.append_transaction(transaction_input).unwrap();

        utxo_set.update_utxo_with_block(&block_transaction_input);

        assert_eq!(utxo_set.utxo.len(), 1);
        assert!(utxo_set.get_balance_in_satoshis() == 20);

    }

    #[test]
    fn test_07_correct_balance_calculation_in_tbtc() {
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
            Outpoint {
                hash: [1; 32],
                index: 23,
            },
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![0x76, 0xa9, 0x14, 0x7a, 0xa8, 0x18, 0x46, 0x85, 0xca, 0x1f, 0x06, 0xf5, 0x43, 0xb6, 0x4a, 0x50, 0x2e, 0xb3, 0xb6, 0x13, 0x5d, 0x67, 0x20, 0x88, 0xac]            ,
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        block.append_transaction(transaction).unwrap();

        let blockchain = BlockChain::new(block).unwrap();

        let utxo_set_blockchain = UTXOSet::from_blockchain(&blockchain, None);
        let utxo_set_account = UTXOSet::from_utxo_set(&utxo_set_blockchain, &Address::new(&"mrhW6tcF2LDetj3kJvaDTvatrVxNK64NXk".to_string()).unwrap());
        assert_eq!(utxo_set_account.get_balance_in_tbtc(),(10 as f64/100_000_000 as f64));
    }

}