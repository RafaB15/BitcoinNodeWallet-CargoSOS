use super::{
    broadcasting, download,
    reference::{get_reference, MutArc},
};

use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig, server_config::ServerConfig,
    },
    connections::error_connection::ErrorConnection,
    concurrency::{stop::Stop, listener::Listener},
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting,
        connection_event::ConnectionEvent,
        connection_id::ConnectionId,
        connection_type::ConnectionType,
        error_node::ErrorNode,
        message_response::MessageResponse,
        process_connection::{ProcessConnection, SenderPotential},
    },
    notifications::notifier::Notifier,
};

use std::{
    net::{IpAddr, SocketAddr, TcpStream, TcpListener},
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
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

/// Crates the thread to manega the potential connections to establish a connection via a handshake
pub fn create_process_connection<N: Notifier + Send + 'static>(
    connection_config: ConnectionConfig,
    sender_confirm_connection: Sender<(TcpStream, ConnectionId)>,
    notifier: N,
    logger: LoggerSender,
) -> (
    JoinHandle<Result<(), ErrorNode>>,
    SenderPotential,
) {
    let (sender_potential_connections, receiver_potential_connections) =
        channel::<ConnectionEvent>();

    let process_connection = ProcessConnection::new(
        connection_config,
        sender_confirm_connection,
        receiver_potential_connections,
        notifier,
        logger,
    );

    let handle = thread::spawn(|| process_connection.execution());

    (
        handle,
        sender_potential_connections,
    )
}

/// Creates a thread to manage the confirmed connections and update the block chain if the connection is a peer
pub fn update_from_connection<N: Notifier + Send + 'static>(
    receiver_confirm_connection: Receiver<(TcpStream, ConnectionId)>,
    sender_response: Sender<MessageResponse>,
    data: (
        MutArc<Broadcasting<TcpStream>>,
        MutArc<BlockChain>,
        MutArc<UTXOSet>,
    ),
    config: (ConnectionConfig, DownloadConfig),
    notifier: N,
    logger: LoggerSender,
) -> JoinHandle<()> {
    let broadcasting = data.0;
    let block_chain = data.1;
    let utxo_set = data.2;

    let magic_numbers = config.0.magic_numbers;

    thread::spawn(move || {
        for (stream, connection_id) in receiver_confirm_connection {
            let (stream, connection_id) = match connection_id.connection_type {
                ConnectionType::Peer => {
                    match download::update_block_chain_with_peer(
                        (stream, connection_id),
                        block_chain.clone(),
                        utxo_set.clone(),
                        config.clone(),
                        notifier.clone(),
                        logger.clone(),
                    ) {
                        Ok(connection) => connection,
                        Err(error) => {
                            let _ = logger.log_connection(format!(
                                "Error while updating the block chain: {:?}",
                                error
                            ));
                            continue;
                        }
                    }
                }
                ConnectionType::Client => (stream, connection_id),
            };

            let mut broadcasting_reference = match get_reference(&broadcasting) {
                Ok(broadcasting_reference) => broadcasting_reference,
                Err(error) => {
                    let _ = logger.log_connection(format!("Error: {:?}", error));
                    continue;
                }
            };

            if stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .is_err()
            {
                let _ = logger.log_connection("Could not set timeout".to_string());
                continue;
            };

            broadcasting::add_peer_to_broadcasting(
                &mut broadcasting_reference,
                (stream, connection_id),
                sender_response.clone(),
                block_chain.clone(),
                magic_numbers,
                notifier.clone(),
                logger.clone(),
            );
        }
    })
}

/// Establish the connection with the peers and the clients
pub fn establish_connection_to_peers(
    mode_config: ModeConfig,
    sender_potential_connections: Sender<ConnectionEvent>,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let potential_sockets = match mode_config.clone() {
        ModeConfig::Server(server_config) => {
            get_potential_peers(server_config.clone(), logger.clone())?
        }
        ModeConfig::Client(client_config) => {
            vec![SocketAddr::new(IpAddr::V4(client_config.address), client_config.port)]
        }
    };

    for potential_socket in potential_sockets {
        if sender_potential_connections
            .send(ConnectionEvent::PotentialPeer(potential_socket))
            .is_err()
        {
            let _ = logger.log_connection("Could not send potential connection".to_string());
        }
    }

    Ok(())
}

pub fn establish_connection_with_clients(
    server_config: ServerConfig,
    receiver_stop: Receiver<Stop>,
    sender_potential_connections: Sender<ConnectionEvent>,
    logger: LoggerSender,
) -> Option<JoinHandle<()>> {

    let mut listener = match TcpListener::bind(SocketAddr::new(
        IpAddr::V4(server_config.address),
        server_config.own_port,
    )) {
        Ok(listener) => listener,
        Err(_) => {
            let _ = logger.log_error("Could not bind port".to_string());
            return None;
        },
    }; 

    if listener.set_nonblocking(true).is_err() {
        let _ = logger.log_error("Could not set non blocking".to_string());
        return None;
    }

    let handle = thread::spawn(move || {

        loop {
            match Listener::listen(&mut listener, &receiver_stop) {
                Listener::Stream(stream, socket_address) => {
                    if sender_potential_connections.send(
                        ConnectionEvent::PotentialClient(stream, socket_address)
                    ).is_err() {
                        let _ = logger.log_error("Could not send client to connect".to_string());
                    }
                },
                Listener::Information(_) => {},
                Listener::Stop => break,
            }
        }

    });

    Some(handle)
}