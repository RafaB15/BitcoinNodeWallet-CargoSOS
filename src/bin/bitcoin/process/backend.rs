use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting, connection, download, error_process::ErrorProcess, load_system::LoadSystem,
        reference, reference::MutArc, save_system::SaveSystem,
    },
    ui::{error_ui::ErrorUI, input_handler::InputHandler},
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    concurrency::stop::Stop,
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig,
    },
    logs::{level::Level, logger_sender::LoggerSender},
    node_structure::{
        broadcasting::Broadcasting, connection_event::ConnectionEvent, connection_id::ConnectionId,
        message_response::MessageResponse,
    },
    notifications::{notification::Notification, notifier::Notifier},
    wallet_structure::wallet::Wallet,
};

use std::{
    net::TcpStream,
    sync::mpsc::{channel, Receiver},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

type HandlePeer = JoinHandle<Result<(), ErrorProcess>>;

/// The main function of the program for the terminal
///
/// ### Error
///  * `ErrorExecution::FailThread`: It will appear when the thread fails
///  * `ErrorUI::CannotGetInner`: It will appear when we try to get the inner value of a mutex
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `UI::ErrorFromPeer`: It will appear when a conextion with a peer fails
///  * `ErrorProcess:CannotCreateDefault`: It will appear when can't create the default value
///  * `ErrorProcess:AlreadyLoaded`: It will appear when try to get a value that is already loadedError
pub fn backend<N, I>(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    input_handler: I,
    notifier: N,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution>
where
    I: InputHandler<TcpStream>,
    N: Notifier + 'static,
{
    let (sender_confirm_connection, receiver_confirm_connection) =
        channel::<(TcpStream, ConnectionId)>();

    let (handle_process_connection, sender_potential_connections) =
        connection::create_process_connection(
            connection_config.clone(),
            sender_confirm_connection,
            notifier.clone(),
            logger.clone(),
        );

    let wallet = load_system.get_wallet()?;

    for account in wallet.get_accounts().iter() {
        notifier.notify(Notification::RegisterWalletAccount(account.clone()));
    }

    let wallet = Arc::new(Mutex::new(wallet));

    let block_chain = load_system.get_block_chain()?;

    let utxo_set = Arc::new(Mutex::new(download::get_utxo_set(
        &block_chain,
        logger.clone(),
    )));

    let block_chain = Arc::new(Mutex::new(block_chain));

    notifier.notify(Notification::NotifyBlockchainIsReady);

    let (sender_response, receiver_response) = channel::<MessageResponse>();

    let (handle_peers, broadcasting) = broadcasting(
        (wallet.clone(), utxo_set.clone(), block_chain.clone()),
        receiver_response,
        notifier.clone(),
        logger.clone(),
    );

    let handle_confirmed_connection = connection::update_from_connection(
        receiver_confirm_connection,
        sender_response,
        (broadcasting.clone(), block_chain.clone(), utxo_set.clone()),
        (connection_config, download_config),
        notifier.clone(),
        logger.clone(),
    );

    connection::establish_connection_to_peers(
        mode_config.clone(),
        sender_potential_connections.clone(),
        logger.clone(),
    )?;

    let (sender_stop, receiver_stop) = channel::<Stop>();

    let posible_handle = match mode_config {
        ModeConfig::Server(server_config) => connection::establish_connection_with_clients(
            server_config,
            receiver_stop,
            sender_potential_connections.clone(),
            logger.clone(),
        ),
        ModeConfig::Client(_) => None,
    };

    input_handler.handle_input(
        broadcasting.clone(),
        wallet.clone(),
        utxo_set,
        block_chain.clone(),
    )?;

    if let Some(handle) = posible_handle {
        if sender_stop.send(Stop::Stop).is_err() {
            let _ = logger.log_data(
                Level::ERROR,
                ErrorUI::ErrorFromPeer("Fail to stop potential connections".to_string()),
            );
        } else if handle.join().is_err() {
            let _ = logger.log_data(
                Level::ERROR,
                ErrorUI::ErrorFromPeer("Fail to close confirmed connections".to_string()),
            );
        }
    }

    if sender_potential_connections
        .send(ConnectionEvent::Stop)
        .is_err()
    {
        let _ = logger.log_data(
            Level::ERROR,
            ErrorUI::ErrorFromPeer("Failed to stop potential connections".to_string()),
        );
    } else {
        match handle_process_connection.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let _ = logger.log_data(Level::ERROR, error);
            }
            Err(_) => {
                let _ = logger.log_data(
                    Level::ERROR,
                    ErrorUI::ErrorFromPeer("Failed to close confirmed connections".to_string()),
                );
            }
        }

        if handle_confirmed_connection.join().is_err() {
            let _ = logger.log_data(
                Level::ERROR,
                ErrorUI::ErrorFromPeer("Failed to close confirmed connections".to_string()),
            );
        }
    }

    if reference::get_reference(&broadcasting)?
        .close_connections(notifier)
        .is_ok()
        && handle_peers.join().is_err()
    {
        let _ = logger.log_data(
            Level::ERROR,
            ErrorUI::ErrorFromPeer("Failed to remove notifications".to_string()),
        );
    }

    Ok(SaveSystem::new(
        reference::get_inner(block_chain)?,
        reference::get_inner(wallet)?,
        logger,
    ))
}

/// Broadcasting blocks and transactions from and to the given peers
fn broadcasting<N: Notifier + 'static>(
    data: (MutArc<Wallet>, MutArc<UTXOSet>, MutArc<BlockChain>),
    receiver_response: Receiver<MessageResponse>,
    notifier: N,
    logger: LoggerSender,
) -> (HandlePeer, MutArc<Broadcasting<TcpStream>>) {
    let wallet: Arc<Mutex<Wallet>> = data.0;
    let utxo_set: Arc<Mutex<UTXOSet>> = data.1;
    let block_chain: Arc<Mutex<BlockChain>> = data.2;

    let broadcasting = Broadcasting::<TcpStream>::new(logger.clone());
    let broadcasting = Arc::new(Mutex::new(broadcasting));

    let handle = broadcasting::handle_peers(
        receiver_response,
        broadcasting.clone(),
        wallet,
        utxo_set,
        block_chain,
        notifier,
        logger,
    );

    (handle, broadcasting)
}
