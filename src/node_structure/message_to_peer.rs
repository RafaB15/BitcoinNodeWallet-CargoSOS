use super::{connection_id::ConnectionId, message_broadcast::MessageBroadcast};

use crate::{
    block_structure::{block::Block, transaction::Transaction},
    concurrency::work::Work,
};

use std::convert::From;

/// Messages to send to the peer
#[derive(Debug)]
pub enum MessageToPeer {
    SendTransaction(Transaction, Option<ConnectionId>),
    SendBlock(Block, ConnectionId),
    Stop,
}

impl From<MessageToPeer> for Work<MessageBroadcast> {
    fn from(message_to_peer: MessageToPeer) -> Self {
        match message_to_peer {
            MessageToPeer::SendTransaction(transaction, id) => {
                Work::Information(MessageBroadcast::Transaction(transaction, id))
            }
            MessageToPeer::SendBlock(block, id) => {
                Work::Information(MessageBroadcast::Block(block, id))
            }
            MessageToPeer::Stop => Work::Stop,
        }
    }
}
