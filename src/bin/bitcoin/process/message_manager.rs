use super::{message_broadcasting::MessageBroadcasting, message_notify::MessageNotify};

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

use std::sync::mpsc::{Receiver, Sender};

pub struct MessageManager {
    receiver: Receiver<MessageBroadcasting>,
    sender: Sender<MessageNotify>,
    account: Account,
    transactions: Vec<Transaction>,
    pub block_chain: BlockChain,
    pub utxo_set: UTXOSet,
}

impl MessageManager {
    pub fn new(
        receiver: Receiver<MessageBroadcasting>,
        sender: Sender<MessageNotify>,
        account: Account,
        transactions: Vec<Transaction>,
        block_chain: BlockChain,
        utxo_set: UTXOSet,
    ) -> Self {
        MessageManager {
            receiver,
            sender,
            account,
            transactions,
            block_chain,
            utxo_set,
        }
    }

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

        let balance = self.utxo_set.get_balance_in_tbtc(&self.account.address);
        if self.sender.send(MessageNotify::Balance(balance)).is_err() {
            todo!()
        }

        self.transactions.clear();
    }

    fn receive_transaction(&mut self, transaction: Transaction) {
        if transaction.tx_out.iter().any(|utxo| self.account.verify_transaction_ownership(utxo)) {
            self.transactions.push(transaction);
        }
    }

    fn receive_block(&mut self, block: Block) {
        todo!()
    }
}
