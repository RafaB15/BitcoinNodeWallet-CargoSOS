use crate::tui::message::Message;

use cargosos_bitcoin::{
    block_structure::{
        block::Block, block_chain::BlockChain, transaction::Transaction, utxo_set::UTXOSet,
    },
    wallet_structure::account::Account,
};

use std::{
    io::{Read, Write},
    mem::replace,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct Broadcasting<RW>
where
    RW: Read + Write + Send + TryCopy + 'static,
{
    peers: Vec<(JoinHandle<RW>, Sender<Message>)>,
    receiver: JoinHandle<MessageManager>,
    sender: Sender<Message>,
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + TryCopy + 'static,
{
    fn new(
        account: Account,
        peers_streams: Vec<RW>,
        block_chain: BlockChain,
        utxo_set: UTXOSet,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<Message>();

        let mut message_manager = MessageManager {
            receiver,
            account,
            transactions: Vec::new(),
            block_chain,
            utxo_set,
        };

        let stop = Arc::new(Mutex::new(false));

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
        sender: Sender<Message>,
    ) -> Vec<(JoinHandle<RW>, Sender<Message>)> {
        peers_streams
            .map(|peer_stream| {
                let sender_clone = sender.clone();
                let (sender_message, receiver_message) = mpsc::channel::<Message>();

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
        if self.sender.send(Message::ChangeAccount(account)).is_err() {
            todo!()
        }
    }

    pub fn send_transaction(&mut self, transaction: Transaction) {
        for (_, sender) in self.peers.iter() {
            if sender
                .send(Message::Transaction(transaction.clone()))
                .is_err()
            {
                todo!()
            }
        }
    }

    pub fn destroy(mut self) -> (Vec<RW>, BlockChain, UTXOSet) {
        for (_, sender) in self.peers.iter() {
            if sender.send(Message::Exit).is_err() {
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

        todo!()
    }
}

pub struct PeerManager<RW>
where
    RW: Read + Write + Send + TryCopy + 'static,
{
    peer: RW,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl<RW> PeerManager<RW>
where
    RW: Read + Write + Send + TryCopy + 'static,
{
    fn listen_peers(mut self) -> Self {
        while true {
            if let Ok(message) = self.receiver.try_recv() {
                match message {
                    Message::Transaction(transaction) => todo!(),
                    Message::Exit => break,
                    _ => (),
                }
            }
        }

        self.peer
    }
}

pub struct MessageManager {
    receiver: Receiver<Message>,
    account: Account,
    transactions: Vec<Transaction>,
    block_chain: BlockChain,
    utxo_set: UTXOSet,
}

impl MessageManager {
    pub fn receive_messages(mut self) -> Self {
        while let Ok(message) = self.receiver.recv() {
            match message {
                Message::Transaction(transaction) => self.receive_transaction(transaction),
                Message::Block(block) => self.receive_block(block),
                Message::ChangeAccount(account) => self.change_account(account),
                Message::Exit => break,
            }
        }

        self
    }

    fn change_account(&mut self, account: Account) {
        self.account = account;
    }

    fn receive_transaction(&mut self, transaction: Transaction) {
        todo!()
    }

    fn receive_block(&mut self, block: Block) {
        todo!()
    }
}
