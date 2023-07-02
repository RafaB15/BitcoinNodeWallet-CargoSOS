use super::{reference::{MutArc, get_reference}, download, broadcasting};

use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    configurations::{
        server_config::ServerConfig, connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig,
    }, 
    connections::error_connection::ErrorConnection,
    block_structure::block_chain::BlockChain,
    node_structure::{
        connection_event::ConnectionEvent, connection_type::ConnectionType, process_connection::ProcessConnection, 
        connection_id::ConnectionId, message_response::MessageResponse, broadcasting::Broadcasting, 
    },
    logs::logger_sender::LoggerSender,
    notifications::{notifier::Notifier},
};

use std::{
    net::{SocketAddr, IpAddr, TcpStream},
    thread::{self, JoinHandle},
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

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

pub fn create_process_connection<N: Notifier + Send + 'static>(
    connection_config: ConnectionConfig,
    notifier: N,
    logger: LoggerSender,
) -> (JoinHandle<()>, Receiver<(TcpStream, ConnectionId)>, Sender<ConnectionEvent>) {
    
    let (sender_potential_connections, receiver_potential_connections) =
        channel::<ConnectionEvent>();

    let (sender_confirm_connection, receiver_confirm_connection) =
        channel::<(TcpStream, ConnectionId)>();

    let process_connection = ProcessConnection::new(
        connection_config,
        sender_confirm_connection,
        receiver_potential_connections,
        notifier,
        logger,
    );

    let handle = thread::spawn(|| process_connection.execution());

    (handle, receiver_confirm_connection, sender_potential_connections)
}

pub fn update_from_connection<N: Notifier + Send + 'static>(
    receiver_confirm_connection: Receiver<(TcpStream, ConnectionId)>,
    sender_response: Sender<MessageResponse>,
    data: (MutArc<Broadcasting<TcpStream>>, MutArc<BlockChain>),
    config: (ConnectionConfig, DownloadConfig),
    notifier: N,
    logger: LoggerSender,
) -> JoinHandle<()> {

    let broadcasting = data.0;
    let block_chain = data.1;

    let magic_numbers = config.0.magic_numbers;
    
    thread::spawn(move || {

        for (stream, connection_id) in receiver_confirm_connection {

            let (stream, connection_id) = match connection_id.connection_type {
                ConnectionType::Peer => {
                    match download::update_block_chain_with_peer(
                        (stream, connection_id),
                        block_chain.clone(),
                        config.clone(),
                        notifier.clone(),
                        logger.clone(),
                    ) {
                        Ok(connection) => connection,
                        Err(error) => {
                            let _ = logger.log_connection(format!("Error while updating the block chain: {:?}", error));
                            continue;
                        }
                    }
                },
                ConnectionType::Client => (stream, connection_id),
            };

            let mut broadcasting_reference = match get_reference(&broadcasting) {
                Ok(broadcasting_reference) => broadcasting_reference,
                Err(error) => {
                    let _ = logger.log_connection(format!("Error: {:?}", error));
                    continue;
                }
            };

            if stream.set_read_timeout(Some(Duration::from_secs(1))).is_err() {
                let _  = logger.log_connection("Could not set timeout".to_string());
                continue;
            };

            broadcasting::add_peer_to_broadcasting(
                &mut broadcasting_reference,
                (stream, connection_id),
                sender_response.clone(),
                magic_numbers,
                notifier.clone(),
                logger.clone(),
            );
        }
    })
}

pub fn establish_connection(
    mode_config: ModeConfig,
    sender_potential_connections: Sender<ConnectionEvent>,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let potential_connections = match mode_config.clone() {
        ModeConfig::Server(server_config) => {

            let mut potential_connections = Vec::new();

            let peer_adresses = get_potential_peers(server_config.clone(), logger.clone())?;

            peer_adresses
                .iter()
                .for_each(|socket_address| {
                    potential_connections.push(
                        ConnectionId::new(socket_address.clone(), ConnectionType::Peer)
                    );
                });

            let port = server_config.own_port;

            for ip in server_config.address {
                let address = SocketAddr::new(IpAddr::V4(ip), port);

                potential_connections.push(ConnectionId::new(address, ConnectionType::Client));
            }

            potential_connections
        }
        ModeConfig::Client(client_config) => {
            let address = SocketAddr::new(IpAddr::V4(client_config.address), client_config.port);

            vec![ConnectionId::new(address, ConnectionType::Peer)]
        },
    };

    for potential_connection in potential_connections {
        if sender_potential_connections.send(ConnectionEvent::PotentialConnection(potential_connection)).is_err() {
            let _ = logger.log_connection("Could not send potential connection".to_string());
        }
    }

    Ok(())
}