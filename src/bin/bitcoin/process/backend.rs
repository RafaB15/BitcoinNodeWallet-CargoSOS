use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting, connection, download, load_system::LoadSystem, reference,
        reference::{MutArc}, save_system::SaveSystem, error_process::ErrorProcess,
    },
    ui::{error_ui::ErrorUI, input_handler::InputHandler},
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig,
    },
    logs::{logger_sender::LoggerSender},
    node_structure::{
        broadcasting::Broadcasting, message_response::MessageResponse, connection_event::ConnectionEvent,
    },
    notifications::{notifier::Notifier, notification::Notification},
    wallet_structure::wallet::Wallet,
};

use std::{
    net::TcpStream,
    thread::JoinHandle,
    sync::mpsc::{channel, Sender},
    sync::{Arc, Mutex},
};

/// The main function of the program for the terminal
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
    let (handle_process_connection, 
        receiver_confirm_connection, 
        sender_potential_connections
    ) = connection::create_process_connection(
        connection_config.clone(),
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

    let (handle_peers, 
        broadcasting, 
        sender_response
    ) = broadcasting(
        (wallet.clone(), utxo_set.clone(), block_chain.clone()),
        notifier.clone(),
        logger.clone(),
    )?;

    let broadcasting = Arc::new(Mutex::new(broadcasting));

    let handle_confirmed_connection = connection::update_from_connection(
        receiver_confirm_connection,
        sender_response,
        (broadcasting.clone(), block_chain.clone(), utxo_set.clone()),
        (connection_config.clone(), download_config.clone()),
        notifier.clone(),
        logger.clone(),
    );

    connection::establish_connection(
        mode_config.clone(),
        sender_potential_connections.clone(),
        logger.clone(),
    )?;

    input_handler.handle_input(
        broadcasting.clone(), 
        wallet.clone(), 
        utxo_set, 
        block_chain.clone(),
    )?;

    if sender_potential_connections.send(ConnectionEvent::Stop).is_err() {
        return Err(ErrorUI::ErrorFromPeer("Fail to stop potential connections".to_string()).into());
    }

    if handle_process_connection.join().is_err() {
        return Err(ErrorUI::ErrorFromPeer("Fail to close confirmed connections".to_string()).into());
    }

    if handle_confirmed_connection.join().is_err() {
        return Err(ErrorUI::ErrorFromPeer("Fail to close confirmed connections".to_string()).into());
    }    

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