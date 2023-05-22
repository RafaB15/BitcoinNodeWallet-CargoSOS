use crate::connections::{
    error_connection::ErrorConnection, p2p_protocol::ProtocolVersionP2P,
    socket_conversion::socket_to_ipv6_port, suppored_services::SupportedServices,
};

use crate::logs::logger_sender::LoggerSender;

use crate::messages::{
    bitfield_services::BitfieldServices,
    command_name::CommandName,
    error_message::ErrorMessage,
    message::{self, Message},
    send_headers::SendHeadersMessage,
    verack_message::VerackMessage,
    version_message::VersionMessage,
};

use std::net::SocketAddr;

use chrono::offset::Utc;

use std::io::{Read, Write};

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
    pub fn send_testnet_version_message<RW: Read + Write>(
        &self,
        local_socket_addr: &SocketAddr,
        potential_peer: &SocketAddr,
        peer_stream: &mut RW,
    ) -> Result<(), ErrorMessage> {
        let timestamp = Utc::now();
        let (recv_addr, recv_port) = socket_to_ipv6_port(potential_peer);
        let (trans_addr, trans_port) = socket_to_ipv6_port(local_socket_addr);

        let recv_services = BitfieldServices::new(vec![SupportedServices::NodeNetworkLimited]);

        let version_message = VersionMessage {
            version: self.protocol_version,
            services: self.services.clone(),
            timestamp,
            recv_services,
            recv_addr,
            recv_port,
            trans_addr,
            trans_port,
            nonce: IGNORE_NONCE,
            user_agent: IGNORE_USER_AGENT.to_string(),
            start_height: self.blockchain_height,
            relay: NO_NEW_TRANSACTIONS,
        };

        VersionMessage::serialize_message(peer_stream, TESTNET_MAGIC_NUMBERS, &version_message)?;

        Ok(())
    }

    ///Function that sends a verack message to the given potential peer.
    pub fn send_testnet_verack_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
    ) -> Result<(), ErrorMessage> {
        VerackMessage::serialize_message(peer_stream, TESTNET_MAGIC_NUMBERS, &VerackMessage)?;

        Ok(())
    }

    pub fn send_testnet_sendheaders_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
    ) -> Result<(), ErrorMessage> {
        SendHeadersMessage::serialize_message(
            peer_stream,
            TESTNET_MAGIC_NUMBERS,
            &SendHeadersMessage,
        )?;

        Ok(())
    }

    fn attempt_version_message_exchange<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        local_socket: &SocketAddr,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorConnection> {
        if let Err(e) = self.send_testnet_version_message(local_socket, potential_peer, peer_stream)
        {
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

        let header_version =
            match message::deserialize_until_found(peer_stream, CommandName::Version) {
                Ok(header) => header,
                Err(e) => {
                    let _ = self.sender_log.log_connection(format!(
                        "Error while receiving the header of version message from peer {}: {:?}",
                        potential_peer, e
                    ));
                    return Err(ErrorConnection::ErrorCannotReceiveMessage);
                }
            };

        if let Err(e) = VersionMessage::deserialize_message(peer_stream, header_version) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving version message from peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotReceiveMessage);
        }
        Ok(())
    }

    fn attempt_verack_message_exchange<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorConnection> {
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

        let header_verack = match message::deserialize_until_found(peer_stream, CommandName::Verack)
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

        if let Err(e) = VerackMessage::deserialize_message(peer_stream, header_verack) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving verack message from peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotReceiveMessage);
        }
        Ok(())
    }

    ///Function that tries to do the handshake with the given potential peer.
    fn attempt_connection_with_testnet_peer<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        local_socket: &SocketAddr,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorConnection> {
        if let Err(e) =
            self.attempt_version_message_exchange(peer_stream, local_socket, potential_peer)
        {
            let _ = self.sender_log.log_connection(format!(
                "Error while trying to exchange version messages with peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotConnectToAddress);
        } else {
            let _ = self.sender_log.log_connection(format!(
                "Version message exchange with peer {} finished successfully",
                potential_peer
            ));
        }

        if let Err(e) = self.attempt_verack_message_exchange(peer_stream, potential_peer) {
            let _ = self.sender_log.log_connection(format!(
                "Error while trying to exchange verack messages with peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotConnectToAddress);
        } else {
            let _ = self.sender_log.log_connection(format!(
                "Verack message exchange with peer {} finished successfully",
                potential_peer
            ));
        }

        if let Err(e) = self.send_testnet_sendheaders_message(peer_stream) {
            let _ = self.sender_log.log_connection(format!(
                "Error while sending send headers message to peer {}: {:?}",
                potential_peer, e
            ));
            return Err(ErrorConnection::ErrorCannotSendMessage);
        } else {
            let _ = self.sender_log.log_connection(format!(
                "Send headers message sent to peer {}",
                potential_peer
            ));
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
    ) -> Result<(), ErrorConnection> {
        if let Err(e) =
            self.attempt_connection_with_testnet_peer(peer_stream, local_socket, potential_peer)
        {
            let _ = self.sender_log.log_connection(format!(
                "Error while trying to connect to peer {}: {:?}",
                potential_peer, e
            ));

            Err(ErrorConnection::ErrorCannotConnectToAddress)
        } else {
            let _ = self.sender_log.log_connection(format!(
                "Connection with peer {} established",
                potential_peer
            ));
            Ok(())
        }
    }
}
