use cargosos_bitcoin::{
    configurations::connection_config::ConnectionConfig,
    logs::logger_sender::LoggerSender,
    node_structure::{handshake::Handshake, handshake_data::HandshakeData},
    notifications::notification::{Notification, NotificationSender},
};

use std::net::{SocketAddr, TcpStream};

/// Creates a connection with the peers and if stablish then is return it's TCP stream
pub fn connect_to_peers(
    potential_peers: Vec<SocketAddr>,
    connection_config: ConnectionConfig,
    logger_sender: LoggerSender,
    notifier: NotificationSender,
) -> Vec<TcpStream> {
    let _ = logger_sender.log_connection("Connecting to potential peers".to_string());

    let node = Handshake::new(
        connection_config.p2p_protocol_version,
        connection_config.services,
        connection_config.block_height,
        HandshakeData {
            nonce: connection_config.nonce,
            user_agent: connection_config.user_agent,
            relay: connection_config.relay,
            magic_number: connection_config.magic_numbers,
        },
        logger_sender.clone(),
    );

    potential_peers
        .iter()
        .filter_map(|potential_peer| {
            filters_peer(
                *potential_peer,
                &node,
                logger_sender.clone(),
                notifier.clone(),
            )
        })
        .collect()
}

/// Creates a connection with a specific peer and if stablish then is return it's TCP stream
fn filters_peer(
    potential_peer: SocketAddr,
    node: &Handshake,
    logger_sender: LoggerSender,
    notifier: NotificationSender,
) -> Option<TcpStream> {
    let mut peer_stream = match TcpStream::connect(potential_peer) {
        Ok(stream) => stream,
        Err(error) => {
            let _ = logger_sender.log_connection(format!(
                "Cannot connect to address: {:?}, it appear {:?}",
                potential_peer, error
            ));
            return None;
        }
    };

    let local_socket = match peer_stream.local_addr() {
        Ok(addr) => addr,
        Err(error) => {
            let _ = logger_sender
                .log_connection(format!("Cannot get local address, it appear {:?}", error));
            return None;
        }
    };

    let _ = notifier.send(Notification::AttemptingHandshakeWithPeer(
        potential_peer.clone(),
    ));
    match node.connect_to_peer(&mut peer_stream, &local_socket, &potential_peer) {
        Ok(_) => {
            let _ = notifier.send(Notification::SuccessfulHandshakeWithPeer(
                potential_peer.clone(),
            ));
            Some(peer_stream)
        }
        Err(error) => {
            let _ = notifier.send(Notification::FailedHandshakeWithPeer(
                potential_peer.clone(),
            ));
            let _ = logger_sender.log_connection(format!(
                "Error while connecting to addres: {:?}, it appear {:?}",
                potential_peer, error
            ));
            None
        }
    }
}
