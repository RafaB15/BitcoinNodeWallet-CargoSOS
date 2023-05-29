mod configuration;
mod error_execution;
mod error_initialization;

use std::net::{SocketAddr, TcpStream};

use std::{io::BufReader, path::Path};

use std::fs::{File, OpenOptions};

use std::thread::{self, JoinHandle};

use configuration::Configuration;
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
    block_broadcasting::BlockBroadcasting, block_download::BlockDownload, error_node::ErrorNode,
    handshake::Handshake, initial_headers_download::InitialHeaderDownload,
};

use cargosos_bitcoin::block_structure::{
    block::Block, block_chain::BlockChain, block_header::BlockHeader, hash::HashType,
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

fn connect_to_testnet_peers(
    potential_peers: Vec<SocketAddr>,
    connection_config: ConnectionConfig,
    logger_sender: LoggerSender,
) -> Result<Vec<TcpStream>, ErrorExecution> {
    logger_sender.log_connection("Connecting to potential peers".to_string())?;

    let node = Handshake::new(
        connection_config.p2p_protocol_version,
        connection_config.services,
        connection_config.block_height,
        logger_sender.clone(),
    );

    let mut peer_streams: Vec<TcpStream> = Vec::new();

    for potential_peer in potential_peers {
        let mut peer_stream = match TcpStream::connect(potential_peer) {
            Ok(stream) => stream,
            Err(error) => {
                logger_sender.log_connection(format!(
                    "Cannot connect to address: {:?}, it appear {:?}",
                    potential_peer, error
                ))?;
                continue;
            }
        };

        let local_socket = match peer_stream.local_addr() {
            Ok(addr) => addr,
            Err(error) => {
                logger_sender
                    .log_connection(format!("Cannot get local address, it appear {:?}", error))?;
                continue;
            }
        };

        if let Err(error) =
            node.connect_to_testnet_peer(&mut peer_stream, &local_socket, &potential_peer)
        {
            logger_sender.log_connection(format!(
                "Error while connecting to addres: {:?}, it appear {:?}",
                potential_peer, error
            ))?;
            continue;
        };

        peer_streams.push(peer_stream);
    }

    Ok(peer_streams)
}

fn get_peer_header(
    peer_stream: &mut TcpStream,
    header_download: &InitialHeaderDownload,
    block_chain: &mut BlockChain,
    logger_sender: &LoggerSender,
) -> Result<(), ErrorExecution> {
    loop {
        let header_count: u32 = match header_download.get_headers(peer_stream, block_chain) {
            Err(ErrorNode::NodeNotResponding(message)) => {
                logger_sender.log_connection(format!("Node not responding, send: {}", message))?;
                break;
            }
            other_response => other_response?,
        };

        logger_sender.log_connection(format!("We get: {}", header_count))?;

        if header_count == 0 {
            break;
        }
    }

    Ok(())
}

fn get_blocks(
    peer_stream: &mut TcpStream,
    block_download: BlockDownload,
    list_of_blocks: Vec<Block>,
    logger_sender: LoggerSender,
) -> Vec<Block> {
    let mut headers: Vec<HashType> = Vec::new();

    for block in list_of_blocks {
        let header_hash = match block.header.get_hash256d() {
            Ok(header_hash) => header_hash,
            Err(_) => continue,
        };

        headers.push(header_hash);
    }

    match block_download.get_data(peer_stream, headers) {
        Ok(blocks) => blocks,
        Err(error) => {
            let _ = logger_sender.log_connection(format!("Cannot get block, we get {:?}", error));
            vec![]
        }
    }
}

fn updating_block_chain(
    block_chain: &mut BlockChain,
    peer_download_handles: Vec<JoinHandle<Vec<Block>>>,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    for peer_download_handle in peer_download_handles {
        logger_sender.log_connection("Finish downloading, loading to blockchain".to_string())?;
        match peer_download_handle.join() {
            Ok(blocks) => {
                logger_sender
                    .log_connection(format!("Loading {} blocks to blockchain", blocks.len(),))?;

                for (i, block) in blocks.iter().enumerate() {
                    block_chain.update_block(block.clone())?;

                    if i % 50 == 0 {
                        logger_sender
                            .log_connection(format!("Loading [{i}] blocks to blockchain",))?;
                    }
                }
            }
            _ => return Err(ErrorExecution::FailThread),
        }
    }

    Ok(())
}

fn get_initial_download_headers_first(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    header_download: InitialHeaderDownload,
    block_download: BlockDownload,
    download_config: DownloadConfig,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    logger_sender.log_connection("Getting initial download headers first".to_string())?;

    let mut peer_download_handles: Vec<JoinHandle<Vec<Block>>> = Vec::new();

    for peer_stream in peer_streams {
        let mut peer_stream = peer_stream;

        logger_sender.log_connection(format!("Connecting to peer: {:?}", peer_stream))?;

        get_peer_header(
            &mut peer_stream,
            &header_download,
            block_chain,
            &logger_sender,
        )?;

        let list_of_blocks = block_chain.get_blocks_after_timestamp(download_config.timestamp)?;

        let block_download_peer = block_download.clone();

        let logger_sender_clone = logger_sender.clone();
        let peer_download_handle = thread::spawn(move || {
            get_blocks(
                &mut peer_stream,
                block_download_peer,
                list_of_blocks,
                logger_sender_clone,
            )
        });

        peer_download_handles.push(peer_download_handle);
    }

    updating_block_chain(block_chain, peer_download_handles, logger_sender)
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
            get_initial_download_headers_first(
                peer_streams,
                block_chain,
                header_download,
                block_download,
                download_config,
                logger_sender,
            )?;
        }
        IBDMethod::BlocksFirst => todo!(),
    }

    Ok(())
}

fn _block_broadcasting(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    logger_sender.log_connection("Broadcasting...".to_string())?;

    let block_broadcasting = BlockBroadcasting::new(logger_sender.clone());

    let blocks_download = BlockDownload::new(logger_sender.clone());

    let mut peer_download_handles: Vec<JoinHandle<Vec<Block>>> = Vec::new();

    for peer_stream in peer_streams {
        let mut peer_stream = peer_stream;

        let (header_count, headers) = match block_broadcasting
            .get_new_headers(&mut peer_stream, block_chain)
        {
            Err(ErrorNode::NodeNotResponding(message)) => {
                logger_sender.log_connection(format!("Node not responding, send: {}", message))?;
                break;
            }
            other_response => other_response?,
        };

        logger_sender.log_connection(format!("We get: {}", header_count))?;

        let blocks = headers.iter().map(|header| Block::new(*header)).collect();

        let logger_sender_clone = logger_sender.clone();
        let peer_block_download = blocks_download.clone();

        let peer_download_handle = thread::spawn(move || {
            get_blocks(
                &mut peer_stream,
                peer_block_download,
                blocks,
                logger_sender_clone,
            )
        });

        peer_download_handles.push(peer_download_handle);
    }

    updating_block_chain(block_chain, peer_download_handles, logger_sender)
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

    let peer_streams = connect_to_testnet_peers(
        potential_peers,
        connection_config.clone(),
        logger_sender.clone(),
    )?;

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
