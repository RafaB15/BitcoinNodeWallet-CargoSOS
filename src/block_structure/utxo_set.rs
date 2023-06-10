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
    account::Account,
};
pub struct UTXOSet {
    pub utxo: Vec<(TransactionOutput, HashType, u32)>,
    pub account: Option<Account>,
}

impl UTXOSet {

    /// Creates a new UTXOSet that can optionally be tied to an account.
    pub fn new(possible_account: Option<Account>) -> UTXOSet {
        UTXOSet {
            utxo: vec![],
            account: possible_account,
        }
    }

    /// Creates a new UTXOSet from a blockchain. If an account is provided, the UTXOSet 
    /// will only contain transactions that belong to the account.
    pub fn from_blockchain(blockchain: &BlockChain, possible_account: Option<Account>) -> UTXOSet {
        let mut utxo_set = UTXOSet::new(possible_account);
        for node_chain in blockchain.blocks.iter() {
            utxo_set.update_utxo_with_block(&node_chain.block);
        }
        utxo_set
    }

    /// Creates a new UTXOSet from an already existing UTXOSet, keeping only the transactions
    /// belonging to the account provided.
    /// The utxo set provided must be up to date.
    pub fn from_utxo_set(utxo_set: &UTXOSet, account: Account) -> UTXOSet {
        let mut new_utxo_set_list = Vec::new();
        for (output, transaction_hash, index) in utxo_set.utxo.iter() {
            if account.verify_transaction_ownership(output) {
                new_utxo_set_list.push((output.clone(), transaction_hash.clone(), index.clone()));
            }
        }
        UTXOSet {
            utxo: new_utxo_set_list,
            account: Some(account),
        }
    }

    /// Returns a list of the utxo that have not been spent yet.
    pub fn get_utxo_list(&self) -> Vec<TransactionOutput> {
        self.utxo.iter().map(|(output, _, _)| output.clone()).collect()
    }

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
                if let Some(account) = &self.account {
                    if account.verify_transaction_ownership(output) {
                        self.utxo.push((output.clone(), hashed_transaction, index_utxo as u32));
                        continue;
                    }
                }
                self.utxo.push((output.clone(), hashed_transaction, index_utxo as u32));
            }
        }
    }

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
}