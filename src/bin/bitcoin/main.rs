mod initialization_error;

use initialization_error::InitializationError;
use std::{io::BufReader, path::Path};
use std::fs::File;
use std::thread;
use cargosos_bitcoin::{
    configurations::configuration::config,
    logs::logger,
};

const DECLARATION_CONFIG: &str = "config";
const DECLARATION_BIG_CONFIG: &str = "configuration";

/// Finds the position of the declaration of the configuration given by  `--config` or `--configuration`
/// 
///  * `ErrorNoGivenFile`: It will appear when there is not `--config` or `--configuration` in the arguments
fn find_config_name(arguments: &Vec<String>) -> Result<usize, InitializationError> {
    
    let config_declarations = &[
        DECLARATION_CONFIG.to_string(), 
        DECLARATION_BIG_CONFIG.to_string()
    ];

    for (index, argument) in arguments.iter().enumerate() {
        if config_declarations.contains(argument) {
            return Ok(index);
        }
    }

    Err(InitializationError::ErrorNoGivenFile)
}

/// Get the configuration name given the arguments 
/// 
/// ### Errors
///  * `ErrorNoGivenFile`: It will appear when there is not `--config` or `--configuration` in the arguments or there is not argument pass that configuration declaration
fn get_config_name(arguments: Vec<String>) -> Result<String, InitializationError> {
    
    let config_position: usize = find_config_name(&arguments)?;

    let config_name: String = match arguments.get(config_position + 1) {
        Some(config_name) => config_name.to_owned(),
        None => return Err(InitializationError::ErrorNoGivenFile),
    };

    Ok(config_name)
}

/// Get the file given by its name
/// 
/// ### Errors
/// * `ErrorFileNotExist`: It will appear when the file does not exist
fn open_config_file(config_name: String) -> Result<BufReader<File>, InitializationError> {
    let config_file = match File::open(config_name) {
        Ok(config_file) => config_file,
        Err(_) => return Err(InitializationError::ErrorFileNotExist),
    };

    Ok(BufReader::new(config_file))   
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();    

    println!("Initialization");

    println!("Reading the configuration file");

    let config_name: String = match get_config_name(arguments) {
        Ok(config_name) => config_name,
        Err(err) => return println!("{err}"),
    };

    let config_file = match open_config_file(config_name) {
        Ok(config_file) => config_file,
        Err(err) => return println!("{err}"),
    };

    let (log_config, connection_config) = match config::new(config_file) {
        Ok(configuration) => configuration,
        Err(err) => return println!("An error ocurre: {:?}", err),
    };

    println!("Creating the logs system");

    let filepath_log = Path::new(&log_config.filepath_log);
    let (logger_sender, logger_receiver) = match logger::initialize_logger(filepath_log) {
        Ok((logger_sender, logger_receiver)) => (logger_sender, logger_receiver),
        Err(err) => return println!("An error ocurre: {:?}", err),
    };

    let handle = thread::spawn(move || logger_receiver.receive_log());        

    if let Err(err) = logger_sender.log_configuration("Logs are already configured".to_string()) {
        return println!("An error ocurre: {:?}", err);
    }    
    
    std::mem::drop(logger_sender);
    let _ = handle.join().unwrap();

}