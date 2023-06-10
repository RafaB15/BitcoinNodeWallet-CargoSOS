use super::{
    block_chain::BlockChain,
    transaction::Transaction,
    transaction_output::TransactionOutput,
    transaction_input::TransactionInput,
    hash::{
        HashType,
        hash256d,
    }
};

use crate::wallet_structure::{
    address::Address,
    account::Account,
};
pub struct UTXOSet {
    pub utxo: Vec<(TransactionOutput, HashType, u32)>,
    pub account: Option<Account>,
}

impl UTXOSet {

    pub fn new(possible_account: Option<Account>) -> UTXOSet {
        UTXOSet {
            utxo: vec![],
            account: possible_account,
        }
    }

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
}