use std::{
    convert::Into,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{Receiver, TryRecvError},
};

#[derive(Debug)]
pub enum Listener<I> {
    Stream(TcpStream, SocketAddr),
    Information(I),
    Stop,
}

impl<I> Listener<I> {
    pub fn listen<M: Into<Listener<I>>>(
        listener: &mut TcpListener,
        receiver: &Receiver<M>,
    ) -> Self {
        loop {
            match listener.accept() {
                Ok((stream, socket_address)) => return Listener::Stream(stream, socket_address),
                Err(error) => {
                    if error.kind() != std::io::ErrorKind::WouldBlock {
                        return Listener::Stop;
                    }
                }
            }

            match receiver.try_recv() {
                Ok(message) => return message.into(),
                Err(TryRecvError::Disconnected) => return Listener::Stop,
                Err(_) => {}
            }
        }
    }
}
