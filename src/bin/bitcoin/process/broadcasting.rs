use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, transaction::Transaction},
    wallet_structure::account::Account,
};

use std::{
    io::{Read, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct Broadcasting {
    peers: Vec<JoinHandle>,
    received: JoinHandle<BlockChain>,
}

impl<RW> Broadcasting
where
    RW: Read + Write,
{
    fn new(account: Account, peers: Vec<RW>) -> Self {}

    fn change_account(&mut self, account: Account) -> Result<()> {}
}

pub struct MessageManager {
    receiver: Receiver,
    account: Account,
    transactions: Vec<Transaction>,
    block_chain: BlockChain,
}
