use std::net::SocketAddr;

use crate::{
    block_structure::{block::Block, transaction::Transaction},
    wallet_structure::account::Account,
};

pub type NotificationSender = std::sync::mpsc::Sender<Notification>;
pub type NotificationReceiver = std::sync::mpsc::Receiver<Notification>;

pub enum Notification {
    AttemptingHandshakeWithPeer(SocketAddr),
    SuccessfulHandshakeWithPeer(SocketAddr),
    FailedHandshakeWithPeer(SocketAddr),
    TransactionOfAccountReceived(Vec<Account>, Transaction),
    TransactionOfAccountInNewBlock(Transaction),
    NewBlockAddedToTheBlockchain(Block),
}
