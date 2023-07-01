use super::{
    connection_id::ConnectionId, error_node::ErrorNode, message_to_peer::MessageToPeer,
    peer_manager::PeerManager,
};

use crate::{
    block_structure::transaction::Transaction,
    logs::logger_sender::LoggerSender,
    notifications::{notification::Notification, notifier::Notifier},
};

use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

type HandleSender<T> = (JoinHandle<Result<T, ErrorNode>>, Sender<MessageToPeer>);
type SenderReceiver<T> = (Sender<T>, Receiver<T>);

// It represents the broadcasting of the transactions and blocks to the peers
pub struct Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    peers: Vec<HandleSender<(RW, ConnectionId)>>,
    logger: LoggerSender,
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(logger: LoggerSender) -> Self {
        Broadcasting {
            peers: Vec::new(),
            logger,
        }
    }

    /// It adds a connection to a peer to the broadcasting
    pub fn add_connection<N: Notifier>(
        &mut self,
        peer_manager: PeerManager<RW, N>,
        sender_receiver: SenderReceiver<MessageToPeer>,
    ) {
        let handle = thread::spawn(move || peer_manager.connecting_to_peer(sender_receiver.1));
        self.peers.push((handle, sender_receiver.0));
    }

    /// It sends a transaction to all the peers
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to a peer
    pub fn send_transaction(&mut self, transaction: Transaction) -> Result<(), ErrorNode> {
        let transaction_id = match transaction.get_tx_id() {
            Ok(id) => id,
            Err(_) => {
                return Err(ErrorNode::WhileSendingMessage(
                    "Getting transaction id".to_string(),
                ))
            }
        };

        let _ = self
            .logger
            .log_transaction(format!("Broadcasting transaction: {:?}", transaction_id));
        for (_, sender) in self.peers.iter() {
            if sender
                .send(MessageToPeer::SendTransaction(transaction.clone()))
                .is_err()
            {
                return Err(ErrorNode::WhileSendingMessage(
                    "Sending transaction message to peer".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// It stops all the peers and returns their streams
    ///
    /// ### Error
    ///  * `ErrorNode::NodeNotResponding`: It will appear when a thread could not finish
    pub fn destroy<N: Notifier>(self, notifier: N) -> Result<Vec<RW>, ErrorNode> {
        let _ = self.logger.log_configuration("Closing peers".to_string());
        notifier.notify(Notification::ClosingPeers);
        for (_, sender) in self.peers.iter() {
            if sender.send(MessageToPeer::Stop).is_err() {
                return Err(ErrorNode::WhileSendingMessage(
                    "Sending transaction message to peer".to_string(),
                ));
            }
        }

        let mut peers_streams = Vec::new();
        for (handle, _) in self.peers {
            match handle.join() {
                Ok(Ok((peer_stream, _))) => peers_streams.push(peer_stream),
                Ok(Err(error)) => return Err(error),
                Err(_) => {
                    return Err(ErrorNode::NodeNotResponding(
                        "Thread could not finish correctly".to_string(),
                    ))
                }
            }
        }

        Ok(peers_streams)
    }
}
