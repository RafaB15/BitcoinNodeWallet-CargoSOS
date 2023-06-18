use super::user_response::{response_handle, user_input};

use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting::Broadcasting, download, handshake, load_system::LoadSystem,
        message_response::MessageResponse, save_system::SaveSystem,
    },
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    connections::ibd_methods::IBDMethod,
    logs::logger_sender::LoggerSender,
};

use std::{
    net::{SocketAddr, TcpStream},
    sync::mpsc::{self, Sender},
};

fn get_potential_peers(
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Result<Vec<SocketAddr>, ErrorExecution> {
    logger.log_connection("Getting potential peers with dns seeder".to_string())?;

    let potential_peers = connection_config.dns_seeder.discover_peers()?;

    let peer_count_max = std::cmp::min(connection_config.peer_count_max, potential_peers.len());

    let potential_peers = potential_peers[0..peer_count_max].to_vec();

    for potential_peer in &potential_peers {
        logger.log_connection(format!("Potential peer: {:?}", potential_peer))?;
    }

    Ok(potential_peers)
}

fn get_block_chain(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    logger: LoggerSender,
) -> Result<Vec<TcpStream>, ErrorExecution> {
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

fn get_utxo_set(block_chain: &BlockChain, logger: LoggerSender) -> UTXOSet {
    let _ = logger.log_wallet("Creating the UTXO set".to_string());

    let utxo_set = UTXOSet::from_blockchain(&block_chain);

    let _ = logger.log_wallet("UTXO set finished successfully".to_string());
    utxo_set
}

fn get_broadcasting(
    peer_streams: Vec<TcpStream>,
    sender_broadcasting: Sender<MessageResponse>,
    logger: LoggerSender,
) -> Result<Broadcasting<TcpStream>, ErrorExecution> {
    let _ = logger.log_node("Broadcasting".to_string());

    let boradcasting = Broadcasting::new(peer_streams, sender_broadcasting);

    Ok(boradcasting)
}

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
    let wallet = load_system.get_wallet()?;

    let peer_streams = get_block_chain(
        peer_streams,
        &mut block_chain,
        connection_config.clone(),
        download_config,
        logger.clone(),
    )?;

    let utxo_set = get_utxo_set(&block_chain, logger.clone());

    let (sender_broadcasting, receiver_broadcasting) = mpsc::channel::<MessageResponse>();

    let broadcasting = get_broadcasting(peer_streams, sender_broadcasting.clone(), logger.clone())?;

    let handle = response_handle(
        receiver_broadcasting,
        wallet,
        utxo_set,
        block_chain,
        logger.clone(),
    );

    user_input(sender_broadcasting, logger.clone())?;

    broadcasting.destroy()?;

    match handle.join() {
        Ok((block_chain, wallet)) => Ok(SaveSystem::new(block_chain, wallet, logger)),
        Err(_) => todo!(),
    }
}
