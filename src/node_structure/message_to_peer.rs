use crate::{block_structure::transaction::Transaction, concurrency::work::Work};

use std::convert::From;

#[derive(Debug)]
pub enum MessageToPeer {
    SendTransaction(Transaction),
    Stop,
}

impl From<MessageToPeer> for Work {
    fn from(message_to_peer: MessageToPeer) -> Self {
        match message_to_peer {
            MessageToPeer::SendTransaction(transaction) => Work::SendTransaction(transaction),
            MessageToPeer::Stop => Work::Stop,
        }
    }
}
