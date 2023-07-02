use super::{backend::user_input, notifier_tui::NotifierTUI};

use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting, connection, download, load_system::LoadSystem, reference,
        reference::{MutArc}, save_system::SaveSystem, error_process::ErrorProcess,
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
        broadcasting::Broadcasting, message_response::MessageResponse, connection_event::ConnectionEvent,
    },
    notifications::notifier::Notifier,
    wallet_structure::wallet::Wallet,
};

use std::{
    net::TcpStream,
    thread::JoinHandle,
    sync::mpsc::{channel, Sender},
    sync::{Arc, Mutex},
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

/// The main function of the program for the terminal
pub fn program_execution(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {

    let notifier = NotifierTUI::new(logger.clone());

    let (handle_process_connection, 
        receiver_confirm_connection, 
        sender_potential_connections
    ) = connection::create_process_connection(
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

    let (handle_peers, 
        broadcasting, 
        sender_response
    ) = broadcasting(
        (wallet.clone(), utxo_set, block_chain.clone()),
        notifier.clone(),
        logger.clone(),
    )?;

    let broadcasting = Arc::new(Mutex::new(broadcasting));

    let handle_confirmed_connection = connection::update_from_connection(
        receiver_confirm_connection,
        sender_response,
        (broadcasting.clone(), block_chain.clone()),
        (connection_config.clone(), download_config.clone()),
        notifier.clone(),
        logger.clone(),
    );

    connection::establish_connection(
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
        logger.clone(),
    )?;

    if sender_potential_connections.send(ConnectionEvent::Stop).is_err() {
        return Err(ErrorUI::ErrorFromPeer("Fail to stop potential connections".to_string()).into());
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
