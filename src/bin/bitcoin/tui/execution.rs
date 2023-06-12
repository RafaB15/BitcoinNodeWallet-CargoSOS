use crate::{
    error_execution::ErrorExecution,
    process::{
        download, handshake, account,
        save_system::SaveSystem, 
        load_system::LoadSystem, 
    },
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig
};

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    block_structure::{
        block_chain::BlockChain,
        utxo_set::UTXOSet,
    },
    connections::ibd_methods::IBDMethod,
};

use std::net::{SocketAddr, TcpStream};

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
) -> Result<(), ErrorExecution> {
    logger.log_connection("Getting block chain".to_string())?;

    match connection_config.ibd_method {
        IBDMethod::HeaderFirst => {
            download::headers_first(
                peer_streams,
                block_chain,
                connection_config,
                download_config,
                logger,
            )?;
        }
        IBDMethod::BlocksFirst => {
            download::blocks_first();
        }
    }

    Ok(())
}

fn _show_merkle_path(
    block_chain: &BlockChain,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let latest = block_chain.latest();

    let last_block = match latest.last() {
        Some(last_block) => last_block,
        None => {
            return Err(ErrorExecution::ErrorBlock(
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
            return Err(ErrorExecution::ErrorBlock(
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

pub fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {
    let potential_peers = get_potential_peers(connection_config.clone(), logger.clone())?;

    let peer_streams = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        logger.clone(),
    );

    let mut block_chain = load_system.get_block_chain()?;
    let mut wallet = load_system.get_wallet()?;

    println!("Wallet: {:?}", wallet);

    get_block_chain(
        peer_streams,
        &mut block_chain,
        connection_config,
        download_config,
        logger.clone(),
    )?;

    // show_merkle_path(&block_chain, logger_sender.clone())?; 

    while account::wants_to_enter_account()? {
        let new_account = account::add_account(logger.clone())?;
        wallet.add_account(new_account);
    }

    let utxo_set = UTXOSet::from_blockchain(&block_chain);

    for account in wallet.accounts.iter() {
        print!("Account's {} utxo: {:?}\n", account.account_name, utxo_set.get_utxo_list(&Some(account.address.clone())));
    }

    Ok(SaveSystem::new(
        block_chain,
        wallet,
        logger,
    ))
}