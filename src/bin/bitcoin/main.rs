mod error_execution;
mod error_initialization;
mod process;

use std::net::{SocketAddr, TcpStream};

use std::{io::BufReader, path::Path};

use std::fs::{File, OpenOptions};

use std::thread::{self, JoinHandle};

use process::{configuration::Configuration, download, handshake};

use error_execution::ErrorExecution;
use error_initialization::ErrorInitialization;

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig, log_config::LogConfig,
    save_config::SaveConfig,
};

use cargosos_bitcoin::logs::{error_log::ErrorLog, logger, logger_sender::LoggerSender};

use cargosos_bitcoin::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    serializable_internal_order::SerializableInternalOrder,
};

use cargosos_bitcoin::node_structure::{
    block_download::BlockDownload, initial_headers_download::InitialHeaderDownload,
};

use cargosos_bitcoin::block_structure::{
    block::Block, block_chain::BlockChain, block_header::BlockHeader,
};

use cargosos_bitcoin::connections::ibd_methods::IBDMethod;

/// Get the configuration name given the arguments
///
/// ### Errors
///  * `ErrorNoGivenFile`: It will appear when there is not argument pass that configuration declaration
fn get_config_name(arguments: Vec<String>) -> Result<String, ErrorInitialization> {
    let config_name: String = match arguments.get(1) {
        Some(config_name) => config_name.to_owned(),
        None => return Err(ErrorInitialization::NoGivenConfigurationFile),
    };

    Ok(config_name)
}

/// Get the file given by its name
///
/// ### Errors
/// * `ErrorFileNotExist`: It will appear when the file does not exist
fn open_config_file(config_name: String) -> Result<BufReader<File>, ErrorInitialization> {
    let config_file = match File::open(config_name) {
        Ok(config_file) => config_file,
        Err(_) => return Err(ErrorInitialization::ConfigurationFileDoesntExist),
    };

    Ok(BufReader::new(config_file))
}

/// Get the file given by its path. If the file does not exist, it will be created. Evrytime the file is opened, it will be truncated to set the file size to 0 and overwrite the previous content
///
/// ### Errors
/// * `ErrorFileNotExist`: It will appear when the file does not exist
/// * `CouldNotTruncateFile`: It will appear when the file could not be truncated
fn open_log_file(log_path: &Path) -> Result<File, ErrorInitialization> {
    let log_file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
    {
        Ok(log_file) => log_file,
        _ => return Err(ErrorInitialization::LogFileDoesntExist),
    };

    Ok(log_file)
}

/// Initialize the logs ready for ejecution
///
/// ### Errors
///  * `ErrorCouldNotFindReceiver`: No se encontro el receiver
///  * `ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
///  * `ErrorLogFileDoesNotExist`: No se encontro el archivo de logs
///  * `CouldNotTruncateFile`: No se pudo truncar el archivo de logs
fn initialize_logs(
    log_config: LogConfig,
) -> Result<(JoinHandle<Result<(), ErrorLog>>, LoggerSender), ErrorExecution> {
    println!("Creating the logs system");

    let filepath_log = Path::new(&log_config.filepath_log);
    let log_file = open_log_file(filepath_log)?;
    let (logger_sender, logger_receiver) =
        logger::initialize_logger(log_file, log_config.show_console)?;

    let handle = thread::spawn(move || logger_receiver.receive_log());

    logger_sender.log_configuration("Logs are already configured".to_string())?;

    Ok((handle, logger_sender))
}

fn get_potential_peers(
    connection_config: ConnectionConfig,
    logger_sender: LoggerSender,
) -> Result<Vec<SocketAddr>, ErrorExecution> {
    logger_sender.log_connection("Getting potential peers with dns seeder".to_string())?;

    let potential_peers = connection_config.dns_seeder.discover_peers()?;

    let peer_count_max = std::cmp::min(connection_config.peer_count_max, potential_peers.len());

    let potential_peers = potential_peers[0..peer_count_max].to_vec();

    for potential_peer in &potential_peers {
        logger_sender.log_connection(format!("Potential peer: {:?}", potential_peer))?;
    }

    Ok(potential_peers)
}

fn get_block_chain(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    logger_sender.log_connection("Getting block chain".to_string())?;

    let header_download = InitialHeaderDownload::new(
        connection_config.p2p_protocol_version,
        logger_sender.clone(),
    );

    let block_download = BlockDownload::new(logger_sender.clone());

    match connection_config.ibd_method {
        IBDMethod::HeaderFirst => {
            download::headers_first(
                peer_streams,
                block_chain,
                header_download,
                block_download,
                download_config,
                logger_sender,
            )?;
        }
        IBDMethod::BlocksFirst => {
            download::blocks_first();
        }
    }

    Ok(())
}

