use crate::connections::{
    p2p_protocol::ProtocolVersionP2P,
    ibd_methods::IBDMethod,
    suppored_services::SupportedServices,
    error_connection::ErrorConnection
};

use std::net::{
    SocketAddr,
    TcpStream,
};

use crate::messages::{
    version_message::VersionMessage,
    verack_message::VerackMessage,
    error_message::ErrorMessage,
    serializable::Serializable,
    deserializable::Deserializable,
};

const IGNORE_NONCE: u64 = 0;
const IGNORE_USER_AGENT: &str = "";
const NO_NEW_TRANSACTIONS: bool = false; 
const TESTNET_MAGIC_NUMBERS: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];

pub struct Node {
    protocol_version: ProtocolVersionP2P,
    ibd_method: IBDMethod,
    pub peers_addrs: Vec<SocketAddr>, //Cambiar el público luego
    services: SupportedServices,
    blockchain_height: i32,
}


impl Node {
    
    pub fn new(
        protocol_version: ProtocolVersionP2P,
        ibd_method: IBDMethod,
        services: SupportedServices,
        blockchain_height: i32,
    ) -> Self {
        Node {
            protocol_version,
            ibd_method,
            peers_addrs: vec![],
            services,
            blockchain_height
        }
    }

    ///Function that sends a version message to the given potential peer.
    pub fn send_testnet_version_message(&self, local_socket_addr: &SocketAddr, potential_peer: &SocketAddr, potencial_peer_stream: &mut TcpStream) -> Result<(), ErrorMessage>{
        let version_message = VersionMessage::new(
            TESTNET_MAGIC_NUMBERS,
            self.protocol_version,
            self.services,
            SupportedServices::Unname,
            potential_peer,
            local_socket_addr,
            IGNORE_NONCE,
            IGNORE_USER_AGENT.to_string(),
            self.blockchain_height,
            NO_NEW_TRANSACTIONS,
        );  
        version_message.serialize(potencial_peer_stream)?;
        Ok(())
    }

    ///Function that sends a verack message to the given potential peer.
    pub fn send_testnet_verack_message(&self, potencial_peer_stream: &mut TcpStream) -> Result<(), ErrorMessage>{
        let verack_message = VerackMessage::new(TESTNET_MAGIC_NUMBERS);
        verack_message.serialize(potencial_peer_stream)?;
        Ok(())
    }

    ///Function that tries to do the handshake with the given potential peer.
    pub fn attempt_connection_with_testnet_peer(&self, potential_peer: &SocketAddr) -> Result<(), ErrorConnection>{

        let mut potencial_peer_stream = match TcpStream::connect(potential_peer) {
            Ok(stream) => stream,
            Err(_) => return Err(ErrorConnection::ErrorCannotConnectToAddress),
        };

        let local_socket_addr = match potencial_peer_stream.local_addr() {
            Ok(addr) => addr,
            Err(_) => return Err(ErrorConnection::ErrorCannotObtainOwnAddress),
        };

        match self.send_testnet_version_message(&local_socket_addr, potential_peer, &mut potencial_peer_stream) {
            Ok(_) => println!("Version message sent to peer {}", potential_peer),
            Err(e) => {
                println!("Error while sending version message to peer {}: {:?}", potential_peer, e);
                return Err(ErrorConnection::ErrorCannotSendMessage);
            }
        };

        match VersionMessage::deserialize(&mut potencial_peer_stream) {
            Ok(message) => message,
            Err(e) => {
                println!("Error while receiving version message from peer {}: {:?}", potential_peer, e);
                return Err(ErrorConnection::ErrorCannotReceiveMessage);
            }
        };

        match self.send_testnet_verack_message(&mut potencial_peer_stream) {
            Ok(_) => println!("Verack message sent to peer {}", potential_peer),
            Err(e) => {
                println!("Error while sending verack message to peer {}: {:?}", potential_peer, e);
                return Err(ErrorConnection::ErrorCannotSendMessage);
            }
        };

        match VerackMessage::deserialize(&mut potencial_peer_stream) {
            Ok(message) => message,
            Err(e) => {
                println!("Error while receiving verack message from peer {}: {:?}", potential_peer, e);
                return Err(ErrorConnection::ErrorCannotReceiveMessage);
            }
        };

        Ok(())
    }


    ///Function that tries to do the handshake with the given vector of potential peers.
    //Recordar implementar la funcionalidad con
    pub fn connect_to_testnet_peers(&mut self, potential_peers: &Vec<SocketAddr>) -> Result<(), ErrorConnection> {
        for potential_peer in potential_peers {
            match self.attempt_connection_with_testnet_peer(potential_peer) {
                Ok(_) => {
                    println!("Connection with peer {} established", potential_peer);
                    self.peers_addrs.push(potential_peer.clone());
            },
                Err(e) => println!("Error while trying to connect to peer {}: {:?}", potential_peer, e)
            }
        }
        Ok(())
    }


}