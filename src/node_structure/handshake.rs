use super::handshake_data::HandshakeData;

use crate::{
    messages::{
        bitfield_services::BitfieldServices, message::Message, message_header::MessageHeader,
        send_headers_message::SendHeadersMessage, verack_message::VerackMessage,
        version_message::VersionMessage,
    },
    serialization::error_serialization::ErrorSerialization,
};

use crate::connections::{
    p2p_protocol::ProtocolVersionP2P, socket_conversion::socket_to_ipv6_port,
    supported_services::SupportedServices,
};

use crate::logs::logger_sender::LoggerSender;

use std::{
    io::{Read, Write},
    net::SocketAddr,
};

use chrono::offset::Utc;

#[derive(Debug, Clone)]
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
    pub fn send_version_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        local_socket_addr: &SocketAddr,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorSerialization> {
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

        if let Err(error) =
            VersionMessage::serialize_message(peer_stream, self.data.magic_number, &version_message)
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

            Ok(())
        }
    }

    /// Function that receives a version message from the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorSerialization::ErrorSerialization`: It will appear when there is an error in the serialization
    ///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorSerialization::ErrorWhileReading`: It will appear when there is an error in the reading from a stream
    pub fn receive_version_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        header: MessageHeader,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorSerialization> {
        if let Err(error) = VersionMessage::deserialize_message(peer_stream, header) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving version message from peer {}: {:?}",
                potential_peer, error
            ));
            return Err(error);
        }
        Ok(())
    }

    /// Function that sends a verack message to the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    pub fn send_verack_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorSerialization> {
        if let Err(error) =
            VerackMessage::serialize_message(peer_stream, self.data.magic_number, &VerackMessage)
        {
            let _ = self.sender_log.log_connection(format!(
                "Error while sending verack message to peer {}: {:?}",
                potential_peer, error
            ));
            return Err(error);
        } else {
            let _ = self
                .sender_log
                .log_connection(format!("Verack message sent to peer {}", potential_peer));
            Ok(())
        }
    }

    /// Function that receives a verack message from the given potential peer.
    ///
    /// ### Error
    ///  * `ErrorSerialization::ErrorSerialization`: It will appear when there is an error in the serialization
    ///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorSerialization::ErrorWhileReading`: It will appear when there is an error in the reading from a stream
    pub fn receive_verack_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        header: MessageHeader,
        potential_peer: &SocketAddr,
    ) -> Result<(), ErrorSerialization> {
        if let Err(error) = VerackMessage::deserialize_message(peer_stream, header) {
            let _ = self.sender_log.log_connection(format!(
                "Error while receiving verack message from peer {}: {:?}",
                potential_peer, error
            ));
            return Err(error.into());
        }

        Ok(())
    }

    /// Sends a send header message to the peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    pub fn send_sendheaders_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
    ) -> Result<(), ErrorSerialization> {
        SendHeadersMessage::serialize_message(
            peer_stream,
            self.data.magic_number,
            &SendHeadersMessage,
        )
    }
}
