use crate::connections::{
    p2p_protocol::ProtocolVersionP2P,
    ibd_methods::IBDMethod,
    suppored_services::SupportedServices,
    connection_error::ConnectionError
};
use crate::messages::serializable::Serializable;
use crate::messages::deserializable::Deserializable;

use std::net::{
    SocketAddr,
    TcpStream,
};
use crate::connections::socket_conversion::socket_to_ipv6_port;

use crate::messages::{
    message::Message,
    version_message::VersionMessage,
    verack_message::VerackMessage,
    payload::Payload,
    error_message::ErrorMessage,
};

use chrono::offset::Utc;

const IGNORE_NONCE: u64 = 0;
const IGNORE_USER_AGENT: &str = "";
const NO_NEW_TRANSACTIONS: bool = false; 
const TESTNET_MAGIC_NUMBERS: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];

pub struct Node {
    protocol_version: ProtocolVersionP2P,
    ibd_method: IBDMethod,
    peers_addrs: Vec<SocketAddr>,
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

    ///Function that builds a version message payload with the current information of the node
    pub fn build_version_message_payload(
        &self,
        recv_socket_addr: &SocketAddr,
        recv_services: SupportedServices,
        trans_socket_addr: &SocketAddr,
        nonce: u64,
        user_agent: String,
        relay: bool
    ) ->  Payload {

        let timestamp = Utc::now();
        let (recv_addr, recv_port) = socket_to_ipv6_port(recv_socket_addr);
        let (trans_addr, trans_port) = socket_to_ipv6_port(trans_socket_addr);
        
        let payload = VersionMessage::new(
            self.protocol_version, 
            self.services, 
            timestamp, 
            recv_services, 
            recv_addr, 
            recv_port, 
            trans_addr, 
            trans_port, 
            nonce, 
            user_agent, 
            self.blockchain_height, 
            relay);
        
        Payload::VersionMessage(payload)
    }

    ///Function that builds a version message with the current information of the node
    pub fn build_version_message(
        &self,
        magic_bytes: [u8; 4],
        recv_socket_addr: &SocketAddr,
        recv_services: SupportedServices,
        trans_socket_addr: &SocketAddr,
        nonce: u64,
        user_agent: String,
        relay: bool
    ) -> Message {

        let payload = self.build_version_message_payload(recv_socket_addr, recv_services, trans_socket_addr, nonce, user_agent, relay);
        Message::new(magic_bytes, payload)
    }

    pub fn send_testnet_version_message(&self, local_socket_addr: &SocketAddr, potential_peer: &SocketAddr, potencial_peer_stream: &mut TcpStream) -> Result<(), ErrorMessage>{
        let version_message = self.build_version_message(
            TESTNET_MAGIC_NUMBERS, 
            potential_peer, 
            SupportedServices::Unname, 
            local_socket_addr, 
            IGNORE_NONCE, 
            IGNORE_USER_AGENT.to_string(), 
            NO_NEW_TRANSACTIONS);
            
        version_message.serialize(potencial_peer_stream)?;
        Ok(())
    }

    pub fn send_testnet_verack_message(&self, potencial_peer_stream: &mut TcpStream) -> Result<(), ErrorMessage>{
        let verack_message = Message::new(TESTNET_MAGIC_NUMBERS, Payload::VerackMessage(VerackMessage::new()));
        verack_message.serialize(potencial_peer_stream)?;
        Ok(())
    }

    ///Function that tries to do the handshake with the given potential peer.
    pub fn attempt_connection_with_testnet_peer(&self, potential_peer: &SocketAddr) -> Result<(), ConnectionError>{

        let mut potencial_peer_stream = match TcpStream::connect(potential_peer) {
            Ok(stream) => stream,
            Err(_) => return Err(ConnectionError::ErrorCannotConnectToAddress),
        };

        let local_socket_addr = match potencial_peer_stream.local_addr() {
            Ok(addr) => addr,
            Err(_) => return Err(ConnectionError::ErrorCannotObtainOwnAddress),
        };

        match self.send_testnet_version_message(&local_socket_addr, potential_peer, &mut potencial_peer_stream) {
            Ok(_) => println!("Version message sent to peer {}", potential_peer),
            Err(e) => {
                println!("Error while sending version message to peer {}: {:?}", potential_peer, e);
                return Err(ConnectionError::ErrorCannotSendMessage);
            }
        };

        match Message::deserialize(&mut potencial_peer_stream) {
            Ok(message) => message,
            Err(e) => {
                println!("Error while receiving version message from peer {}: {:?}", potential_peer, e);
                return Err(ConnectionError::ErrorCannotReceiveMessage);
            }
        };

        match self.send_testnet_verack_message(&mut potencial_peer_stream) {
            Ok(_) => println!("Verack message sent to peer {}", potential_peer),
            Err(e) => {
                println!("Error while sending verack message to peer {}: {:?}", potential_peer, e);
                return Err(ConnectionError::ErrorCannotSendMessage);
            }
        };

        match Message::deserialize(&mut potencial_peer_stream) {
            Ok(message) => message,
            Err(e) => {
                println!("Error while receiving verack message from peer {}: {:?}", potential_peer, e);
                return Err(ConnectionError::ErrorCannotReceiveMessage);
            }
        };

        Ok(())
    }


    ///Function that tries to do the handshake with the given vector of potential peers.
    //Recordar implementar la funcionalidad con
    pub fn connect_to_testnet_peers(&mut self, potential_peers: &Vec<SocketAddr>) -> Result<(), ConnectionError> {
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