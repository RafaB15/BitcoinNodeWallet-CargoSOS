use super::{
    error_process::ErrorProcess, message_response::MessageResponse, peer_manager::PeerManager,
};

use cargosos_bitcoin::block_structure::transaction::Transaction;

use std::{
    io::{Read, Write},
    sync::mpsc::{self, Sender},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

type HandleSender<T> = (JoinHandle<Result<T, ErrorProcess>>, Sender<Transaction>);

pub struct Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    peers: Vec<HandleSender<RW>>,
    stop: Arc<Mutex<bool>>,
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(peer_streams: Vec<RW>, sender_response: Sender<MessageResponse>) -> Self {
        let stop = Arc::new(Mutex::new(false));

        Broadcasting {
            peers: Self::create_peers(peer_streams, sender_response, stop.clone()),
            stop,
        }
    }

    fn create_peers(
        peers_streams: Vec<RW>,
        sender: Sender<MessageResponse>,
        stop: Arc<Mutex<bool>>,
    ) -> Vec<HandleSender<RW>> {
        let mut peers: Vec<HandleSender<RW>> = Vec::new();

        for peer_stream in peers_streams {
            let sender_clone = sender.clone();
            let (sender_transaction, receiver_transaction) = mpsc::channel::<Transaction>();
            let stop_clone = stop.clone();

            let handle = thread::spawn(move || {
                let peer_manager =
                    PeerManager::new(peer_stream, sender_clone, receiver_transaction, stop_clone);

                peer_manager.listen_peers()
            });

            peers.push((handle, sender_transaction));
        }

        peers
    }

    pub fn send_transaction(&mut self, transaction: Transaction) {
        for (_, sender) in self.peers.iter() {
            if sender.send(transaction.clone()).is_err() {
                todo!()
            }
        }
    }

    pub fn destroy(self) -> Result<Vec<RW>, ErrorProcess> {
        match self.stop.lock() {
            Ok(mut stop) => *stop = true,
            Err(_) => todo!(),
        }

        let mut peers_streams = Vec::new();
        for (handle, _) in self.peers {
            match handle.join() {
                Ok(peer_stream) => peers_streams.push(peer_stream?),
                Err(_) => todo!(),
            }
        }

        Ok(peers_streams)
    }
}
