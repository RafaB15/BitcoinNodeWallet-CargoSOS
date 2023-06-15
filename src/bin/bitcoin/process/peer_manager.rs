use super::message_broadcasting::MessageBroadcasting;

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

pub struct PeerManager<RW>
where
    RW: Read + Write + Send + 'static,
{
    peer: RW,
    sender: Sender<MessageBroadcasting>,
    receiver: Receiver<MessageBroadcasting>,
}

impl<RW> PeerManager<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(
        peer: RW,
        sender: Sender<MessageBroadcasting>,
        receiver: Receiver<MessageBroadcasting>,
    ) -> Self {
        PeerManager {
            peer,
            sender,
            receiver,
        }
    }

    pub fn listen_peers(mut self) -> RW {
        while let Ok(header) = MessageHeader::deserialize_header(&mut self.peer) {
            self.manage_message(header);

            if let Ok(message) = self.receiver.try_recv() {
                match message {
                    MessageBroadcasting::Transaction(transaction) => {
                        self.send_transaction(transaction)
                    }
                    MessageBroadcasting::Exit => break,
                    _ => (),
                }
            }
        }

        self.peer
    }

    fn manage_message(&self, header: MessageHeader) {
        todo!()
    }

    fn send_transaction(&self, transaction: Transaction) {
        todo!()
    }
}
