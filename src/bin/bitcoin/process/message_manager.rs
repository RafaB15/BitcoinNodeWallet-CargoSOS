use super::message_broadcasting::MessageBroadcasting;

use cargosos_bitcoin::{
    block_structure::{
        block::Block, block_chain::BlockChain, transaction::Transaction, utxo_set::UTXOSet,
    },
    messages::{
        block_message::BlockMessage, command_name::CommandName, get_data_message::GetDataMessage,
        message, message_header::MessageHeader,
    },
    wallet_structure::account::Account,
};

use std::{
    io::{Read, Write},
    mem::replace,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

type ReadWrite = Read + Write + Send + 'static;

pub struct MessageManager {
    receiver: Receiver<Message>,
    account: Account,
    transactions: Vec<Transaction>,
    pub block_chain: BlockChain,
    pub utxo_set: UTXOSet,
}

impl MessageManager {
    pub fn receive_messages(mut self) -> Self {
        while let Ok(message) = self.receiver.recv() {
            match message {
                MessageBroadcasting::Transaction(transaction) => {
                    self.receive_transaction(transaction)
                }
                MessageBroadcasting::Block(block) => self.receive_block(block),
                MessageBroadcasting::ChangeAccount(account) => self.change_account(account),
                MessageBroadcasting::Exit => break,
            }
        }

        self
    }

    fn change_account(&mut self, account: Account) {
        self.account = account;
    }

    fn receive_transaction(&mut self, transaction: Transaction) {
        todo!()
    }

    fn receive_block(&mut self, block: Block) {
        todo!()
    }
}
