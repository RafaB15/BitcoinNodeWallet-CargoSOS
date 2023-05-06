mod error_initialization;
mod error_ejecution;

use std::{io::BufReader, path::Path};
use std::fs::File;
use std::thread::{self, JoinHandle};

use cargosos_bitcoin::configurations::{
    configuration::config,
    log_config::LogConfig,
};

use cargosos_bitcoin::logs::{
    logger,
    logger_sender::LoggerSender,
    error_log::ErrorLog,
};

use error_initialization::ErrorInitialization;
use error_ejecution::ErrorEjecution;


const DECLARATION_CONFIG: &str = "config";
const DECLARATION_BIG_CONFIG: &str = "configuration";

/// Finds the position of the declaration of the configuration given by  `--config` or `--configuration`
/// 
///  * `ErrorNoGivenFile`: It will appear when there is not `--config` or `--configuration` in the arguments
fn find_config_name(arguments: &Vec<String>) -> Result<usize, ErrorInitialization> {
    
    let config_declarations = &[
        DECLARATION_CONFIG.to_string(), 
        DECLARATION_BIG_CONFIG.to_string()
    ];

    for (index, argument) in arguments.iter().enumerate() {
        if config_declarations.contains(argument) {
            return Ok(index);
        }
    }

    Err(ErrorInitialization::ErrorNoGivenConfigurationFile)
}

/// Get the configuration name given the arguments 
/// 
/// ### Errors
///  * `ErrorNoGivenFile`: It will appear when there is not `--config` or `--configuration` in the arguments or there is not argument pass that configuration declaration
fn get_config_name(arguments: Vec<String>) -> Result<String, ErrorInitialization> {
    
    let config_position: usize = find_config_name(&arguments)?;

    let config_name: String = match arguments.get(config_position + 1) {
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

fn main() -> Result<(), ErrorEjecution> {
    let arguments: Vec<String> = std::env::args().collect();    

    println!("\tInitialization");
    println!("Reading the configuration file");

    let config_name: String = get_config_name(arguments)?;
    let config_file = open_config_file(config_name)?;
    let (log_config, _connection_config) = config::new(config_file)?;

    
    let (handle, logger_sender) = initialize_logs(log_config)?;

    // Ejecutar programa




    logger_sender.log_configuration("Closing program".to_string())?;
    
    std::mem::drop(logger_sender);
    if let Ok(resultado) = handle.join() {
        let _ = resultado?;
    }

    Ok(())
}