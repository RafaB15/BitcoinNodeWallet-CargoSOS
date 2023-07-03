use std::net::{SocketAddr, TcpStream};

#[derive(Debug)]
pub enum ConnectionEvent {
    PotentialClient(TcpStream, SocketAddr),
    PotentialPeer(SocketAddr),
    Stop,
}
