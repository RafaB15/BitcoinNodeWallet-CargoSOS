use cargosos_bitcoin::{
    configurations::connection_config::ConnectionConfig, logs::logger_sender::LoggerSender,
    node_structure::handshake::Handshake,
};

use std::net::{SocketAddr, TcpStream};

/// Creates a connection with the peers and if stablish then is return it's TCP stream
pub fn connect_to_peers(
    potential_peers: Vec<SocketAddr>,
    connection_config: ConnectionConfig,
    logger_sender: LoggerSender,
) -> Vec<TcpStream> {
    let _ = logger_sender.log_connection("Connecting to potential peers".to_string());

    let node = Handshake::new(
        connection_config.p2p_protocol_version,
        connection_config.services,
        connection_config.block_height,
        logger_sender.clone(),
    );

    let mut peer_streams: Vec<TcpStream> = Vec::new();

    for potential_peer in potential_peers {
        let mut peer_stream = match TcpStream::connect(potential_peer) {
            Ok(stream) => stream,
            Err(error) => {
                let _ = logger_sender.log_connection(format!(
                    "Cannot connect to address: {:?}, it appear {:?}",
                    potential_peer, error
                ));
                continue;
            }
        };

        let local_socket = match peer_stream.local_addr() {
            Ok(addr) => addr,
            Err(error) => {
                let _ = logger_sender
                    .log_connection(format!("Cannot get local address, it appear {:?}", error));
                continue;
            }
        };

        if let Err(error) =
            node.connect_to_testnet_peer(&mut peer_stream, &local_socket, &potential_peer)
        {
            let _ = logger_sender.log_connection(format!(
                "Error while connecting to addres: {:?}, it appear {:?}",
                potential_peer, error
            ));
            continue;
        };

        peer_streams.push(peer_stream);
    }

    peer_streams
}
