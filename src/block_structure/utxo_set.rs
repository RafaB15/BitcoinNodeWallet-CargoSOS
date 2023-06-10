use super::{
    block_chain::BlockChain,
    transaction_output::TransactionOutput,
    hash::{
        HashType,
    }
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
        let mut utxo: Vec<(TransactionOutput, HashType, u32)> = vec![];
        for node_chain in blockchain.blocks.iter() {
            node_chain.block.update_utxo_list(&mut utxo, &possible_account);
        }
        UTXOSet {
            utxo,
            account: possible_account, 
        }
    }

    /// Creates a new UTXOSet from an already existing UTXOSet, keeping only the transactions
    /// belonging to the account provided.
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
}