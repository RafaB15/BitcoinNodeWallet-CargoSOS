use crate::tui::message::Message;

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, transaction::Transaction, block::Block},
    wallet_structure::account::Account,
};

use std::{
    io::{Read, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    mem::replace,
};

pub struct Broadcasting<RW>
where
    RW: Read + Write + Send + 'static
{
    peers: Vec<JoinHandle<RW>>,
    received: Option<JoinHandle<MessageManager>>,
    sender: Sender<Message>
}

impl<RW> Broadcasting<RW>
where
    RW: Read + Write + Send + 'static
{
    fn new(account: Account, peers_streams: Vec<RW>, block_chain: BlockChain) -> Self {

        let (sender, receiver) = mpsc::channel::<Message>();

        let mut message_manager = MessageManager {
            receiver,
            account,
            transactions: Vec::new(),
            block_chain,
        };

        Broadcasting {
            received: Some(Self::create_receiver(message_manager)),
            peers: Self::create_peers(peers_streams),
            sender,
        }
    }

    fn create_receiver(mut message_manager: MessageManager) -> JoinHandle<MessageManager> {
        thread::spawn(move || {
            message_manager.receive_messages();
            message_manager
        })
    }

    fn create_peers(peers_streams: Vec<RW>) -> Vec<JoinHandle<RW>> {
        let mut peers: Vec<JoinHandle<RW>> = Vec::new();

        for peer_stream in peers_streams {
            peers.push(thread::spawn(move || {
            
                

                peer_stream
            }));
        }

        peers
    }

    pub fn change_account(&mut self, account: Account) {
        if self.sender.send(Message::Exit).is_err() {
            todo!()
        }

        let receiver = replace(&mut self.received, None);

        if let Some(receiver) = receiver {
            let mut message_manager = match receiver.join() {
                Ok(message_manager) => message_manager,
                Err(_) => todo!(),
            };

            message_manager.change_account(account);

            self.received = Some(Self::create_receiver(message_manager));
        } else {
            todo!()
        }
    }

    pub fn send_transaccion(&mut self, transaction: Transaction) {
        todo!()
    }

    pub fn destroy(mut self) -> (Vec<RW>, BlockChain) {
        if self.sender.send(Message::Exit).is_err() {
            todo!()
        }

        let block_chain = match self.received {
            Some(receiver) => {
                let mut message_manager = match receiver.join() {
                    Ok(message_manager) => message_manager,
                    Err(_) => todo!(),
                };
                    
                message_manager.block_chain
            },
            None => todo!(),
        };

        todo!()
    }
}

pub struct MessageManager {
    receiver: Receiver<Message>,
    account: Account,
    transactions: Vec<Transaction>,
    block_chain: BlockChain,
}

impl MessageManager {

    pub fn change_account(&mut self, account: Account) {
        self.account = account;
    }

    pub fn receive_messages(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            match message {
                Message::Transaction(transaction) => self.receive_transaction(transaction),
                Message::Block(block) => self.receive_block(block),
                Message::Exit => break,
            }
        }
    }

    fn receive_transaction(&mut self, transaction: Transaction) {
        todo!()
    }

    fn receive_block(&mut self, block: Block) {
        todo!()
    }
}