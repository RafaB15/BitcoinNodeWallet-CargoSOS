mod error_execution;
mod error_initialization;

use std::net::{
    SocketAddr,
    TcpStream,
};

use std::{
    io::BufReader, 
    path::Path
};

use std::fs::{
    File, 
    OpenOptions
};

use std::thread::{
    self, 
    JoinHandle,
};

use cargosos_bitcoin::block_structure::hash::{
    HashType,
    hash256d,
};

use cargosos_bitcoin::configurations::{
    configuration::config, 
    log_config::LogConfig,
};

use cargosos_bitcoin::logs::{
    logger, 
    logger_sender::LoggerSender,
    error_log::ErrorLog, 
};

use error_execution::ErrorExecution;
use error_initialization::ErrorInitialization;

use cargosos_bitcoin::node_structure::{
    handshake::Handshake,
    initial_block_download::InitialBlockDownload,
    error_node::ErrorNode,
};

use cargosos_bitcoin::serialization::{
    serializable::Serializable,
};

use cargosos_bitcoin::block_structure::{
    block_chain::BlockChain,
    block::Block,
    block_header::BlockHeader,
};

use cargosos_bitcoin::connections::{
    dns_seeder::DNSSeeder,
    p2p_protocol::ProtocolVersionP2P,
    suppored_services::SupportedServices,
    initial_download_method::InitialDownloadMethod,
    error_connection::ErrorConnection,
};

use cargosos_bitcoin::messages::bitfield_services::BitfieldServices;

const MAX_HEADERS: u32 = 2000;

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

    let log_file = match OpenOptions::new().create(true).write(true).truncate(true).open(log_path) {
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
fn initialize_logs(log_config: LogConfig) -> Result<(JoinHandle<Result<(), ErrorLog>>, LoggerSender), ErrorExecution> {
    println!("Creating the logs system");

    let filepath_log = Path::new(&log_config.filepath_log);
    let log_file = open_log_file(filepath_log)?;
    let (logger_sender, logger_receiver) = logger::initialize_logger(log_file, true)?;

    let handle = thread::spawn(move || logger_receiver.receive_log());

    logger_sender.log_configuration("Logs are already configured".to_string())?;

    Ok((handle, logger_sender))
}

fn get_potential_peers(logger_sender: LoggerSender) -> Result<Vec<SocketAddr>, ErrorExecution> {
    logger_sender.log_connection("Getting potential peers with dns seeder".to_string())?;

    let dns_seeder = DNSSeeder::new("seed.testnet.bitcoin.sprovoost.nl", 18333);
    let potential_peers = dns_seeder.discover_peers()?;

    for potential_peer in &potential_peers {
        logger_sender.log_connection(format!("Potential peer: {:?}", potential_peer))?;
    }

    Ok(potential_peers)
}

fn connect_to_testnet_peers(
    potential_peers: Vec<SocketAddr>,
    logger_sender: LoggerSender, 
) -> Result<Vec<TcpStream>, ErrorExecution> 
{
    logger_sender.log_connection("Connecting to potential peers".to_string())?;

    let node = Handshake::new(
        ProtocolVersionP2P::V70015,
        BitfieldServices::new(vec![SupportedServices::Unname]),
        0,
        logger_sender.clone(),
    );

    let mut peer_streams: Vec<TcpStream> = Vec::new();

    for potential_peer in potential_peers {

        let mut peer_stream = match TcpStream::connect(potential_peer) {
            Ok(stream) => stream,
            Err(error) => {
                logger_sender.log_connection(
                    format!("Cannot connect to address: {:?}, it appear {:?}", potential_peer, error)
                )?;
                continue;
            },
        };

        let local_socket = match peer_stream.local_addr() {
            Ok(addr) => addr,
            Err(error) => {
                logger_sender.log_connection(
                    format!("Cannot get local address, it appear {:?}", error)
                )?;
                continue;
            },
        };

        if let Err(error) = node.connect_to_testnet_peer(
            &mut peer_stream,
            &local_socket,
            &potential_peer,
        ) {
            logger_sender.log_connection(
                format!("Error while connecting to addres: {:?}, it appear {:?}", potential_peer, error)
            )?;
            continue;
        };

        peer_streams.push(peer_stream);
    }

    Ok(peer_streams)
}

fn get_peer_header(
    peer_stream: &mut TcpStream,
    block_download: &InitialBlockDownload,
    block_chain: &mut BlockChain,
    logger_sender: &LoggerSender,
) -> Result<(), ErrorExecution> {

    loop {
        let header_count: u32 = match block_download.get_headers(
            peer_stream,
            block_chain,
        ) {
            Err(ErrorNode::NodeNotResponding(message)) => {
                logger_sender.log_connection(
                    format!("Node not responding, send: {}", message)
                )?;
                break;
            },
            other_response => other_response?,
        };

        logger_sender.log_connection(
            format!("We get: {}", header_count)
        )?;

        if header_count < MAX_HEADERS {
            break;
        }        
    }

    Ok(())
}

fn get_blocks_recursive(
    peer_stream: &mut TcpStream,
    block_download: InitialBlockDownload,
    blocks: &mut Vec<Block>,
    block_chain_actual: BlockChain,
) {

    let block_header = block_chain_actual.block.header;

    let mut bytes: Vec<u8> = Vec::new();
    if block_header.serialize(&mut bytes).is_err() {
        return;
    }

    let heashed_header: HashType = match hash256d(&bytes) {
        Ok(heashed_header) => heashed_header,
        Err(_) => return,
    };

    if let Ok(block) = block_download.get_data(
        peer_stream,
        &heashed_header,
    ) {

        blocks.push(block);

        for block_chain in block_chain_actual.next_blocks {
            get_blocks_recursive(
                peer_stream,
                block_download.clone(),
                blocks,
                block_chain,
            );
        }
    }
}

fn get_blocks(
    peer_stream: &mut TcpStream,
    block_download: InitialBlockDownload,
    block_chain: BlockChain,
) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();

    get_blocks_recursive(
        peer_stream, 
        block_download, 
        &mut blocks, 
        block_chain,
    );

    blocks
}

