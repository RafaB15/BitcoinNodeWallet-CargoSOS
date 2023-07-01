use cargosos_bitcoin::{
    configurations::connection_config::ConnectionConfig,
    logs::logger_sender::LoggerSender,
    node_structure::{
        connection_id::ConnectionId, connection_type::ConnectionType, handshake::Handshake,
        handshake_data::HandshakeData,
    },
    notifications::{notification::Notification, notifier::Notifier},
};

use std::{
    net::TcpStream,
};

/// Creates a connection with the peers and if established then is return it's TCP stream
pub fn connect_to_peers<N: Notifier>(
    potential_connections: Vec<ConnectionId>,
    connection_config: ConnectionConfig,
    notifier: N,
    logger_sender: LoggerSender,
) -> Vec<(TcpStream, ConnectionId)> {
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

    potential_connections
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

/// Creates a connection with a specific peer and if established then is return it's TCP stream
fn filters_peer<N: Notifier>(
    potential_connection: ConnectionId,
    node: &Handshake,
    logger_sender: LoggerSender,
    notifier: N,
) -> Option<(TcpStream, ConnectionId)> {
    let mut peer_stream = match TcpStream::connect(potential_connection.address) {
        Ok(stream) => stream,
        Err(error) => {
            let _ = logger_sender.log_connection(format!(
                "Cannot connect to address: {:?}, it appear {:?}",
                potential_connection, error
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

    notifier.notify(Notification::AttemptingHandshakeWithPeer(
        potential_connection.address.clone(),
    ));

    let result = match potential_connection {
        ConnectionId { address, connection_type: ConnectionType::Peer } => {
            node.connect_to_peer(&mut peer_stream, &local_socket, &address)
        }, 
        ConnectionId { address, connection_type: ConnectionType::Client } => {
            node.connect_to_client(&mut peer_stream, &local_socket, &address)
        }, 
    };

    match result {
        Ok(_) => {
            notifier.notify(Notification::SuccessfulHandshakeWithPeer(potential_connection.address));
            Some((peer_stream, potential_connection))
        }
        Err(error) => {
            let _ = logger_sender.log_connection(format!(
                "Error while connecting to addres: {:?}, it appear {:?}",
                potential_connection, error
            ));
            notifier.notify(Notification::FailedHandshakeWithPeer(potential_connection.address));
            None
        }
    }
}
