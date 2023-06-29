use cargosos_bitcoin::{
    configurations::server_config::ServerConfig,
    logs::logger_sender::LoggerSender,
    connections::error_connection::ErrorConnection,
};

use std::net::SocketAddr;

/// Get the peers from the dns seeder
///
/// ### Error
///  * `ErrorConnection::ErrorInvalidIPOrPortNumber`: It will appear if the IP or the port number its not valid
pub fn get_potential_peers(
    server_config: ServerConfig,
    logger: LoggerSender,
) -> Result<Vec<SocketAddr>, ErrorConnection> {
    let _ = logger.log_connection("Getting potential peers with dns seeder".to_string());

    let potential_peers = server_config.dns_seeder.discover_peers()?;

    let peer_count_max = std::cmp::min(server_config.peer_count_max, potential_peers.len());

    let potential_peers = potential_peers[0..peer_count_max].to_vec();

    for potential_peer in &potential_peers {
        let _ = logger.log_connection(format!("Potential peer: {:?}", potential_peer));
    }

    Ok(potential_peers)
}