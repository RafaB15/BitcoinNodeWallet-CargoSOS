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
    fn send_version_message<RW: Read + Write>(
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
    fn send_verack_message<RW: Read + Write>(&self, peer_stream: &mut RW) -> Result<(), ErrorNode> {
        VerackMessage::serialize_message(peer_stream, self.data.magic_number, &VerackMessage)?;

        Ok(())
    }

    /// Sends a send header message to the peer.
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    fn send_sendheaders_message<RW: Read + Write>(
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
        if let Err(error) = self.send_version_message(local_socket, potential_peer, peer_stream) {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        
        logs::logger,
        
    };

    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use chrono::{DateTime, NaiveDateTime};

    struct Stream {
        stream: Vec<u8>,
        pointer: usize,
    }

    impl Stream {
        pub fn new() -> Stream {
            Stream {
                stream: Vec::new(),
                pointer: 0,
            }
        }
    }

    impl Read for Stream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut i = 0;
            while i < buf.len() && self.pointer < self.stream.len() {
                buf[i] = self.stream[self.pointer];
                self.pointer += 1;
                i += 1;
            }
            Ok(i)
        }
    }

    impl Write for Stream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut i = 0;
            while i < buf.len() {
                self.stream.push(buf[i]);
                i += 1;
            }
            Ok(i)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn serialize_verack_message<RW: Read + Write>(
        stream: &mut RW,
        magic_number: [u8; 4],
    ) -> Result<(), ErrorNode> {
        VerackMessage::serialize_message(stream, magic_number, &VerackMessage)?;
        Ok(())
    }

    fn serialize_version_message<RW: Read + Write>(
        stream: &mut RW,
        protocol_version: ProtocolVersionP2P,
        services: BitfieldServices,
        block_height: i32,
        handshake_data: HandshakeData,
        local_ip: (Ipv4Addr, u16),
        remote_ip: (Ipv4Addr, u16),
    ) -> Result<(), ErrorNode> {
        let naive = NaiveDateTime::from_timestamp_opt(1234 as i64, 0).unwrap();
        let timestamp: DateTime<Utc> = DateTime::from_utc(naive, Utc);

        let version_message = VersionMessage {
            version: protocol_version,
            services: services,
            timestamp,
            recv_services: BitfieldServices::new(vec![SupportedServices::Unname]),
            recv_addr: Ipv4Addr::to_ipv6_mapped(&local_ip.0),
            recv_port: local_ip.1,
            trans_addr: Ipv4Addr::to_ipv6_mapped(&remote_ip.0),
            trans_port: remote_ip.1,
            nonce: handshake_data.nonce,
            user_agent: handshake_data.user_agent.clone(),
            start_height: block_height,
            relay: handshake_data.relay,
        };

        VersionMessage::serialize_message(stream, handshake_data.magic_number, &version_message)?;

        Ok(())
    }

    #[test]
    fn test01_verack_exchange() -> Result<(), ErrorNode> {
        let mut stream: Stream = Stream::new();

        let magic_number = [11, 17, 9, 7];
        serialize_verack_message(&mut stream, magic_number)?;

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);

        let handshake = Handshake::new(
            ProtocolVersionP2P::V70016,
            BitfieldServices::new(vec![SupportedServices::Unname]),
            0,
            HandshakeData {
                nonce: 0,
                user_agent: "".to_string(),
                relay: false,
                magic_number,
            },
            sender,
        );

        let potential_peer = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 8333);

        handshake.attempt_verack_message_exchange(&mut stream, &potential_peer)
    }

    #[test]
    fn test02_version_exchange() -> Result<(), ErrorNode> {
        let mut stream: Stream = Stream::new();

        let handshake_data = HandshakeData {
            nonce: 0,
            user_agent: "".to_string(),
            relay: false,
            magic_number: [11, 17, 9, 7],
        };

        let local_ip: (Ipv4Addr, u16) = (Ipv4Addr::new(127, 0, 0, 1), 8333);
        let remote_ip: (Ipv4Addr, u16) = (Ipv4Addr::new(127, 0, 0, 2), 8333);

        let p2p_protocol = ProtocolVersionP2P::V70016;
        let services = BitfieldServices::new(vec![SupportedServices::Unname]);
        let block_height = 0;

        serialize_version_message(
            &mut stream,
            p2p_protocol.clone(),
            services.clone(),
            block_height,
            handshake_data.clone(),
            local_ip.clone(),
            remote_ip.clone(),
        )?;

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);

        let handshake =
            Handshake::new(p2p_protocol, services, block_height, handshake_data, sender);

        let local_socket = SocketAddr::new(IpAddr::V4(local_ip.0), local_ip.1);
        let potential_peer = SocketAddr::new(IpAddr::V4(remote_ip.0), remote_ip.1);

        handshake.attempt_version_message_exchange(&mut stream, &local_socket, &potential_peer)
    }

    #[test]
    fn test03_connection_to_peer_successfully() -> Result<(), ErrorConnection> {
        let mut stream: Stream = Stream::new();

        let handshake_data = HandshakeData {
            nonce: 0,
            user_agent: "".to_string(),
            relay: false,
            magic_number: [11, 17, 9, 7],
        };

        let local_ip: (Ipv4Addr, u16) = (Ipv4Addr::new(127, 0, 0, 1), 8333);
        let remote_ip: (Ipv4Addr, u16) = (Ipv4Addr::new(127, 0, 0, 2), 8333);

        let p2p_protocol = ProtocolVersionP2P::V70016;
        let services = BitfieldServices::new(vec![SupportedServices::Unname]);
        let block_height = 0;

        serialize_version_message(
            &mut stream,
            p2p_protocol.clone(),
            services.clone(),
            block_height,
            handshake_data.clone(),
            local_ip.clone(),
            remote_ip.clone(),
        )
        .unwrap();

        serialize_verack_message(&mut stream, handshake_data.magic_number).unwrap();

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);

        let handshake =
            Handshake::new(p2p_protocol, services, block_height, handshake_data, sender);

        let local_socket = SocketAddr::new(IpAddr::V4(local_ip.0), local_ip.1);
        let potential_peer = SocketAddr::new(IpAddr::V4(remote_ip.0), remote_ip.1);

        handshake.connect_to_peer(&mut stream, &local_socket, &potential_peer)
    }
}
