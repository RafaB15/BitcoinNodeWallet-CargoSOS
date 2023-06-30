use crate::{
    block_structure::transaction::Transaction,
    messages::message_header::MessageHeader,
    node_structure::message_to_peer::MessageToPeer,
};

use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, TryRecvError},
};


#[derive(Debug)]
pub enum Work {
    Message(MessageHeader),
    SendTransaction(Transaction),
    Stop,
}

impl Work {
    pub fn listen<RW: Read + Write>(
        stream: &mut RW,
        receiver: &Receiver<MessageToPeer>,
    ) -> Work {
        loop {
            if let Ok(header) = MessageHeader::deserialize_header(stream) {
                return Work::Message(header);
            }

            match receiver.try_recv() {
                Ok(message_to_peer) => return message_to_peer.into(),
                Err(TryRecvError::Disconnected) => return Work::Stop,
                Err(_) => continue,
            }
        }
    }
}