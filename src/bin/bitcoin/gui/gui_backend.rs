use super::{
    signal_to_front::SignalToFront,
    signal_to_back::SignalToBack,
};

use gtk::glib;
use std::thread;

use crate::{
    error_execution::ErrorExecution,
    process::{
        download, handshake,
        load_system::LoadSystem, 
    }
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig,
    save_config::SaveConfig,
};

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    block_structure::{
        block_chain::BlockChain,
        utxo_set::UTXOSet,
    },
    connections::ibd_methods::IBDMethod,
};

use std::{
    net::{SocketAddr, TcpStream},
};

use std::sync::mpsc;

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
            download::blocks_first::<TcpStream>();
        }
    }

    Ok(())
}

pub fn backend_initialization(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: LoadSystem,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> Result<(), ErrorExecution> {


    let mut load_system = load_system;

    let potential_peers = get_potential_peers(connection_config.clone(), logger.clone())?;

    let mut block_chain = load_system.get_block_chain()?;

    let peer_streams = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        logger.clone(),
    );

    let peer_streams = get_block_chain(
        peer_streams,
        &mut block_chain,
        connection_config.clone(),
        download_config,
        logger.clone(),
    )?;
    let mut wallet = load_system.get_wallet()?;
    for account in wallet.accounts.iter() {
        tx_to_front.send(SignalToFront::RegisterWallet(account.account_name.clone())).unwrap();
    }

    tx_to_front.send(SignalToFront::LoadBlockChain).unwrap();

    let utxo_set = UTXOSet::from_blockchain(&block_chain);

    for rx in rx_from_front {
        match rx {
            SignalToBack::GetAccountBalance(account_name) => {
                let balance = utxo_set.get_balance_in_tbtc(&wallet.get_account_with_name(&account_name).unwrap().address);
                tx_to_front.send(SignalToFront::LoadAvailableBalance(balance)).unwrap();
            },
            _ => {}
        }
    }


    //let block_chain = Arc::new(Mutex::new(block_chain));

    //tx_to_front.send(SignalToFront::LoadBlockChain).unwrap();

    //tx_to_front.send(SignalToFront::LoadBlockChain(load_system.get_block_chain()?)).unwrap();

    println!("Wallet: {:?}", wallet);
    Ok(())
}

pub fn spawn_backend_handler(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let load_system = LoadSystem::new(
            save_config.clone(),
            logger.clone(),
        );
        let _ = backend_initialization(connection_config, download_config, load_system, logger, tx_to_front, rx_from_front);
    })
}