use super::{backend::user_input, notifier_tui::NotifierTUI};

use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting, connection, download, load_system::LoadSystem, reference,
        reference::{MutArc, get_reference}, save_system::SaveSystem, error_process::ErrorProcess,
    },
    ui::error_ui::ErrorUI,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig,
    },
    logs::{logger_sender::LoggerSender},
    node_structure::{
        broadcasting::Broadcasting, connection_id::ConnectionId, message_response::MessageResponse,
        connection_type::ConnectionType, connection_event::ConnectionEvent, process_connection::ProcessConnection,
    },
    notifications::notifier::Notifier,
    wallet_structure::wallet::Wallet,
};

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    thread::{self, JoinHandle},
    sync::mpsc::{channel, Receiver, Sender},
    sync::{Arc, Mutex},
    time::Duration,
};

fn _show_merkle_path(block_chain: &BlockChain, logger: LoggerSender) -> Result<(), ErrorExecution> {
    let latest = block_chain.latest();

    let last_block = match latest.last() {
        Some(last_block) => last_block,
        None => {
            return Err(ErrorExecution::_ErrorBlock(
                "Last block not found".to_string(),
            ))
        }
    };

    logger.log_connection(format!(
        "With the block with header: \n{:?}",
        last_block.header,
    ))?;

    let transaction_position =
        std::cmp::min::<u64>(6, last_block.header.transaction_count.value - 1);

    let transaction = match last_block.transactions.get(transaction_position as usize) {
        Some(transaction) => transaction,
        None => {
            return Err(ErrorExecution::_ErrorBlock(
                "Transaction not found".to_string(),
            ))
        }
    };

    logger.log_connection(format!("And transaction: \n{:?}", transaction,))?;

    let merkle_path = last_block.get_merkle_path(transaction)?;

    let mut path: String = "\n".to_string();
    for hash in merkle_path {
        path = format!("{path}\t{:?}\n", hash);
    }

    logger.log_connection(format!("We get the merkle path: {path}"))?;

    Ok(())
}


/// Broadcasting blocks and transactions from and to the given peers
///
/// ### Error
///  *
fn broadcasting<N: Notifier + 'static>(
    data: (MutArc<Wallet>, MutArc<UTXOSet>, MutArc<BlockChain>),
    notifier: N,
    logger: LoggerSender,
) -> Result<(JoinHandle<Result<(), ErrorProcess>>, Broadcasting::<TcpStream>, Sender<MessageResponse>), ErrorExecution> {
    let wallet: Arc<Mutex<Wallet>> = data.0;
    let utxo_set: Arc<Mutex<UTXOSet>> = data.1;
    let block_chain: Arc<Mutex<BlockChain>> = data.2;

    let (sender_response, receiver_response) = channel::<MessageResponse>();

    let handle = broadcasting::handle_peers(
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
        block_chain.clone(),
        notifier.clone(),
        logger.clone(),
    );

    Ok((handle, Broadcasting::<TcpStream>::new(logger.clone()), sender_response))
}


fn create_process_connection<N: Notifier + Send + 'static>(
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

fn update_block_chain_with_peer<N: Notifier + Send + 'static>(
    connection: (TcpStream, ConnectionId),
    block_chain: MutArc<BlockChain>,
    config: (ConnectionConfig, DownloadConfig),
    notifier: N,
    logger: LoggerSender,
) -> Result<(TcpStream, ConnectionId), ErrorExecution> {

    let connection_config = config.0;
    let download_config = config.1;

    let mut block_chain_reference = get_reference(&block_chain)?;

    let connection = download::update_block_chain(
        connection,
        &mut block_chain_reference,
        connection_config.clone(),
        download_config,
        notifier.clone(),
        logger.clone(),
    )?;

    Ok(connection)
}

fn update_from_connection<N: Notifier + Send + 'static>(
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

            let mut connection = (stream, connection_id);

            if connection_id.connection_type == ConnectionType::Peer {
                connection = match update_block_chain_with_peer(
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
            }

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
                connection,
                sender_response,
                magic_numbers,
                notifier.clone(),
                logger.clone(),
            );
        }
    })
}

fn establish_connection(
    mode_config: ModeConfig,
    sender_potential_connections: Sender<ConnectionEvent>,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let potential_connections = match mode_config.clone() {
        ModeConfig::Server(server_config) => {

            let mut potential_connections = Vec::new();

            let peer_adresses = connection::get_potential_peers(server_config, logger.clone())?;

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

                potential_connections.push(ConnectionId::new(address, ConnectionType::Peer));
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

/// The main function of the program for the terminal
pub fn program_execution(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {

    let notifier = NotifierTUI::new(logger.clone());

    let (handle_process_connection, receiver_confirm_connection, sender_potential_connections) = create_process_connection(
        connection_config.clone(),
        notifier.clone(),
        logger.clone(),
    );

    let mut block_chain = load_system.get_block_chain()?;

    let wallet = Arc::new(Mutex::new(load_system.get_wallet()?));

    let utxo_set = Arc::new(Mutex::new(download::get_utxo_set(
        &block_chain,
        logger.clone(),
    )));

    let block_chain = Arc::new(Mutex::new(block_chain));

    let (handle_peers, broadcasting, sender_response) = broadcasting(
        (wallet.clone(), utxo_set, block_chain.clone()),
        notifier.clone(),
        logger.clone(),
    )?;

    let broadcasting = Arc::new(Mutex::new(broadcasting));

    let handle_confirmed_connection = update_from_connection(
        receiver_confirm_connection,
        sender_response,
        (broadcasting.clone(), block_chain.clone()),
        (connection_config.clone(), download_config.clone()),
        notifier.clone(),
        logger.clone(),
    );

    establish_connection(
        mode_config.clone(),
        sender_potential_connections,
        logger.clone(),
    )?;

    user_input(
        broadcasting.clone(),
        wallet,
        utxo_set,
        block_chain,
        notifier.clone(),
        logger,
    )?;   

    reference::get_inner(broadcasting)?.destroy(notifier)?;

    if handle_peers.join().is_err() {
        return Err(ErrorUI::ErrorFromPeer("Fail to remove notifications".to_string()).into());
    }

    Ok(SaveSystem::new(
        reference::get_inner(block_chain)?,
        reference::get_inner(wallet)?,
        logger,
    ))
}
