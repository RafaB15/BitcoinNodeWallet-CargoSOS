mod error_initialization;
mod error_execution;

use std::net::SocketAddr;
use std::{io::BufReader, path::Path};
use std::fs::File;
use std::thread::{self, JoinHandle};

use cargosos_bitcoin::configurations::error_configuration::ErrorConfiguration;
use cargosos_bitcoin::configurations::{
    configuration::config,
    log_config::LogConfig,
};

use cargosos_bitcoin::connections::error_connection::ErrorConnection;
use cargosos_bitcoin::logs::{
    logger,
    logger_sender::LoggerSender,
    error_log::ErrorLog,
};

use error_initialization::ErrorInitialization;
use error_execution::ErrorExecution;

use cargosos_bitcoin::node_structure::handshake::Handshake;

use cargosos_bitcoin::connections::{
    dns_seeder::DNSSeeder,
    p2p_protocol::ProtocolVersionP2P,
    suppored_services::SupportedServices,
};

use cargosos_bitcoin::messages::bitfield_services::BitfieldServices;

/// Get the configuration name given the arguments 
/// 
/// ### Errors
///  * `ErrorNoGivenFile`: It will appear when there is not argument pass that configuration declaration
fn get_config_name(arguments: Vec<String>) -> Result<String, ErrorInitialization> {
    let config_name: String = match arguments.get(1) {
        Some(config_name) => config_name.to_owned(),
        None => return Err(ErrorInitialization::ErrorNoGivenConfigurationFile),
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
        Err(_) => return Err(ErrorInitialization::ErrorConfigurationFileDoesntExist),
    };

    Ok(BufReader::new(config_file))   
}

/// Initialize the logs ready for ejecution
/// 
/// ### Errors
///  * `ErrorFileNotFound`: No se encontro el file
///  * `ErrorCouldNotWriteInFile`: No se pudo escribir en el file
///  * `ErrorCouldNotFindReceiver`: No se encontro el receiver
///  * `ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
fn initialize_logs(log_config: LogConfig) -> Result<(JoinHandle<Result<(), ErrorLog>>, LoggerSender), ErrorLog> {
    println!("Creating the logs system");

    let filepath_log = Path::new(&log_config.filepath_log);
    let (logger_sender, logger_receiver) = logger::initialize_logger(filepath_log)?;

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

fn connect_to_testnet_peers(logger_sender: LoggerSender, potential_peers: Vec<SocketAddr>) -> Result<Vec<SocketAddr>, ErrorExecution> {
    logger_sender.log_connection("Connecting to potential peers".to_string())?;

    let mut node = Handshake::new(
        ProtocolVersionP2P::V70015,
        BitfieldServices::new(vec![SupportedServices::Unname]),
        0,
        logger_sender.clone(),  
    );

    let peers = node.connect_to_testnet_peers(&potential_peers)?;
    Ok(peers)
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

        let peers = connect_to_testnet_peers(logger_sender.clone(), potential_peers)?;
    }

    logger_sender.log_configuration("Closing program".to_string())?;
    
    std::mem::drop(logger_sender);
    if let Ok(resultado) = handle.join() {
        resultado?;
    }

    Ok(())
}