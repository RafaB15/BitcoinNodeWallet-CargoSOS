use crate::{
    block_structure::transaction::Transaction, messages::message_header::MessageHeader, serialization::error_serialization::ErrorSerialization,
};

use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, TryRecvError},
    convert::Into,
};

#[derive(Debug)]
pub enum Work {
    Message(MessageHeader),
    SendTransaction(Transaction),
    Stop,
}

impl Work {
    pub fn listen<RW: Read + Write, M : Into<Work>>(stream: &mut RW, receiver: &Receiver<M>) -> Work {
        loop {
            match MessageHeader::deserialize_header(stream) {
                Ok(header) => return Work::Message(header),
                Err(ErrorSerialization::InformationNotReady) => {},
                _ => return Work::Stop,
            }

            match receiver.try_recv() {
                Ok(message_to_peer) => return message_to_peer.into(),
                Err(TryRecvError::Disconnected) => return Work::Stop,
                Err(_) => {},
            }
        }
    }
}
