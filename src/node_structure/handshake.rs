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

use std::io::{
    Read,
    Write,
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
    pub fn send_testnet_version_message<RW : Read + Write>(
        &self, 
        local_socket_addr: &SocketAddr, 
        potential_peer: &SocketAddr, 
        peer_stream: &mut RW
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
            peer_stream, 
            TESTNET_MAGIC_NUMBERS, 
            CommandName::Version, 
            &version_message,
        )?;
        Ok(())
    }

    ///Function that sends a verack message to the given potential peer.
    pub fn send_testnet_verack_message<RW : Read + Write>(
        &self, 
        peer_stream: &mut RW
    ) -> Result<(), ErrorMessage>
    {        
        
        let verack_message = VerackMessage;

        message::serialize_message(
            peer_stream, 
            TESTNET_MAGIC_NUMBERS, 
            CommandName::Verack, 
            &verack_message
        )?;
      
        Ok(())
    }

    ///Function that tries to do the handshake with the given potential peer.
    pub fn attempt_connection_with_testnet_peer<RW : Read + Write>(
        &self,
        peer_stream: &mut RW,
        local_socket: &SocketAddr,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorConnection> 
    {
        if let Err(e) = self.send_testnet_version_message(
            &local_socket,
            potential_peer,
            peer_stream,
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
            peer_stream, 
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

        if let Err(e) = VersionMessage::deserialize_message(
            peer_stream, 
            header_version
        ) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving version message from peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotReceiveMessage);
        }

        if let Err(e) = self.send_testnet_verack_message(peer_stream) {
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
            peer_stream, 
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
            peer_stream, 
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
    pub fn connect_to_testnet_peer<RW: Read + Write>(
        &self, 
        peer_stream: &mut RW,
        local_socket: &SocketAddr,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorConnection> 
    {
        
        if let Err(e) = self.attempt_connection_with_testnet_peer(
            peer_stream, 
            local_socket,
            &potential_peer
        ) {
            let _ = self.sender_log.log_connection(
                format!("Error while trying to connect to peer {}: {:?}", potential_peer, e)
            );

            Err(ErrorConnection::ErrorCannotConnectToAddress)
        } else {
            let _ = self.sender_log.log_connection(
                format!("Connection with peer {} established", potential_peer)
            );
            Ok(())
        }
    }
}
