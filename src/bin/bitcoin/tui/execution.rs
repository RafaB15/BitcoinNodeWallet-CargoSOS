use super::{backend::user_input, notifier_tui::NotifierTUI};

use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting, connection, download, handshake, load_system::LoadSystem, reference,
        reference::MutArc, save_system::SaveSystem,
    },
    ui::error_ui::ErrorUI,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig,
    },
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting, connection_id::ConnectionId, message_response::MessageResponse,
    },
    notifications::notifier::Notifier,
    wallet_structure::wallet::Wallet,
};

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    sync::mpsc,
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
    connections: Vec<(TcpStream, ConnectionId)>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    connection_config: ConnectionConfig,
    notifier: N,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let (sender_response, receiver_response) = mpsc::channel::<MessageResponse>();

    let handle = broadcasting::handle_peers(
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
        block_chain.clone(),
        notifier.clone(),
        logger.clone(),
    );

    let mut broadcasting = Broadcasting::<TcpStream>::new(logger.clone());

    broadcasting::add_peers(
        &mut broadcasting,
        connections,
        sender_response,
        connection_config.magic_numbers,
        notifier.clone(),
        logger.clone(),
    );

    user_input(
        &mut broadcasting,
        wallet,
        utxo_set,
        block_chain,
        notifier.clone(),
        logger,
    )?;

    broadcasting.destroy(notifier)?;

    match handle.join() {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrorUI::ErrorFromPeer("Fail to remove notifications".to_string()).into()),
    }
}

/// The main function of the program for the terminal
pub fn program_execution(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {
    let potential_peers = match mode_config {
        ModeConfig::Server(server_config) => {
            connection::get_potential_peers(server_config, logger.clone())?
        }
        ModeConfig::Client(client_config) => vec![SocketAddr::new(
            IpAddr::V4(client_config.address),
            client_config.port,
        )],
    };

    let notifier = NotifierTUI::new(logger.clone());

    let connections = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        notifier.clone(),
        logger.clone(),
    );

    let mut block_chain = load_system.get_block_chain()?;

    let connections = download::update_block_chain(
        connections,
        &mut block_chain,
        connection_config.clone(),
        download_config,
        notifier.clone(),
        logger.clone(),
    )?;

    let wallet = Arc::new(Mutex::new(load_system.get_wallet()?));
    let utxo_set = Arc::new(Mutex::new(download::get_utxo_set(
        &block_chain,
        logger.clone(),
    )));
    let block_chain = Arc::new(Mutex::new(block_chain));

    broadcasting(
        connections,
        wallet.clone(),
        utxo_set,
        block_chain.clone(),
        connection_config,
        notifier.clone(),
        logger.clone(),
    )?;

    Ok(SaveSystem::new(
        reference::get_inner(block_chain)?,
        reference::get_inner(wallet)?,
        logger,
    ))
}
