use crate::{
    messages::message_header::MessageHeader, serialization::error_serialization::ErrorSerialization,
};

use std::{
    convert::Into,
    io::{Read, Write},
    sync::mpsc::{Receiver, TryRecvError},
};

#[derive(Debug)]
pub enum Work<I> {
    Message(MessageHeader),
    Information(I),
    Stop,
}

impl<I> Work<I> {
    pub fn listen<RW: Read + Write, M: Into<Work<I>>>(
        stream: &mut RW,
        receiver: &Receiver<M>,
    ) -> Self {
        loop {
            match MessageHeader::deserialize_header(stream) {
                Ok(header) => return Work::Message(header),
                Err(ErrorSerialization::InformationNotReady) => {}
                _ => return Work::Stop,
            }

            match receiver.try_recv() {
                Ok(message_to_peer) => return message_to_peer.into(),
                Err(TryRecvError::Disconnected) => return Work::Stop,
                Err(_) => {}
            }
        }
    }
}
