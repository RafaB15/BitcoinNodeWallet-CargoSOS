use super::{
    message_broadcasting::MessageBroadcasting, message_manager::MessageManager,
    peer_manager::PeerManager,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, transaction::Transaction, utxo_set::UTXOSet},
    messages::{
        block_message::BlockMessage, command_name::CommandName, get_data_message::GetDataMessage,
        message, message_header::MessageHeader,
    },
    wallet_structure::account::Account,
};

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
    peers: Vec<HandleSender<RW>>,
    receiver: HandleSender<MessageManager>,
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(
        account: Account,
        peer_streams: Vec<RW>,
        block_chain: BlockChain,
        utxo_set: UTXOSet,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<MessageBroadcasting>();

        let message_manager =
            MessageManager::new(receiver, account, Vec::new(), block_chain, utxo_set);

        Broadcasting {
            peers: Self::create_peers(peer_streams, sender.clone()),
            receiver: (Self::create_receiver(message_manager), sender),
        }
    }

    fn create_receiver(mut message_manager: MessageManager) -> JoinHandle<MessageManager> {
        thread::spawn(move || message_manager.receive_messages())
    }

    fn create_peers(
        peers_streams: Vec<RW>,
        sender: Sender<MessageBroadcasting>,
    ) -> Vec<(JoinHandle<RW>, Sender<MessageBroadcasting>)> {
        let mut peers: Vec<HandleSender<RW>> = Vec::new();

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

    pub fn change_account(&mut self, account: Account) {
        let (_, sender) = &self.receiver;
        if sender
            .send(MessageBroadcasting::ChangeAccount(account))
            .is_err()
        {
            todo!()
        }
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

    pub fn destroy(self) -> (Vec<RW>, BlockChain, UTXOSet) {
        for (_, sender) in self.peers.iter() {
            if sender.send(MessageBroadcasting::Exit).is_err() {
                todo!()
            }
        }

        let mut peers_streams = Vec::new();
        for (handle, _) in self.peers {
            match handle.join() {
                Ok(peer_stream) => peers_streams.push(peer_stream),
                Err(_) => todo!(),
            }
        }

        let (handle, sender) = self.receiver;

        if sender.send(MessageBroadcasting::Exit).is_err() {
            todo!()
        }

        let message_manager = match handle.join() {
            Ok(message_manager) => message_manager,
            Err(_) => todo!(),
        };

        let block_chain = message_manager.block_chain;
        let utxo_set = message_manager.utxo_set;

        (peers_streams, block_chain, utxo_set)
    }
}
