use super::{error_node::ErrorNode, message_response::MessageResponse, peer_manager::PeerManager};

use crate::{
    block_structure::transaction::Transaction, configurations::connection_config::ConnectionConfig,
    logs::logger_sender::LoggerSender,
};

use std::{
    io::{Read, Write},
    sync::mpsc::{self, Sender},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

type HandleSender<T> = (JoinHandle<Result<T, ErrorNode>>, Sender<Transaction>);

// It represents the broadcasting of the transactions and blocks to the peers
pub struct Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    peers: Vec<HandleSender<RW>>,
    stop: Arc<Mutex<bool>>,
    logger: LoggerSender,
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(
        peer_streams: Vec<RW>,
        sender_response: Sender<MessageResponse>,
        connection_config: ConnectionConfig,
        logger: LoggerSender,
    ) -> Self {
        let stop = Arc::new(Mutex::new(false));

        Broadcasting {
            peers: Self::create_peers(
                peer_streams,
                sender_response,
                stop.clone(),
                connection_config,
                logger.clone(),
            ),
            stop,
            logger,
        }
    }

    /// It creates a thread for each peer with it's corresponding sender of transactions
    fn create_peers(
        peers_streams: Vec<RW>,
        sender: Sender<MessageResponse>,
        stop: Arc<Mutex<bool>>,
        connection_config: ConnectionConfig,
        logger: LoggerSender,
    ) -> Vec<HandleSender<RW>> {
        let mut peers: Vec<HandleSender<RW>> = Vec::new();

        for peer_stream in peers_streams {
            let (sender_transaction, receiver_transaction) = mpsc::channel::<Transaction>();

            let peer_manager = PeerManager::new(
                peer_stream,
                sender.clone(),
                receiver_transaction,
                stop.clone(),
                connection_config.magic_numbers,
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
            .log_transaction(format!(
                "Broadcasting transaction: {:?}",
                transaction_id
            ));
        for (_, sender) in self.peers.iter() {
            if sender.send(transaction.clone()).is_err() {
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
    pub fn destroy(self) -> Result<Vec<RW>, ErrorNode> {
        let _ = self.logger.log_configuration("Closing peers".to_string());
        match self.stop.lock() {
            Ok(mut stop) => *stop = true,
            Err(_) => {
                return Err(ErrorNode::NodeNotResponding(
                    "Thread could not stop peers".to_string(),
                ))
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