fn get_initial_download_headers_first(
    mut peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    block_download: InitialBlockDownload,
    logger_sender: LoggerSender, 
) -> Result<(), ErrorExecution> 
{
    logger_sender.log_connection("Getting initial download headers first".to_string())?;

    let mut peer_download_handles: Vec<JoinHandle<Vec<Block>>> = Vec::new();

    for peer_stream in peer_streams {
        let mut peer_stream = peer_stream;

        logger_sender.log_connection(
            format!("Connecting to peer: {:?}", peer_stream)
        )?;

        get_peer_header(
            &mut peer_stream,
            &block_download,
            block_chain,
            &logger_sender,
        )?;

        let timestamp: u32 = 40000;
        let partial_block_chain = block_chain.get_block_after_timestamp(timestamp)?;
        let block_download_peer = block_download.clone();

        let peer_download_handle = thread::spawn(move || {
            
            get_blocks(
                &mut peer_stream,
                block_download_peer,
                partial_block_chain,
            )
        });

        peer_download_handles.push(peer_download_handle);

    }

    println!("block_chain: {:?}", block_chain);

    for peer_download_handle in peer_download_handles {
        match peer_download_handle.join() {
            Ok(blocks) => {
                for block in blocks {
                    block_chain.update_block(block)?;
                }
            },
            _ => return Err(ErrorExecution::FailThread),
        }
    }

    Ok(())
}

fn get_block_chain(
    mut peer_streams: Vec<TcpStream>,
    logger_sender: LoggerSender, 
) -> Result<BlockChain, ErrorExecution> 
{    
    logger_sender.log_connection("Getting block chain".to_string())?;

    let method = InitialDownloadMethod::HeadersFirst;

    let block_download = InitialBlockDownload::new(
        ProtocolVersionP2P::V70015, 
    );

    let genesis_header: BlockHeader = BlockHeader::generate_genesis_block_header();
    let genesis_block: Block = Block::new(genesis_header);

    let mut block_chain: BlockChain = BlockChain::new(genesis_block);

    match method {
        InitialDownloadMethod::HeadersFirst => {
            get_initial_download_headers_first(
                peer_streams, 
                &mut block_chain, 
                block_download,
                logger_sender
            )?;
        },
        InitialDownloadMethod::BlocksFirst => todo!(),
    }

    Ok(block_chain)
}

fn main() -> Result<(), ErrorExecution> {
    let arguments: Vec<String> = std::env::args().collect();

    println!("\tInitialization");
    println!("Reading the configuration file");

    let config_name: String = get_config_name(arguments)?;
    let config_file = open_config_file(config_name)?;
    let (log_config, _connection_config) = config::new(config_file)?;
 
    let (handle, logger_sender) = initialize_logs(log_config)?;

    // Ejecutar programa

    {
        let potential_peers = get_potential_peers(logger_sender.clone())?;

        let peer_streams = connect_to_testnet_peers(potential_peers, logger_sender.clone())?;

        let block_chain = get_block_chain(peer_streams, logger_sender.clone())?;

        println!("Block chain: {:?}", block_chain);
    }

    logger_sender.log_configuration("Closing program".to_string())?;

    std::mem::drop(logger_sender);
    match handle.join() {
        Ok(result) => result?,
        _ => return Err(ErrorExecution::FailThread),
    }

    Ok(())
}
