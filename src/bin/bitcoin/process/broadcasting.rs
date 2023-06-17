use super::{
    error_process::ErrorProcess, message_broadcasting::MessageBroadcasting,
    peer_manager::PeerManager,
};

use cargosos_bitcoin::block_structure::transaction::Transaction;

use std::{
    io::{Read, Write},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

type HandleSender<T> = (JoinHandle<T>, Sender<MessageBroadcasting>);

pub struct Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    peers: Vec<HandleSender<Result<RW, ErrorProcess>>>,
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(peer_streams: Vec<RW>, sender_broadcasting: Sender<MessageBroadcasting>) -> Self {
        Broadcasting {
            peers: Self::create_peers(peer_streams, sender_broadcasting),
        }
    }

    fn create_peers(
        peers_streams: Vec<RW>,
        sender: Sender<MessageBroadcasting>,
    ) -> Vec<HandleSender<Result<RW, ErrorProcess>>> {
        let mut peers: Vec<HandleSender<Result<RW, ErrorProcess>>> = Vec::new();

        for peer_stream in peers_streams {
            let sender_clone = sender.clone();
            let (sender_message, receiver_message) = mpsc::channel::<MessageBroadcasting>();

            let handle = thread::spawn(move || {
                let peer_manager = PeerManager::new(peer_stream, sender_clone, receiver_message);

                peer_manager.listen_peers()
            });

            peers.push((handle, sender_message));
        }

        peers
    }

    pub fn send_transaction(&mut self, transaction: Transaction) {
        for (_, sender) in self.peers.iter() {
            if sender
                .send(MessageBroadcasting::Transaction(transaction.clone()))
                .is_err()
            {
                todo!()
            }
        }
    }

    pub fn destroy(self) -> Result<Vec<RW>, ErrorProcess> {
        for (_, sender) in self.peers.iter() {
            if sender.send(MessageBroadcasting::Exit).is_err() {
                todo!()
            }
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
