use super::{message_broadcasting::MessageBroadcasting, message_notify::MessageNotify};

use cargosos_bitcoin::{
    block_structure::{
        block::Block, transaction::Transaction,
    },
    wallet_structure::account::Account,
};

use std::sync::mpsc::{Receiver, Sender};

pub struct MessageManager {
    receiver: Receiver<MessageBroadcasting>,
    sender_notify: Sender<MessageNotify>,
    account: Account,
}

impl MessageManager {
    pub fn new(
        receiver: Receiver<MessageBroadcasting>,
        sender_notify: Sender<MessageNotify>,
        account: Account,
    ) -> Self {
        MessageManager {
            receiver,
            sender_notify,
            account,
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
    }

    fn receive_transaction(&mut self, transaction: Transaction) {
        if transaction.tx_out
            .iter()
            .any(|utxo| self.account.verify_transaction_ownership(utxo)) 
        {
            todo!()
        } else {
            todo!()
        }
    }

    fn receive_block(&mut self, block: Block) {
        if self.sender_notify.send(MessageNotify::Block(block)).is_err() {
            todo!()
        }
    }
}
