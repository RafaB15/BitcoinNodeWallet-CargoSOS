use super::{
    error_tui::ErrorTUI,
    user_response::{handle_peers, user_input},
};

use crate::{
    error_execution::ErrorExecution,
    process::{download, handshake, load_system::LoadSystem, save_system::SaveSystem},
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    connections::ibd_methods::IBDMethod,
    logs::logger_sender::LoggerSender,
    node_structure::{broadcasting::Broadcasting, message_response::MessageResponse},
    wallet_structure::wallet::Wallet,
};

use std::{
    net::{SocketAddr, TcpStream},
    sync::mpsc::{self, Sender},
    sync::{Arc, Mutex},
};

/// Get the peers from the dns seeder
///
/// ### Error
///  * `ErrorTUI::ErrorFromPeer`: It will appear when a conextion with a peer fails
fn get_potential_peers(
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Result<Vec<SocketAddr>, ErrorTUI> {
    let _ = logger.log_connection("Getting potential peers with dns seeder".to_string());

    let potential_peers = match connection_config.dns_seeder.discover_peers() {
        Ok(potential_peers) => potential_peers,
        Err(_) => {
            return Err(ErrorTUI::ErrorFromPeer(
                "Fail to getting potencial peers".to_string(),
            ))
        }
    };

    let peer_count_max = std::cmp::min(connection_config.peer_count_max, potential_peers.len());

    let potential_peers = potential_peers[0..peer_count_max].to_vec();

    for potential_peer in &potential_peers {
        let _ = logger.log_connection(format!("Potential peer: {:?}", potential_peer));
    }

    Ok(potential_peers)
}

/// Update the block chain given a downloading method and a list of peers
///
/// ### Error
///  * `ErrorMessage::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
///  * `ErrorNode::NodeNotResponding`: It will appear when
///  * `ErrorNode::WhileValidating`: It will appear when
///  * `ErrorBlock::CouldNotUpdate`: It will appear when the block is not in the blockchain.
///  * `ErrorProcess::FailThread`: It will appear when a thread panics and fails
fn update_block_chain(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    logger: LoggerSender,
) -> Result<Vec<TcpStream>, ErrorTUI> {
    let _ = logger.log_connection("Getting block chain".to_string());

    Ok(match connection_config.ibd_method {
        IBDMethod::HeaderFirst => download::headers_first(
            peer_streams,
            block_chain,
            connection_config,
            download_config,
            logger,
        )?,
        IBDMethod::BlocksFirst => download::blocks_first(),
    })
}

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

/// Creates the UTXO set from the given block chain
fn get_utxo_set(block_chain: &BlockChain, logger: LoggerSender) -> UTXOSet {
    let _ = logger.log_wallet("Creating the UTXO set".to_string());

    let utxo_set = UTXOSet::from_blockchain(&block_chain);

    let _ = logger.log_wallet("UTXO set finished successfully".to_string());
    utxo_set
}

/// Creates the broadcasting
fn get_broadcasting(
    peer_streams: Vec<TcpStream>,
    sender_response: Sender<MessageResponse>,
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Broadcasting<TcpStream> {
    let _ = logger.log_node("Broadcasting".to_string());
    Broadcasting::new(peer_streams, sender_response, connection_config, logger)
}

/// Get the value of a mutable reference given by Arc<Mutex<T>>
///
/// ### Error
///  * `ErrorTUI::CannotGetInner`: It will appear when we try to get the inner value of a mutex
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn get_inner<T>(reference: Arc<Mutex<T>>) -> Result<T, ErrorTUI> {
    match Arc::try_unwrap(reference) {
        Ok(reference_unwrap) => match reference_unwrap.into_inner() {
            Ok(reference) => Ok(reference),
            Err(_) => Err(ErrorTUI::CannotGetInner),
        },
        Err(_) => Err(ErrorTUI::CannotUnwrapArc),
    }
}

/// Broadcasting blocks and transactions from and to the given peers
/// 
/// ### Error
///  * 
fn broadcasting(
    peer_streams: Vec<TcpStream>,
    wallet: Arc<Mutex<Wallet>>,
    utxo_set: Arc<Mutex<UTXOSet>>,
    block_chain: Arc<Mutex<BlockChain>>,
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let (sender_response, receiver_response) = mpsc::channel::<MessageResponse>();

    let handle = handle_peers(
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
        block_chain.clone(),
        logger.clone(),
    );

    let mut broadcasting = get_broadcasting(
        peer_streams,
        sender_response,
        connection_config,
        logger.clone(),
    );

    user_input(
        &mut broadcasting,
        wallet.clone(),
        utxo_set,
        block_chain.clone(),
        logger.clone(),
    )?;

    broadcasting.destroy()?;

    match handle.join() {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrorTUI::ErrorFromPeer("Fail to remove notifications".to_string()).into()),
    }
}

/// The main function of the program for the terminal
pub fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {
    let potential_peers = get_potential_peers(connection_config.clone(), logger.clone())?;

    let peer_streams =
        handshake::connect_to_peers(potential_peers, connection_config.clone(), logger.clone());

    let mut block_chain = load_system.get_block_chain()?;

    let peer_streams = update_block_chain(
        peer_streams,
        &mut block_chain,
        connection_config.clone(),
        download_config,
        logger.clone(),
    )?;

    let wallet = Arc::new(Mutex::new(load_system.get_wallet()?));
    let utxo_set = Arc::new(Mutex::new(get_utxo_set(&block_chain, logger.clone())));
    let block_chain = Arc::new(Mutex::new(block_chain));

    broadcasting(
        peer_streams,
        wallet.clone(),
        utxo_set,
        block_chain.clone(),
        connection_config,
        logger.clone(),
    )?;

    Ok(SaveSystem::new(
        get_inner(block_chain)?,
        get_inner(wallet)?,
        logger,
    ))
}
