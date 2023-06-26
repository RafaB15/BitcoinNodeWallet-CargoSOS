use super::{error_node::ErrorNode, handshake_data::HandshakeData};

use crate::messages::{
    bitfield_services::BitfieldServices,
    command_name::CommandName,
    message::{self, Message},
    send_headers_message::SendHeadersMessage,
    verack_message::VerackMessage,
    version_message::VersionMessage,
};

use crate::connections::{
    error_connection::ErrorConnection, p2p_protocol::ProtocolVersionP2P,
    socket_conversion::socket_to_ipv6_port, supported_services::SupportedServices,
};

use crate::logs::logger_sender::LoggerSender;

use std::{
    io::{Read, Write},
    net::SocketAddr,
};

use chrono::offset::Utc;

pub struct Handshake {
    protocol_version: ProtocolVersionP2P,
    services: BitfieldServices,
    blockchain_height: i32,
    data: HandshakeData,
    sender_log: LoggerSender,
}

impl Handshake {
    pub fn new(
        protocol_version: ProtocolVersionP2P,
        services: BitfieldServices,
        blockchain_height: i32,
        data: HandshakeData,
        sender_log: LoggerSender,
    ) -> Self {
        Handshake {
            protocol_version,
            services,
            blockchain_height,
            data,
            sender_log,
        }
    }

    /// Function that sends a version message to the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    pub fn send_testnet_version_message<RW: Read + Write>(
        &self,
        local_socket_addr: &SocketAddr,
        potential_peer: &SocketAddr,
        peer_stream: &mut RW,
    ) -> Result<(), ErrorNode> {
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
            nonce: self.data.nonce,
            user_agent: self.data.user_agent.clone(),
            start_height: self.blockchain_height,
            relay: self.data.relay,
        };

        VersionMessage::serialize_message(peer_stream, self.data.magic_number, &version_message)?;

        Ok(())
    }

    /// Function that sends a verack message to the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    pub fn send_verack_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
    ) -> Result<(), ErrorNode> {
        VerackMessage::serialize_message(peer_stream, self.data.magic_number, &VerackMessage)?;

        Ok(())
    }

    /// Sends a send header message to the peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    pub fn send_sendheaders_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
    ) -> Result<(), ErrorNode> {
        SendHeadersMessage::serialize_message(
            peer_stream,
            self.data.magic_number,
            &SendHeadersMessage,
        )?;

        Ok(())
    }

    /// Send and waits to receive a version message from the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileReceivingMessage`: It will appear when there is an error in the reading from a stream
    fn attempt_version_message_exchange<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        local_socket: &SocketAddr,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorNode> {
        if let Err(error) =
            self.send_testnet_version_message(local_socket, potential_peer, peer_stream)
        {
            let _ = self.sender_log.log_connection(format!(
                "Error while sending version message to peer {}: {:?}",
                potential_peer, error
            ));
            return Err(error);
        } else {
            let _ = self
                .sender_log
                .log_connection(format!("Version message sent to peer {}", potential_peer));
        }

        let header_version =
            match message::deserialize_until_found(peer_stream, CommandName::Version) {
                Ok(header) => header,
                Err(error) => {
                    let _ = self.sender_log.log_connection(format!(
                        "Error while receiving the header of version message from peer {}: {:?}",
                        potential_peer, error
                    ));
                    return Err(error.into());
                }
            };

        if let Err(error) = VersionMessage::deserialize_message(peer_stream, header_version) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving version message from peer {}: {:?}",
                potential_peer, error
            ));
            return Err(error.into());
        }
        Ok(())
    }

    /// Send and waits to receive a verack message from the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileReceivingMessage`: It will appear when there is an error in the reading from a stream
    fn attempt_verack_message_exchange<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorNode> {
        if let Err(error) = self.send_verack_message(peer_stream) {
            let _ = self.sender_log.log_connection(format!(
                "Error while sending verack message to peer {}: {:?}",
                potential_peer, error
            ));
            return Err(error);
        } else {
            let _ = self
                .sender_log
                .log_connection(format!("Verack message sent to peer {}", potential_peer));
        }

        let header_verack = match message::deserialize_until_found(peer_stream, CommandName::Verack)
        {
            Ok(header) => header,
            Err(error) => {
                let _ = self.sender_log.log_connection(format!(
                    "Error while receiving the header message from peer {}: {:?}",
                    potential_peer, error
                ));
                return Err(error.into());
            }
        };

        if let Err(error) = VerackMessage::deserialize_message(peer_stream, header_verack) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving verack message from peer {}: {:?}",
                potential_peer, error
            ));
            return Err(error.into());
        }
        Ok(())
    }

    /// Function that tries to do the handshake with the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorConnection::ErrorCannotConnectToAddress`: It will appear when the connection is not established with a peer
    ///  * `ErrorConnection::ErrorCannotSendMessage`: It will appear when a message to a peer cannot be sent
    fn attempt_connection_with_peer<RW: Read + Write>(
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

        if let Err(e) = self.send_sendheaders_message(peer_stream) {
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

    /// Function that tries to do the handshake with the given vector of potential peers.
    ///
    /// ### Error
    ///  * `ErrorConnection::ErrorCannotConnectToAddress`: It will appear when the connection is not established with a peer
    ///  * `ErrorConnection::ErrorCannotSendMessage`: It will appear when a message to a peer cannot be sent
    pub fn connect_to_peer<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        local_socket: &SocketAddr,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorConnection> {
        if let Err(error) =
            self.attempt_connection_with_peer(peer_stream, local_socket, potential_peer)
        {
            let _ = self.sender_log.log_connection(format!(
                "Error while trying to connect to peer {}: {:?}",
                potential_peer, error
            ));

            Err(error)
        } else {
            let _ = self.sender_log.log_connection(format!(
                "Connection with peer {} established",
                potential_peer
            ));
            Ok(())
        }
    }
}
