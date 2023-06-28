use std::net::SocketAddr;

use crate::{
    block_structure::{block::Block, transaction::Transaction},
    wallet_structure::account::Account,
};

use std::sync::mpsc::{Receiver, Sender};

pub type NotificationSender = Sender<Notification>;
pub type NotificationReceiver = Receiver<Notification>;

pub enum Notification {
    AttemptingHandshakeWithPeer(SocketAddr),
    SuccessfulHandshakeWithPeer(SocketAddr),
    FailedHandshakeWithPeer(SocketAddr),
    TransactionOfAccountReceived(Vec<Account>, Transaction),
    TransactionOfAccountInNewBlock(Transaction),
    NewBlockAddedToTheBlockchain(Block),
}