fn get_initial_block_chain(
    posible_path: Option<String>,
    logger_sender: LoggerSender,
) -> JoinHandle<Result<BlockChain, ErrorExecution>> {
    thread::spawn(move || {
        if let Some(path) = posible_path {
            if let Ok(file) = OpenOptions::new().read(true).open(path) {
                let mut file = BufReader::new(file);

                let _ =
                    logger_sender.log_connection("Reading the blockchain from file".to_string());

                let block_chain = BlockChain::io_deserialize(&mut file)?;

                let _ = logger_sender.log_connection("Blockchain loaded from file".to_string());

                return Ok(block_chain);
            }

            let _ = logger_sender.log_connection("Could not open file".to_string());
        }

        let genesis_header: BlockHeader = BlockHeader::generate_genesis_block_header();
        let genesis_block: Block = Block::new(genesis_header);

        let _ =
            logger_sender.log_connection("Initializing blockchain from genesis block".to_string());

        Ok(BlockChain::new(genesis_block)?)
    })
}

fn save_block_chain(
    block_chain: &BlockChain,
    posible_path: Option<String>,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    let path = match posible_path {
        Some(path) => path,
        None => {
            let _ = logger_sender.log_connection("No path to save the blockchain".to_string());

            return Ok(());
        }
    };

    let mut file = match OpenOptions::new().create(true).write(true).open(path) {
        Ok(file) => file,
        _ => return Err(ErrorInitialization::BlockchainFileDoesntExist.into()),
    };

    let _ = logger_sender.log_connection("Writting the blockchain to file".to_string());

    block_chain.io_serialize(&mut file)?;

    Ok(())
}

fn show_merkle_path(
    block_chain: &BlockChain,
    logger_sender: LoggerSender,
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

    logger_sender.log_connection(format!(
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

    logger_sender.log_connection(format!("And transaction: \n{:?}", transaction,))?;

    let merkle_path = last_block.get_merkle_path(transaction)?;

    let mut path: String = "\n".to_string();
    for hash in merkle_path {
        path = format!("{path}\t{:?}\n", hash);
    }

    logger_sender.log_connection(format!("We get the merkle path: {path}"))?;

    Ok(())
}

fn show_utxo_set(block_chain: &BlockChain, logger_sender: LoggerSender) {
    let max_transaction_count: usize = 20;
    let utxo_vec = block_chain.get_utxo();

    let mut path: String = "\n".to_string();
    for utxo in utxo_vec[0..max_transaction_count].iter().cloned() {
        path = format!("{path}\tTransactionOutput {{ value: {:?} }}\n", utxo.value);
    }

    let _ = logger_sender.log_connection(format!("We get the merkle path: {path}"));
}

fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    let block_chain_handle =
        get_initial_block_chain(save_config.read_block_chain, logger_sender.clone());

    let potential_peers = get_potential_peers(connection_config.clone(), logger_sender.clone())?;

    let peer_streams = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        logger_sender.clone(),
    );

    let mut block_chain = match block_chain_handle.join() {
        Ok(block_chain) => block_chain?,
        _ => return Err(ErrorExecution::FailThread),
    };

    get_block_chain(
        peer_streams,
        &mut block_chain,
        connection_config,
        download_config,
        logger_sender.clone(),
    )?;

    show_merkle_path(&block_chain, logger_sender.clone())?;

    show_utxo_set(&block_chain, logger_sender.clone());

    save_block_chain(&block_chain, save_config.write_block_chain, logger_sender)?;

    Ok(())
}

fn main() -> Result<(), ErrorExecution> {
    let arguments: Vec<String> = std::env::args().collect();

    println!("\tInitialization");
    println!("Reading the configuration file");

    let config_name: String = get_config_name(arguments)?;
    let config_file = open_config_file(config_name)?;

    let configuration = Configuration::new(config_file)?;
    let (log_config, connection_config, download_config, save_config) = configuration.separate();

    let (handle, logger_sender) = initialize_logs(log_config)?;

    program_execution(
        connection_config,
        download_config,
        save_config,
        logger_sender.clone(),
    )?;

    logger_sender.log_configuration("Closing program".to_string())?;

    std::mem::drop(logger_sender);
    match handle.join() {
        Ok(result) => result?,
        _ => return Err(ErrorExecution::FailThread),
    }

    Ok(())
}
