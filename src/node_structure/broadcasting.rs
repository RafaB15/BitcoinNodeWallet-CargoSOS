use super::{
    error_node::ErrorNode, message_response::MessageResponse, message_to_peer::MessageToPeer,
    peer_manager::PeerManager,
};

use crate::{
    block_structure::transaction::Transaction,
    configurations::connection_config::ConnectionConfig,
    logs::logger_sender::LoggerSender,
    notifications::{notification::Notification, notifier::Notifier},
};

use std::{
    io::{Read, Write},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

type HandleSender<T> = (JoinHandle<Result<T, ErrorNode>>, Sender<MessageToPeer>);

// It represents the broadcasting of the transactions and blocks to the peers
pub struct Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    peers: Vec<HandleSender<RW>>,
    logger: LoggerSender,
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new<N: Notifier + 'static>(
        peer_streams: Vec<RW>,
        sender_response: Sender<MessageResponse>,
        connection_config: ConnectionConfig,
        notifier: N,
        logger: LoggerSender,
    ) -> Self {
        Broadcasting {
            peers: Self::create_peers(
                peer_streams,
                sender_response,
                connection_config,
                notifier,
                logger.clone(),
            ),
            logger,
        }
    }

    /// It creates a thread for each peer with it's corresponding sender of transactions
    fn create_peers<N: Notifier + 'static>(
        peers_streams: Vec<RW>,
        sender: Sender<MessageResponse>,
        connection_config: ConnectionConfig,
        notifier: N,
        logger: LoggerSender,
    ) -> Vec<HandleSender<RW>> {
        let mut peers: Vec<HandleSender<RW>> = Vec::new();

        for peer_stream in peers_streams {
            let (sender_transaction, receiver_transaction) = mpsc::channel::<MessageToPeer>();

            let peer_manager = PeerManager::new(
                peer_stream,
                sender.clone(),
                receiver_transaction,
                connection_config.magic_numbers,
                notifier.clone(),
                logger.clone(),
            );

            let handle = thread::spawn(move || peer_manager.connecting_to_peer());

            peers.push((handle, sender_transaction));
        }

        peers
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
                Ok(peer_stream) => peers_streams.push(peer_stream?),
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
