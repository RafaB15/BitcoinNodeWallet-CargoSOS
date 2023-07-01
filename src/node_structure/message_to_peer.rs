use crate::{block_structure::transaction::Transaction, concurrency::work::Work};

use std::convert::From;

/// Messages to send to the peer
#[derive(Debug)]
pub enum MessageToPeer {
    SendTransaction(Transaction),
    Stop,
}

impl From<MessageToPeer> for Work<Transaction> {
    fn from(message_to_peer: MessageToPeer) -> Self {
        match message_to_peer {
            MessageToPeer::SendTransaction(transaction) => Work::Information(transaction),
            MessageToPeer::Stop => Work::Stop,
        }
    }
}
