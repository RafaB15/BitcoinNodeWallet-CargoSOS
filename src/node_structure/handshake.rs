use crate::connections::{
    error_connection::ErrorConnection, p2p_protocol::ProtocolVersionP2P,
    suppored_services::SupportedServices,
};

use crate::logs::logger_sender::LoggerSender;

use crate::messages::{
    message,
    command_name::CommandName,
    version_message::VersionMessage,
    verack_message::VerackMessage,
    bitfield_services::BitfieldServices,
    error_message::ErrorMessage,
};

use std::net::{
    SocketAddr, 
    TcpStream
};

const IGNORE_NONCE: u64 = 0;
const IGNORE_USER_AGENT: &str = "";
const NO_NEW_TRANSACTIONS: bool = false;
const TESTNET_MAGIC_NUMBERS: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];

pub struct Handshake {
    protocol_version: ProtocolVersionP2P,
    services: BitfieldServices,
    blockchain_height: i32,
    sender_log: LoggerSender,
}

impl Handshake {
    pub fn new(
        protocol_version: ProtocolVersionP2P,
        services: BitfieldServices,
        blockchain_height: i32,
        sender_log: LoggerSender,
    ) -> Self {
        Handshake {
            protocol_version,
            services,
            blockchain_height,
            sender_log,
        }
    }

    ///Function that sends a version message to the given potential peer.
    pub fn send_testnet_version_message(
        &self, 
        local_socket_addr: &SocketAddr, 
        potential_peer: &SocketAddr, 
        potencial_peer_stream: &mut TcpStream
    ) -> Result<(), ErrorMessage>
    {
        let version_message = VersionMessage::new(
            self.protocol_version,
            self.services.clone(),
            BitfieldServices::new(vec![SupportedServices::NodeNetworkLimited]),
            potential_peer,
            local_socket_addr,
            IGNORE_NONCE,
            IGNORE_USER_AGENT.to_string(),
            self.blockchain_height,
            NO_NEW_TRANSACTIONS,
        );  

        message::serialize_message(
            potencial_peer_stream, 
            TESTNET_MAGIC_NUMBERS, 
            CommandName::Version, 
            &version_message,
        )?;
        Ok(())
    }

    ///Function that sends a verack message to the given potential peer.
    pub fn send_testnet_verack_message(&self, potencial_peer_stream: &mut TcpStream) -> Result<(), ErrorMessage>{        
        
        let verack_message = VerackMessage;

        message::serialize_message(
            potencial_peer_stream, 
            TESTNET_MAGIC_NUMBERS, 
            CommandName::Verack, 
            &verack_message
        )?;
      
        Ok(())
    }

    ///Function that tries to do the handshake with the given potential peer.
    pub fn attempt_connection_with_testnet_peer(
        &self,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorConnection> 
    {
        let mut potencial_peer_stream = match TcpStream::connect(potential_peer) {
            Ok(stream) => stream,
            Err(_) => return Err(ErrorConnection::ErrorCannotConnectToAddress),
        };

        let local_socket_addr = match potencial_peer_stream.local_addr() {
            Ok(addr) => addr,
            Err(_) => return Err(ErrorConnection::ErrorCannotObtainOwnAddress),
        };

        if let Err(e) = self.send_testnet_version_message(
            &local_socket_addr,
            potential_peer,
            &mut potencial_peer_stream,
        ) {
            let _ = self.sender_log.log_connection(format!(
                "Error while sending version message to peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotSendMessage);
        } else {
            let _ = self
                .sender_log
                .log_connection(format!("Version message sent to peer {}", potential_peer));
        }

        let header_version = match message::deserialize_until_found(
            &mut potencial_peer_stream, 
            CommandName::Version) {
                Ok(header) => header,
                Err(e) => {
                    let _ = self.sender_log.log_connection(format!(
                        "Error while receiving the header message from peer {}: {:?}",
                        potential_peer, e
                    ));
                    return Err(ErrorConnection::ErrorCannotReceiveMessage);
                }
        };

        if let Err(e) = VersionMessage::deserialize_message(&mut potencial_peer_stream, header_version) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving version message from peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotReceiveMessage);
        }

        if let Err(e) = self.send_testnet_verack_message(&mut potencial_peer_stream) {
            let _ = self.sender_log.log_connection(format!(
                "Error while sending verack message to peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotSendMessage);
        } else {
            let _ = self
                .sender_log
                .log_connection(format!("Verack message sent to peer {}", potential_peer));
        }

        let header_verack = match message::deserialize_until_found(
            &mut potencial_peer_stream, 
            CommandName::Verack) 
        {
            Ok(header) => header,
            Err(e) => {
                let _ = self.sender_log.log_connection(format!(
                    "Error while receiving the header message from peer {}: {:?}",
                    potential_peer, e
                ));
                return Err(ErrorConnection::ErrorCannotReceiveMessage);
            }
        };

        if let Err(e) = VerackMessage::deserialize_message(
            &mut potencial_peer_stream, 
            header_verack) 
        {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving verack message from peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotReceiveMessage);
        }

        Ok(())
    }

    ///Function that tries to do the handshake with the given vector of potential peers.
    //Recordar implementar la funcionalidad con
    pub fn connect_to_testnet_peer(&mut self, potential_peer: SocketAddr) -> Result<SocketAddr, ErrorConnection> {
        
        if let Err(e) = self.attempt_connection_with_testnet_peer(&potential_peer) {
            let _ = self.sender_log.log_connection(format!("Error while trying to connect to peer {}: {:?}", potential_peer, e));
        } else {
            let _ = self.sender_log.log_connection(format!("Connection with peer {} established", potential_peer));
        }
        Ok(potential_peer)
    }
}
