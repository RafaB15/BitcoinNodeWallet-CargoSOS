use super::{
    message_broadcasting::MessageBroadcasting, message_manager::MessageManager,
    peer_manager::PeerManager,
};

use cargosos_bitcoin::{
    block_structure::{
        block::Block, block_chain::BlockChain, transaction::Transaction, utxo_set::UTXOSet,
    },
    messages::{
        block_message::BlockMessage, command_name::CommandName, get_data_message::GetDataMessage,
        message, message_header::MessageHeader,
    },
    wallet_structure::account::Account,
};

use std::{
    io::{Read, Write},
    mem::replace,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

type ReadWrite = Read + Write + Send + 'static;

pub struct Broadcasting<RW: ReadWrite> {
    peers: Vec<(JoinHandle<RW>, Sender<MessageBroadcasting>)>,
    receiver: JoinHandle<MessageManager>,
    sender: Sender<MessageBroadcasting>,
}

impl<RW: ReadWrite> Broadcasting<RW> {
    fn new(
        account: Account,
        peers_streams: Vec<RW>,
        block_chain: BlockChain,
        utxo_set: UTXOSet,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<MessageBroadcasting>();

        let mut message_manager = MessageManager {
            receiver,
            account,
            transactions: Vec::new(),
            block_chain,
            utxo_set,
        };

        Broadcasting {
            receiver: Self::create_receiver(message_manager),
            peers: Self::create_peers(peers_streams, sender.clone()),
            sender,
        }
    }

    fn create_receiver(mut message_manager: MessageManager) -> JoinHandle<MessageManager> {
        thread::spawn(move || message_manager.receive_messages())
    }

    fn create_peers(
        peers_streams: Vec<RW>,
        sender: Sender<MessageBroadcasting>,
    ) -> Vec<(JoinHandle<RW>, Sender<Message>)> {
        peers_streams
            .map(|peer_stream| {
                let sender_clone = sender.clone();
                let (sender_message, receiver_message) = mpsc::channel::<MessageBroadcasting>();

                let handle = thread::spawn(move || {
                    let peer_manager = PeerManager {
                        peer: peer_stream,
                        sender: sender_clone,
                        receiver: receiver_message,
                    };

                    peer_manager.listen_peers()
                });

                (handle, sender_message)
            })
            .collect()
    }

    pub fn change_account(&mut self, account: Account) {
        if self
            .sender
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

    pub fn destroy(mut self) -> (Vec<RW>, BlockChain, UTXOSet) {
        for (_, sender) in self.peers.iter() {
            if sender.send(MessageBroadcasting::Exit).is_err() {
                todo!()
            }
        }

        let mut peers_streams = Vec::new();
        for (handle, _) in self.peers.iter() {
            match handle.join() {
                Ok(peer_stream) => peers_streams.push(peer_stream),
                Err(_) => todo!(),
            }
        }

        if self.sender.send(Message::Exit).is_err() {
            todo!()
        }

        let mut message_manager = match self.receiver.join() {
            Ok(message_manager) => message_manager,
            Err(_) => todo!(),
        };

        let block_chain = message_manager.block_chain;
        let utxo_set = message_manager.utxo_set;

        (peers_streams, block_chain, utxo_set)
    }
}
